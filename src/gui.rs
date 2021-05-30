use glium::glutin::event::{Event, StartCause, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::glutin::window::WindowBuilder;
use glium::glutin::ContextBuilder;
use lazy_static::lazy_static;
use send_wrapper::SendWrapper;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::game::Game;
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
    let mut game = Game::load_from_file();
    let mut events_buffer = VecDeque::new();

    // Main loop.
    let mut last_frame_time = Instant::now();
    let mut next_frame_time = Instant::now();
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
            Some(Event::LoopDestroyed) =>
                game.save_to_file()
            ,

            // Queue the event to be handled next time we render
            // everything.
            Some(ev) => events_buffer.push_back(ev),

            // Ignore this event.
            None => (),
        }

        if do_frame && next_frame_time <= now {
            let frame_duration = Duration::from_secs_f64(1.0 / 60.0);

            next_frame_time = now + frame_duration;
            if next_frame_time < Instant::now() {
                // Skip a frame (or several).
                next_frame_time = Instant::now() + frame_duration;
            }
            *control_flow = ControlFlow::WaitUntil(next_frame_time);

            let frame_duration = now
                .checked_duration_since(last_frame_time)
                .unwrap_or(frame_duration);
            // TODO: give `frame_duration` to egui if egui wants it
            last_frame_time = now;

            for ev in events_buffer.drain(..) {
                // Handle events.
                match ev {
                    Event::WindowEvent { event, .. } => match event {
                        // Handle window close event.
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,

                        // Let the game handle any other event.
                        ev => game.handle_event(ev),
                    },
                    _ => (),
                }
            }

            game.do_frame(frame_duration);

            // Draw everything.
            let mut target = display.draw();
            render::draw_grid(&mut target, &game.grid, &mut game.camera);
            target.finish().expect("Failed to swap buffers");
        }
    })
}
