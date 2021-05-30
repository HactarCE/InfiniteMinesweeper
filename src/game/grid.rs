use std::collections::HashMap;

use itertools::Itertools;
use rand::Rng;

use super::tile::{FlagState, HiddenState, PackedTile, Tile};
use super::MINE_DENSITY;

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

    /// Places mines in unknown squares within a chunk.
    pub fn place_mines_in_chunk(&mut self, pos: ChunkPos) {
        // TODO: use a deterministic RNG, seeded using the game seed + chunk pos
        let mut rng = rand::thread_rng();
        let chunk = self.get_chunk_mut(pos);
        if chunk.all_mines_placed {
            return;
        }
        for tile in &mut chunk.tiles {
            if let Tile::Covered(f, h) = tile.unpack() {
                if h == HiddenState::Unknown {
                    let h = if rng.gen_bool(MINE_DENSITY) {
                        HiddenState::Mine
                    } else {
                        HiddenState::Safe
                    };
                    *tile = Tile::Covered(f, h).pack();
                }
            }
        }
        chunk.all_mines_placed = true;
    }

    /// Toggles flag on a tile in the grid.
    pub fn toggle_flag(&mut self, pos: TilePos) {
        self.set_tile(pos, self.get_tile(pos).toggle_flag());
    }

    /// Reveals a square.
    pub fn reveal(&mut self, pos: TilePos) {
        match self.get_tile(pos) {
            Tile::Covered(_, _) => self.reveal_hidden(pos),
            Tile::Number(_) => self.reveal_adjacent_safely(pos),
            Tile::Mine => (),
        }
    }
    /// Reveals a hidden tile in the grid.
    pub fn reveal_hidden(&mut self, pos: TilePos) {
        self.place_mines_in_chunk(pos.chunk());

        match self.get_tile(pos) {
            Tile::Covered(FlagState::None, h) | Tile::Covered(FlagState::Question, h) => match h {
                HiddenState::Unknown => panic!("expected all mines to be placed"),
                HiddenState::Safe => {
                    let n = self.count_neighbors(pos, Tile::is_mine);
                    self.set_tile(pos, Tile::Number(n));
                    if n == 0 {
                        for nbr in pos.neighbors() {
                            self.reveal_hidden(nbr);
                        }
                    }
                }
                HiddenState::Mine => {
                    self.set_tile(pos, Tile::Mine);
                }
            },
            _ => (),
        }
    }
    /// Reveals hidden tiles adjacent to a known one, if the correct number of
    /// flags have been placed nearby.
    pub fn reveal_adjacent_safely(&mut self, pos: TilePos) {
        match self.get_tile(pos) {
            Tile::Number(n) => {
                let n_flags = self.count_neighbors(pos, Tile::is_assumed_mine);
                if n_flags == n {
                    for nbr in pos.neighbors() {
                        self.reveal_hidden(nbr);
                    }
                }
            }
            _ => (),
        }
    }

    /// Returns the number of neighboring tiles that satisfy a predicate,
    /// populating chunks with mines as needed.
    fn count_neighbors(&mut self, pos: TilePos, mut predicate: impl FnMut(Tile) -> bool) -> u8 {
        pos.neighbors()
            .filter(|&p| {
                self.place_mines_in_chunk(p.chunk());
                predicate(self.get_tile(p))
            })
            .count() as u8
    }
}

/// Square chunk of tiles.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Chunk {
    tiles: [PackedTile; CHUNK_SIZE * CHUNK_SIZE],
    all_mines_placed: bool,
}
impl Default for Chunk {
    fn default() -> Self {
        Self {
            tiles: [PackedTile::default(); CHUNK_SIZE * CHUNK_SIZE],
            all_mines_placed: false,
        }
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
        self.tiles[Self::index_of_tile(pos)].unpack()
    }
    /// Sets a tile in the chunk.
    pub fn set_tile(&mut self, pos: TilePos, tile: Tile) {
        self.tiles[Self::index_of_tile(pos)] = tile.pack();
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
    /// Returns an iterator over neighboring positions.
    pub fn neighbors(self) -> impl Iterator<Item = Self> {
        (-1..=1)
            .cartesian_product(-1..=1)
            .map(move |(dx, dy)| TilePos(self.0 + dx, self.1 + dy))
    }
}

/// Global coordinates of a chunk.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ChunkPos(pub i32, pub i32);
