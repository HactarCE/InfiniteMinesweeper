use glium::glutin::event::{
    ElementState, KeyboardInput, ModifiersState, MouseButton, MouseScrollDelta, WindowEvent,
};

mod camera;
mod grid;
mod input;
mod scale;
mod tile;

pub use camera::Camera;
pub use grid::{Chunk, ChunkPos, Grid, TilePos, CHUNK_SIZE};
pub use scale::Scale;
pub use tile::{FlagState, HiddenState, Tile};

#[derive(Debug, Default, Clone)]
pub struct Game {
    pub grid: Grid,
    pub camera: Camera,
    cursor_pos: Option<(u32, u32)>,

    drag: Option<input::Drag>,

    moved_this_frame: bool,
}
impl Game {
    /// Returns a new game.
    pub fn new() -> Self {
        Game::default()
    }

    /// Updates camera according to a drag.
    pub fn update_camera_for_drag(&mut self) {
        if let Some(drag) = &mut self.drag {
            if drag.past_threshold {
                match drag.kind {
                    input::DragKind::Pan => {
                        let start = drag.tile_coords;
                        let end = self.camera.pixel_to_tile_coords(drag.cursor_end);
                        let new_center = self.camera.center() + (start - end);
                        self.camera.set_center(new_center);
                    }
                    input::DragKind::Scale => {
                        let y1 = drag.cursor_start.1 as f64;
                        let y2 = drag.cursor_end.1 as f64;
                        let delta = (y2 - y1) / -camera::PIXELS_PER_2X_SCALE;
                        let initial = Scale::from_factor(drag.initial_scale_factor);
                        let new_scale = Scale::from_log2_factor(initial.log2_factor() + delta);
                        self.camera.set_scale(new_scale);
                    }
                }
            }
            self.moved_this_frame = true;
        }
    }

    pub fn handle_event(&mut self, ev: WindowEvent<'_>) {
        match ev {
            // Handle keyboard input.
            WindowEvent::KeyboardInput { input, .. } => self.handle_keyboard_input(input),
            // Handle keyboard modifies.
            WindowEvent::ModifiersChanged(modifiers_state) => {
                self.update_modifiers(modifiers_state)
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
                        self.update_camera_for_drag();
                    }
                }
            }
            WindowEvent::CursorLeft { .. } => self.cursor_pos = None,

            // Handle mouse wheel.
            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(dy, _) => todo!(),
                MouseScrollDelta::PixelDelta(dy) => todo!(),
            },

            // Handle mouse click.
            WindowEvent::MouseInput { state, button, .. } => match state {
                ElementState::Pressed => self.handle_mouse_press(button),
                ElementState::Released => self.handle_mouse_release(button),
            },

            _ => (),
        }
    }

    fn handle_keyboard_input(&mut self, input: KeyboardInput) {}

    fn update_modifiers(&mut self, modifiers_state: ModifiersState) {}

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
            MouseButton::Left => self.grid.set_tile(tile_pos, Tile::Number(0)),
            MouseButton::Right => self.grid.toggle_flag(tile_pos),
            MouseButton::Middle => (),
            MouseButton::Other(_) => (),
        }
    }

    pub fn do_frame(&mut self) {}
}
