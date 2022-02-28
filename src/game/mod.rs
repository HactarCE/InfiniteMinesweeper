use cgmath::{Point2, Vector2};
use glium::glutin::event::{
    ElementState, ModifiersState, MouseButton, MouseScrollDelta, ScanCode, VirtualKeyCode,
    WindowEvent,
};
use std::fmt;
use std::str::FromStr;
use std::time::Duration;

mod camera;
mod grid;
mod input;
mod scale;
mod tile;

pub use camera::Camera;
pub use grid::{Chunk, ChunkPos, Grid, TilePos, CHUNK_SIZE};
pub use scale::Scale;
pub use tile::{FlagState, HiddenState, Tile};

pub const MINE_DENSITY: f64 = 0.2;
pub const SAVE_FILE_NAME: &str = "infinite_minesweeper_data.txt";

#[derive(Debug, Default, Clone)]
pub struct Game {
    /// Tile grid.
    pub grid: Grid,
    /// Camera.
    pub camera: Camera,
    /// Interpolation target camera.
    pub camera_target: Camera,

    /// Position of the mouse cursor.
    cursor_pos: Option<(u32, u32)>,
    /// Mouse drag in progress.
    drag: Option<input::Drag>,

    /// Set of pressed keys.
    keys: input::KeysPressed,
    /// Set of pressed modifiers.
    modifiers: ModifiersState,
}
impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cam_pos = self.camera_target.center();
        write!(f, "{},{}*\n\n{}", cam_pos.x, cam_pos.y, self.grid)
    }
}
impl FromStr for Game {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut ret = Self::new();

        let (cam_pos, grid) = s.split_once('*').ok_or(())?;
        let (cam_x, cam_y) = cam_pos.split_once(',').ok_or(())?;

        ret.camera_target.set_center(Point2::new(
            cam_x.trim().parse().map_err(|_| ())?,
            cam_y.trim().parse().map_err(|_| ())?,
        ));
        ret.grid = grid.parse()?;

        Ok(ret)
    }
}
impl Game {
    /// Returns a new game.
    pub fn new() -> Self {
        Game::default()
    }

    /// Updates camera according to a drag.
    pub fn update_camera_for_drag(cam: &mut Camera, drag: input::Drag) {
        if drag.past_threshold {
            match drag.kind {
                input::DragKind::Pan => {
                    let start = drag.tile_coords;
                    let end = cam.pixel_to_tile_coords(drag.cursor_end);
                    let new_center = cam.center() + (start - end);
                    cam.set_center(new_center);
                }
                input::DragKind::Scale => {
                    let y1 = drag.cursor_start.1 as f64;
                    let y2 = drag.cursor_end.1 as f64;
                    let delta = (y2 - y1) / -camera::PIXELS_PER_2X_SCALE;
                    let initial = Scale::from_factor(drag.initial_scale_factor);
                    let new_scale = Scale::from_log2_factor(initial.log2_factor() + delta);
                    cam.set_scale(new_scale);
                }
            }
        }
    }

    pub fn handle_event(&mut self, ev: WindowEvent<'_>) {
        match ev {
            // Handle keyboard input.
            WindowEvent::KeyboardInput { input, .. } => {
                self.keys.update(&input);
                let sc = input.scancode;
                let vkc = input.virtual_keycode;
                match input.state {
                    ElementState::Pressed => self.handle_key_press(sc, vkc),
                    ElementState::Released => self.handle_key_release(sc, vkc),
                }
            }
            // Handle keyboard modifies.
            WindowEvent::ModifiersChanged(modifiers_state) => {
                self.modifiers = modifiers_state;
            }

            // Handle cursor events.
            WindowEvent::CursorMoved { position, .. } => {
                let pos = (position.x as u32, position.y as u32);
                // Update cursor position.
                self.cursor_pos = Some(pos);
                // Update drag in progress.
                if let Some(d) = &mut self.drag {
                    d.update_cursor_end(pos);
                    if d.past_threshold {
                        Self::update_camera_for_drag(&mut self.camera, *d);
                        Self::update_camera_for_drag(&mut self.camera_target, *d);
                    }
                }
            }
            WindowEvent::CursorLeft { .. } => self.cursor_pos = None,

            // Handle mouse wheel.
            WindowEvent::MouseWheel { delta, .. } => self.handle_mouse_wheel(delta),

            // Handle mouse click.
            WindowEvent::MouseInput { state, button, .. } => match state {
                ElementState::Pressed => self.handle_mouse_press(button),
                ElementState::Released => self.handle_mouse_release(button),
            },

            _ => (),
        }
    }

    fn handle_key_press(&mut self, _sc: ScanCode, vkc: Option<VirtualKeyCode>) {
        if vkc == Some(VirtualKeyCode::S) && self.modifiers == ModifiersState::CTRL {
            self.save_to_file();
        }
    }
    fn handle_key_release(&mut self, _sc: ScanCode, _vkc: Option<VirtualKeyCode>) {}

