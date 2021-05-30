use glium::glutin::event::{Event, StartCause, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::glutin::window::{Icon, WindowBuilder};
use glium::glutin::ContextBuilder;
use glium::Surface;
use lazy_static::lazy_static;
use send_wrapper::SendWrapper;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::grid::Scale;
use crate::render;

lazy_static! {
    static ref EVENT_LOOP: SendWrapper<RefCell<Option<EventLoop<()>>>> =
        SendWrapper::new(RefCell::new(Some(EventLoop::new())));
    pub static ref DISPLAY: SendWrapper<glium::Display> = SendWrapper::new({
        let wb = WindowBuilder::new().with_title(crate::TITLE.to_owned());
        let cb = ContextBuilder::new().with_vsync(true);
        glium::Display::new(wb, cb, EVENT_LOOP.borrow().as_ref().unwrap())
            .expect("Failed to initialize display")
    });
}

pub fn show_gui() -> ! {
    let display = &**DISPLAY;

    // Initialize runtime data.
    let mut grid = crate::grid::Grid::new();
    let mut camera = crate::grid::Camera::new();
    let mut events_buffer = VecDeque::new();

    // Main loop.
    let mut last_frame_time = Instant::now();
    let mut next_frame_time = Instant::now();
    let mut frame_count = 0;
    let ev_loop = EVENT_LOOP.borrow_mut().take().unwrap();
    ev_loop.run(move |event, _ev_loop, control_flow| {
        // Handle events.
        let mut now = Instant::now();
        let mut do_frame = false;
        match event.to_static() {
            Some(Event::NewEvents(cause)) => match cause {
                StartCause::ResumeTimeReached {
                    start: _,
                    requested_resume,
                } => {
                    now = requested_resume;
                    do_frame = true;
                }
                StartCause::Init => {
                    next_frame_time = now;
                    do_frame = true;
                }
                _ => (),
            },

            // The program is about to exit.
            Some(Event::LoopDestroyed) => (),

            // Queue the event to be handled next time we render
            // everything.
            Some(ev) => events_buffer.push_back(ev),

            // Ignore this event.
            None => (),
        }

        if do_frame && next_frame_time <= now {
            frame_count += 1;

            let frame_duration = Duration::from_secs_f64(1.0 / 60.0);

            next_frame_time = now + frame_duration;
            if next_frame_time < Instant::now() {
                // Skip a frame (or several).
                next_frame_time = Instant::now() + frame_duration;
            }
            *control_flow = ControlFlow::WaitUntil(next_frame_time);

            for ev in events_buffer.drain(..) {
                // Handle events.
                match ev {
                    Event::WindowEvent { event, .. } => match event {
                        // Handle window close event.
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,

                        // Handle keyboard input.
                        WindowEvent::KeyboardInput {
                            device_id,
                            input,
                            is_synthetic,
                        } => (),
                        // Handle keyboard modifies.
                        WindowEvent::ModifiersChanged(_) => (),

                        // Handle cursor events.
                        WindowEvent::CursorMoved { position, .. } => {
                            let pos = (position.x as u32, position.y as u32);
                            cursor_pos = Some(pos);
                            if let Some(d) = &mut drag {
                                d.update_cursor_end(pos);
                                if d.past_threshold {
                                    camera.drag(*d);
                                }
                            }
                        }
                        WindowEvent::CursorLeft { .. } => cursor_pos = None,

                        // Handle mouse wheel.
                        WindowEvent::MouseWheel { delta, .. } => (),

                        // Handle mouse click.
                        WindowEvent::MouseInput { state, button, .. } => {
                            if let Some(pixel) = cursor_pos {
                                match state {
                                    ElementState::Pressed => {
                                        if drag.is_none() {
                                            let drag_kind = match button {
                                                MouseButton::Left | MouseButton::Right => {
                                                    Some(DragKind::Pan)
                                                }
                                                MouseButton::Middle => Some(DragKind::Scale),
                                                _ => None,
                                            };
                                            if let Some(kind) = drag_kind {
                                                drag = Some(Drag {
                                                    tile_coords: camera.pixel_to_tile_coords(pixel),
                                                    initial_scale_factor: camera.scale().factor(),

                                                    cursor_start: pixel,
                                                    cursor_end: pixel,
                                                    past_threshold: false,

                                                    kind,
                                                });
                                            }
                                        }
                                    }
                                    ElementState::Released => {
                                        let tile_pos = camera.pixel_to_tile_pos(pixel);
                                        if let Some(d) = drag {
                                            drag = None;
                                        } else {
                                            match button {
                                                MouseButton::Left => {
                                                    grid.set_tile(tile_pos, Tile::Number(0));
                                                }
                                                MouseButton::Right => match grid.get_tile(tile_pos)
                                                {
                                                    Tile::Covered(FlagState::None, h) => grid
                                                        .set_tile(
                                                            tile_pos,
                                                            Tile::Covered(FlagState::Flag, h),
                                                        ),
                                                    Tile::Covered(FlagState::Flag, h) => grid
                                                        .set_tile(
                                                            tile_pos,
                                                            Tile::Covered(FlagState::Question, h),
                                                        ),
                                                    Tile::Covered(FlagState::Question, h) => grid
                                                        .set_tile(
                                                            tile_pos,
                                                            Tile::Covered(FlagState::None, h),
                                                        ),
                                                    _ => (),
                                                },
                                                MouseButton::Middle => todo!(),
                                                MouseButton::Other(_) => todo!(),
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        _ => (),
                    },
                    _ => (),
                }
            }

            // Draw everything.
            let mut target = display.draw();
            render::draw_grid(&mut target, &grid, &mut camera);
            target.finish().expect("Failed to swap buffers");
        }
    })
}
