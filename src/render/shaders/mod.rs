use glium::program;
use glium::Program;
use lazy_static::lazy_static;
use send_wrapper::SendWrapper;

lazy_static! {
    pub static ref SPRITESHEET_PROGRAM: SendWrapper<Program> = SendWrapper::new(
        glium::program!(
            &**crate::DISPLAY,
            140 => {
                vertex: include_str!("sprite.vert"),
                fragment: include_str!("sprite.frag"),
                outputs_srgb: false,
            },
        )
        .expect("Failed to compile shader")
    );
}
