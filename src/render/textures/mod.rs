use glium::texture::{MipmapsOption, RawImage2d, SrgbFormat, SrgbTexture2d};
use lazy_static::lazy_static;
use send_wrapper::SendWrapper;

macro_rules! load_mipmapped_sprites {
    ($filename_prefix:expr) => {{
        let raw_img_64 = include_bytes!(concat!($filename_prefix, "_64.png"));
        let raw_img_32 = include_bytes!(concat!($filename_prefix, "_32.png"));
        let raw_img_16 = include_bytes!(concat!($filename_prefix, "_16.png"));
        let raw_img_8 = include_bytes!(concat!($filename_prefix, "_8.png"));

        let t = SrgbTexture2d::with_mipmaps(
            &**crate::DISPLAY,
            load_rgba_image(raw_img_64),
            MipmapsOption::EmptyMipmapsMax(3),
        )
        .expect("Failed to create texture");

        write_tex_mipmap(&t, 1, load_rgba_image(raw_img_32));
        write_tex_mipmap(&t, 2, load_rgba_image(raw_img_16));
        write_tex_mipmap(&t, 3, load_rgba_image(raw_img_8));

        SendWrapper::new(t)
    }};
}

fn write_tex_mipmap(t: &SrgbTexture2d, level: u32, image: RawImage2d<'_, u8>) {
    let mipmap_level = t.mipmap(level).expect("Missing mipmap level");
    let (width, height) = mipmap_level.dimensions();
    let rect = glium::Rect {
        left: 0,
        bottom: 0,
        width,
        height,
    };
    mipmap_level.write(rect, image);
}

fn load_rgba_image(image_bytes: &[u8]) -> RawImage2d<'_, u8> {
    let image = image::load_from_memory(image_bytes)
        .expect("Failed to load image data")
        .to_rgba8();
    let dimensions = image.dimensions();
    RawImage2d::from_raw_rgba_reversed(&image.into_raw(), dimensions)
}

lazy_static! {
    pub static ref COVERED: SendWrapper<SrgbTexture2d> = load_mipmapped_sprites!("covered");
    pub static ref UNCOVERED: SendWrapper<SrgbTexture2d> = load_mipmapped_sprites!("uncovered");
    pub static ref OVERLAY: SendWrapper<SrgbTexture2d> = load_mipmapped_sprites!("overlay");
}