    fn handle_mouse_wheel(&mut self, delta: MouseScrollDelta) {
        let dy = match delta {
            MouseScrollDelta::LineDelta(_dx, dy) => dy as f64,
            MouseScrollDelta::PixelDelta(delta) => delta.y,
        };

        let invariant_pos = if let Some(pixel) = self.cursor_pos {
            Some(self.camera.pixel_to_tile_coords(pixel))
        } else {
            None
        };

        if !self.is_drag_scaling() {
            self.camera_target.scale_by_log2_factor(dy, invariant_pos);
        }
    }

    fn handle_mouse_press(&mut self, button: MouseButton) {
        if self.drag.is_some() {
            return;
        }

        let pixel = match self.cursor_pos {
            Some(pixel) => pixel,
            None => return,
        };

        let drag_kind = match button {
            MouseButton::Left | MouseButton::Right => input::DragKind::Pan,
            MouseButton::Middle => input::DragKind::Scale,
            _ => return,
        };

        self.drag = Some(input::Drag {
            button,
            tile_coords: self.camera.pixel_to_tile_coords(pixel),
            initial_scale_factor: self.camera.scale().factor(),

            cursor_start: pixel,
            cursor_end: pixel,
            past_threshold: false,

            kind: drag_kind,
        });
    }
    fn handle_mouse_release(&mut self, button: MouseButton) {
        let tile_pos = match self.cursor_pos {
            Some(pixel) => self.camera.pixel_to_tile_pos(pixel),
            None => return,
        };

        if let Some(d) = self.drag {
            if button == d.button {
                self.drag = None;
                if d.past_threshold {
                    return;
                }
            } else {
                return;
            }
        }

        match button {
            MouseButton::Left => self.grid.reveal(tile_pos),
            MouseButton::Right => self.grid.toggle_flag(tile_pos),
            MouseButton::Middle => (),
            MouseButton::Other(_) => (),
        }
    }

    pub fn do_frame(&mut self, frame_duration: Duration) {
        self.camera_target
            .set_target_dimensions(self.camera.target_dimensions());

        let mut dx = 0.0;
        let mut dy = 0.0;
        let mut dz = 0.0;

        if !self.modifiers.ctrl() && !self.modifiers.alt() && !self.modifiers.logo() {
            use input::sc;
            dx += self.keys[sc::D] as u32 as f64;
            dx -= self.keys[sc::A] as u32 as f64;
            dy += self.keys[sc::W] as u32 as f64;
            dy -= self.keys[sc::S] as u32 as f64;
            dz += self.keys[sc::Q] as u32 as f64;
            dz -= (self.keys[sc::Z] || self.keys[sc::E]) as u32 as f64;
            if self.modifiers.shift() {
                dx *= 2.0;
                dy *= 2.0;
                dz *= 2.0;
            }
        }

        let pan_delta = Vector2::new(dx, dy) * input::KEYBD_MOVE_SPEED
            / self.camera_target.scale().factor()
            * frame_duration.as_secs_f64();
        self.camera_target.pan(pan_delta);

        let scale_delta = dz * input::KEYBD_SCALE_SPEED * frame_duration.as_secs_f64();
        self.camera_target.scale_by_log2_factor(scale_delta, None);

        if dz == 0.0 && !self.is_drag_scaling() {
            self.camera_target.snap_scale(None);
        }

        self.camera
            .advance_interpolation(self.camera_target, frame_duration);
    }

    fn is_drag_scaling(&self) -> bool {
        if let Some(d) = self.drag {
            d.kind == input::DragKind::Scale
        } else {
            false
        }
    }

    pub fn save_to_file(&self) {
        match self.try_save_to_file() {
            Ok(()) => eprintln!(
                "Saved game to {}",
                Self::get_data_file_path().unwrap().display(),
            ),
            Err(()) => eprintln!("Failed to save game data"),
        }
    }
    pub fn load_from_file() -> Self {
        Self::try_load_from_file().unwrap_or_else(|| {
            eprintln!("Unable to load existing game data; starting new game");
            Game::new()
        })
    }

    pub fn try_save_to_file(&self) -> Result<(), ()> {
        std::fs::write(Self::get_data_file_path().ok_or(())?, self.to_string()).map_err(|_| ())
    }
    pub fn try_load_from_file() -> Option<Self> {
        std::fs::read_to_string(Self::get_data_file_path()?)
            .ok()?
            .parse()
            .ok()
    }
    fn get_data_file_path() -> Option<std::path::PathBuf> {
        let mut path = std::env::current_exe().ok()?.parent()?.to_path_buf();
        path.push(SAVE_FILE_NAME);
        Some(path)
    }
}
