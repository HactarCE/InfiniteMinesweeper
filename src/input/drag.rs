use cgmath::Point2;

const DRAG_THRESHOLD: u32 = 1;

#[derive(Debug, Copy, Clone)]
pub struct Drag {
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
