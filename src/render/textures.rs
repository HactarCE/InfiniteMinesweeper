use glium::texture::{MipmapsOption, RawImage2d, SrgbTexture2d};
use glium::uniforms::{MinifySamplerFilter, Sampler};
use lazy_static::lazy_static;
use send_wrapper::SendWrapper;

use crate::game::{FlagState, Tile};

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
        let raw_img_64 = include_bytes!("../../resources/tilemaps/tiles_64.png");
        let raw_img_32 = include_bytes!("../../resources/tilemaps/tiles_32.png");
        let raw_img_16 = include_bytes!("../../resources/tilemaps/tiles_16.png");
        let raw_img_8 = include_bytes!("../../resources/tilemaps/tiles_8.png");
        let raw_img_4 = include_bytes!("../../resources/tilemaps/tiles_4.png");
        let raw_img_2 = include_bytes!("../../resources/tilemaps/tiles_2.png");

        let t = SrgbTexture2d::with_mipmaps(
            &**crate::DISPLAY,
            load_rgba_image(raw_img_64),
            MipmapsOption::EmptyMipmapsMax(5),
        )
        .expect("Failed to create texture");

        write_tex_mipmap(&t, 1, load_rgba_image(raw_img_32));
        write_tex_mipmap(&t, 2, load_rgba_image(raw_img_16));
        write_tex_mipmap(&t, 3, load_rgba_image(raw_img_8));
        write_tex_mipmap(&t, 4, load_rgba_image(raw_img_4));
        write_tex_mipmap(&t, 5, load_rgba_image(raw_img_2));

        SendWrapper::new(t)
    };

    /// Mipmapped texture sampler for the tiles spritesheet.
    pub static ref TILES_SPRITESHEET_SAMPLER: SendWrapper<Sampler<'static, SrgbTexture2d>> =
        SendWrapper::new(TILES_SPRITESHEET_TEX
            .sampled()
            .minify_filter(MinifySamplerFilter::NearestMipmapNearest));
}

pub fn bg_sprite_coords(tile: Tile) -> [u32; 2] {
    match tile {
        Tile::Covered(_, _) => [1, 2],
        Tile::Number(_) | Tile::Mine => [0, 2],
    }
}
pub fn fg_sprite_coords(tile: Tile) -> Option<[u32; 2]> {
    match tile {
        Tile::Covered(f, _) => match f {
            FlagState::None => None,
            FlagState::Flag => Some([0, 1]),
            FlagState::Question => Some([1, 1]),
        },
        Tile::Number(0) => None,
        Tile::Number(i) => Some([i as u32 - 1, 0]),
        Tile::Mine => Some([2, 1]),
    }
}
