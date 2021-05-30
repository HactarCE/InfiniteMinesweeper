use glium::texture::{MipmapsOption, RawImage2d, SrgbTexture2d};
use glium::uniforms::{MinifySamplerFilter, Sampler};
use lazy_static::lazy_static;
use send_wrapper::SendWrapper;

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
    /// Mipmapped spritesheet texture for tiles.
    static ref TILES_SPRITESHEET_TEX: SendWrapper<SrgbTexture2d> = {
        let raw_img_64 = include_bytes!("tiles_64.png");
        let raw_img_32 = include_bytes!("tiles_32.png");
        let raw_img_16 = include_bytes!("tiles_16.png");
        let raw_img_8 = include_bytes!("tiles_8.png");

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
    };

    /// Mipmapped texture sampler for the tiles spritesheet.
    pub static ref TILES_SPRITESHEET_SAMPLER: SendWrapper<Sampler<'static, SrgbTexture2d>> =
        SendWrapper::new(TILES_SPRITESHEET_TEX
            .sampled()
            .minify_filter(MinifySamplerFilter::NearestMipmapNearest));
}
