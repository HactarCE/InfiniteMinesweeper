use glium::program;
use glium::Program;
use lazy_static::lazy_static;
use send_wrapper::SendWrapper;

lazy_static! {
    pub static ref RGBA_PROGRAM: SendWrapper<Program> = SendWrapper::new(
        glium::program!(
            &**crate::DISPLAY,
            140 => {
                vertex: include_str!("rgba.vert"),
                fragment: include_str!("rgba.frag"),
                outputs_srgb: false,
            },
        )
        .expect("Failed to compile shader")
    );
}
