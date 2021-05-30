use cgmath::Point2;
use glium::glutin::event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode};
use std::collections::HashSet;
use std::ops::Index;

const DRAG_THRESHOLD: u32 = 3;

pub const KEYBD_MOVE_SPEED: f64 = 1000.0;
pub const KEYBD_SCALE_SPEED: f64 = 4.0;

// Define keyboard scancodes. OSX scancodes are from
// https://eastmanreference.com/complete-list-of-applescript-key-codes
#[cfg(any(target_os = "macos"))]
pub mod sc {
    pub const W: u32 = 13;
    pub const A: u32 = 0;
    pub const S: u32 = 1;
    pub const D: u32 = 2;
    pub const Q: u32 = 12;
    pub const E: u32 = 14;
    pub const Z: u32 = 6;
}
#[cfg(not(any(target_os = "macos")))]
pub mod sc {
    pub const W: u32 = 17;
    pub const A: u32 = 30;
    pub const S: u32 = 31;
    pub const D: u32 = 32;
    pub const Q: u32 = 16;
    pub const E: u32 = 18;
    pub const Z: u32 = 44;
}

#[derive(Debug, Copy, Clone)]
pub struct Drag {
    pub button: MouseButton,
    pub tile_coords: Point2<f64>,
    pub initial_scale_factor: f64,

    pub cursor_start: (u32, u32),
    pub cursor_end: (u32, u32),
    pub past_threshold: bool,

    pub kind: DragKind,
}
impl Drag {
    pub fn update_cursor_end(&mut self, (x, y): (u32, u32)) {
        self.cursor_end = (x, y);
        if (self.cursor_start.0 as i32 - x as i32).abs() as u32 >= DRAG_THRESHOLD
            || (self.cursor_start.1 as i32 - y as i32).abs() as u32 >= DRAG_THRESHOLD
        {
            self.past_threshold = true;
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DragKind {
    Pan,
    Scale,
}

#[derive(Debug, Default, Clone)]
pub struct KeysPressed {
    /// Set of scancodes for keys that are held.
    scancodes: HashSet<u32>,
    /// Set of virtual keycodes for keys that are held.
    virtual_keycodes: HashSet<VirtualKeyCode>,
}
impl KeysPressed {
    /// Updates internal key state based on a KeyboardInput event.
    pub fn update(&mut self, input: &KeyboardInput) {
        match input.state {
            ElementState::Pressed => {
                self.scancodes.insert(input.scancode);
                if let Some(virtual_keycode) = input.virtual_keycode {
                    self.virtual_keycodes.insert(virtual_keycode);
                }
            }
            ElementState::Released => {
                self.scancodes.remove(&input.scancode);
                if let Some(virtual_keycode) = input.virtual_keycode {
                    self.virtual_keycodes.remove(&virtual_keycode);
                }
            }
        }
    }
}
impl Index<u32> for KeysPressed {
    type Output = bool;
    fn index(&self, scancode: u32) -> &bool {
        if self.scancodes.contains(&scancode) {
            &true
        } else {
            &false
        }
    }
}
impl Index<VirtualKeyCode> for KeysPressed {
    type Output = bool;
    fn index(&self, virtual_keycode: VirtualKeyCode) -> &bool {
        if self.virtual_keycodes.contains(&virtual_keycode) {
            &true
        } else {
            &false
        }
    }
}
