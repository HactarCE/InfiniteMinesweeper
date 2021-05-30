use glium::index::PrimitiveType;
use glium::uniforms::MinifySamplerFilter;
use glium::{Frame, IndexBuffer, Program, Surface, VertexBuffer};
use lazy_static::lazy_static;
use send_wrapper::SendWrapper;

mod shaders;
mod textures;

use crate::grid::{Camera, Grid, Tile};

const TILE_BATCH_SIZE: usize = 32;

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
}

pub fn draw_grid(target: &mut glium::Frame, grid: &Grid, camera: &mut Camera) {
    // Update target dimensisons and get camera data.
    camera.set_target_dimensions(target.get_dimensions());
    let tile_transform_matrix: [[f32; 4]; 4] = camera.gl_matrix().into();

    let draw_params = glium::DrawParameters {
        blend: glium::Blend::alpha_blending(),
        ..glium::DrawParameters::default()
    };

    let mut tiles = vec![];
    for y in -10..10 {
        for x in -10..10 {
            tiles.push(TileAttr {
                tile_coords: [x, y],
                sprite_coords: [if x < 0 { 1 } else { 0 }, 2],
            });

            if y == 0 && 0 <= x && x < 8 {
                tiles.push(TileAttr {
                    tile_coords: [x, y],
                    sprite_coords: [x as u32, 0],
                });
            } else if (x + 2 * y) % 4 == 0 {
                tiles.push(TileAttr {
                    tile_coords: [x, y],
                    sprite_coords: [y as u32 % 2 + if x < 0 { 0 } else { 2 }, 1],
                });
            }
        }
    }

    let uniform = glium::uniform! {
        spritesheet: **textures::TILES_SPRITESHEET_SAMPLER,

        camera_center: camera.int_center(),
        transform: tile_transform_matrix,
    };
    for batch in tiles.chunks(TILE_BATCH_SIZE) {
        let instances_slice = TILE_INSTANCES_VBO.slice(0..batch.len()).unwrap();
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
