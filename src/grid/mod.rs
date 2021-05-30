mod camera;
mod scale;
mod tile;

pub use camera::Camera;
pub use scale::Scale;
pub use tile::*;

use std::collections::HashMap;

pub const CHUNK_SIZE_LOG_2: usize = 6;
pub const CHUNK_SIZE: usize = 2_usize.pow(CHUNK_SIZE_LOG_2 as u32);

#[derive(Debug, Default, Clone)]
pub struct Grid(HashMap<ChunkPos, Chunk>);
impl Grid {
    /// Returns a new empty grid.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a chunk of the grid, or `None` if the chunk is missing.
    pub fn get_chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.0.get(&pos)
    }
    /// Returns a chunk of the grid mutably, filling it with a default if it is
    /// missing.
    pub fn get_chunk_mut(&mut self, pos: ChunkPos) -> &mut Chunk {
        self.0.entry(pos).or_insert_with(Chunk::default)
    }
    /// Returns a tile in the grid.
    pub fn get_tile(&self, pos: TilePos) -> Tile {
        match self.get_chunk(pos.chunk()) {
            Some(chunk) => chunk.get_tile(pos),
            None => Tile::default(),
        }
    }
    /// Sets a tile in the grid.
    pub fn set_tile(&mut self, pos: TilePos, tile: Tile) {
        self.get_chunk_mut(pos.chunk()).set_tile(pos, tile);
    }
}

/// Tile coordinates.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TilePos(pub i32, pub i32);
impl TilePos {
    /// Returns the position of the chunk containing the tile position.
    pub fn chunk(self) -> ChunkPos {
        let TilePos(x, y) = self;
        ChunkPos(x >> CHUNK_SIZE_LOG_2, y >> CHUNK_SIZE_LOG_2)
    }
}

/// Global coordinates of a chunk.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ChunkPos(pub i32, pub i32);

/// Square chunk of tiles.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Chunk([PackedTile; CHUNK_SIZE * CHUNK_SIZE]);
impl Default for Chunk {
    fn default() -> Self {
        // const generics whennnnnn
        Self([PackedTile::default(); CHUNK_SIZE * CHUNK_SIZE])
    }
}
impl Chunk {
    /// Returns the index of a tile position in its chunk.
    fn index_of_tile(TilePos(x, y): TilePos) -> usize {
        let x = x & (CHUNK_SIZE as i32 - 1);
        let y = y & (CHUNK_SIZE as i32 - 1);
        (y as usize) << CHUNK_SIZE_LOG_2 | x as usize
    }

    /// Returns a tile in the chunk.
    pub fn get_tile(&self, pos: TilePos) -> Tile {
        self.0[Self::index_of_tile(pos)].unpack()
    }
    /// Sets a tile in the chunk.
    pub fn set_tile(&mut self, pos: TilePos, tile: Tile) {
        self.0[Self::index_of_tile(pos)] = tile.pack();
    }
}
