use glium::{Surface, VertexBuffer};
use lazy_static::lazy_static;
use send_wrapper::SendWrapper;

mod shaders;
mod textures;

use crate::game::{Camera, ChunkPos, Grid, Tile, TilePos, CHUNK_SIZE};

const TILE_BATCH_SIZE: usize = 4096;

#[derive(Debug, Copy, Clone)]
struct Vertex2D {
    pos: [f32; 2],
}
glium::implement_vertex!(Vertex2D, pos);

#[derive(Debug, Copy, Clone)]
struct TileAttr {
    tile_coords: [i32; 2],
    sprite_coords: [u32; 2],
}
glium::implement_vertex!(TileAttr, tile_coords, sprite_coords);
impl TileAttr {
    fn new(tile_coords: [i32; 2], sprite_coords: [u32; 2]) -> Self {
        Self {
            tile_coords,
            sprite_coords,
        }
    }
}

lazy_static! {
    static ref SQUARE_VBO: SendWrapper<VertexBuffer<Vertex2D>> = SendWrapper::new(
        VertexBuffer::immutable(
            &**crate::DISPLAY,
            &[
                Vertex2D { pos: [0.0, 0.0] },
                Vertex2D { pos: [1.0, 0.0] },
                Vertex2D { pos: [0.0, 1.0] },
                Vertex2D { pos: [1.0, 1.0] },
            ]
        )
        .expect("Failed to create vertex buffer")
    );
    static ref TILE_INSTANCES_VBO: SendWrapper<VertexBuffer<TileAttr>> = SendWrapper::new(
        VertexBuffer::empty_dynamic(&**crate::DISPLAY, TILE_BATCH_SIZE)
            .expect("Failed to create vertex buffer")
    );
    static ref TILE_INSTANCES_OVERFLOW_VBO: SendWrapper<VertexBuffer<TileAttr>> = SendWrapper::new(
        VertexBuffer::empty_dynamic(&**crate::DISPLAY, TILE_BATCH_SIZE)
            .expect("Failed to create vertex buffer")
    );
}

pub fn draw_grid(target: &mut glium::Frame, grid: &Grid, camera: &mut Camera) {
    target.clear_color_srgb(0.2, 0.2, 0.2, 1.0);

    // Update target dimensisons and get camera data.
    camera.set_target_dimensions(target.get_dimensions());
    let tile_transform_matrix: [[f32; 4]; 4] = camera.gl_matrix().into();

    let draw_params = glium::DrawParameters {
        blend: glium::Blend::alpha_blending(),
        ..glium::DrawParameters::default()
    };

    let (target_w, target_h) = target.get_dimensions();
    let TilePos(mut x1, mut y1) = camera.pixel_to_tile_pos((0, target_h));
    x1 -= 1;
    y1 -= 1;
    let TilePos(mut x2, mut y2) = camera.pixel_to_tile_pos((target_w, 0));
    x2 += 1;
    y2 += 1;

    let ChunkPos(chunk_x1, chunk_y1) = TilePos(x1, y1).chunk();
    let ChunkPos(chunk_x2, chunk_y2) = TilePos(x2, y2).chunk();

    let mut tile_attrs = vec![];

    for chunk_y in chunk_y1..=chunk_y2 {
        for chunk_x in chunk_x1..=chunk_x2 {
            let chunk = grid.get_chunk(ChunkPos(chunk_x, chunk_y));
            for y in 0..CHUNK_SIZE as i32 {
                for x in 0..CHUNK_SIZE as i32 {
                    let tile_coords = [
                        x + chunk_x * CHUNK_SIZE as i32,
                        y + chunk_y * CHUNK_SIZE as i32,
                    ];
                    let tile = match chunk {
                        Some(c) => c.get_tile(TilePos(x, y)),
                        None => Tile::default(),
                    };
                    let bg_sprite_coords = textures::bg_sprite_coords(tile);
                    tile_attrs.push(TileAttr::new(tile_coords, bg_sprite_coords));
                    if let Some(fg_sprite_coords) = textures::fg_sprite_coords(tile) {
                        tile_attrs.push(TileAttr::new(tile_coords, fg_sprite_coords));
                    }
                }
            }
        }
    }

    let uniform = glium::uniform! {
        spritesheet: **textures::TILES_SPRITESHEET_SAMPLER,

        camera_center: camera.int_center(),
        transform: tile_transform_matrix,
    };
    for batch in tile_attrs.chunks(TILE_BATCH_SIZE) {
        let instances_slice = if batch.len() == TILE_BATCH_SIZE {
            &**TILE_INSTANCES_VBO
        } else {
            // For some bizarre reason, writing to only a portion of a VBO used
            // for instanced rendering messes up *previous* draw calls using
            // that same VBO. So we have to use the "overflow" VBO for the last
            // batch.
            &**TILE_INSTANCES_OVERFLOW_VBO
        }
        .slice(0..batch.len())
        .unwrap();

        instances_slice.write(batch);

        target
            .draw(
                (&**SQUARE_VBO, instances_slice.per_instance().unwrap()),
                &glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip),
                &shaders::SPRITESHEET_PROGRAM,
                &uniform,
                &draw_params,
            )
            .expect("Failed to draw tiles");
    }
}
