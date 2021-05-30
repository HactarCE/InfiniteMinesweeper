/// Tile in the Minesweeper grid, packed into a single byte.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(super) struct PackedTile(pub(super) u8);
impl Default for PackedTile {
    fn default() -> Self {
        Tile::default().pack()
    }
}
impl PackedTile {
    /// Unpacks the `Tile` from a single byte.
    pub(super) fn unpack(self) -> Tile {
        if self.0 == '!' as u8 {
            Tile::Mine
        } else if self.0 == ' ' as u8 {
            Tile::Number(0)
        } else if self.0 <= '9' as u8 {
            Tile::Number(self.0 - '0' as u8)
        } else if self.0 < 0x60 {
            Tile::Number(self.0 - 'A' as u8 + 10)
        } else {
            Tile::Covered(
                FlagState::from((self.0 >> 2) & 0b11),
                HiddenState::from(self.0 & 0b11),
            )
        }
    }
}

/// Tile in the Minesweeper grid.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Tile {
    /// Covered tile.
    Covered(FlagState, HiddenState),
    /// Revealed safe tile.
    Number(u8),
    /// Revealed mine tile.
    Mine,
}
impl Default for Tile {
    fn default() -> Self {
        Tile::Covered(FlagState::default(), HiddenState::default())
    }
}
impl Tile {
    /// Packs the tile into a single byte.
    pub(super) fn pack(self) -> PackedTile {
        match self {
            Tile::Covered(f, h) => PackedTile(0x60 | (f as u8) << 2 | h as u8),
            Tile::Number(0) => PackedTile(' ' as u8),
            Tile::Number(n) if n < 10 => PackedTile(n + '0' as u8),
            Tile::Number(n) => PackedTile(n - 10 + 'A' as u8),
            Tile::Mine => PackedTile('!' as u8),
        }
    }

    /// Toggles flag on the tile.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn toggle_flag(self) -> Tile {
        match self {
            Tile::Covered(f, h) => {
                let new_f = match f {
                    FlagState::None => FlagState::Flag,
                    FlagState::Flag => FlagState::None,
                    FlagState::Question => FlagState::None,
                };
                Tile::Covered(new_f, h)
            }
            _ => self,
        }
    }

    /// Returns `true` if the tile is a mine or `false` if it might not be.
    pub fn is_mine(self) -> bool {
        match self {
            Tile::Covered(_, HiddenState::Mine) => true,
            Tile::Mine => true,
            _ => false,
        }
    }
    /// Returns `true` if the tile is a flag or a revealed mine.
    pub fn is_assumed_mine(self) -> bool {
        match self {
            Tile::Covered(FlagState::Flag, _) => true,
            Tile::Mine => true,
            _ => false,
        }
    }
}

/// Flag or question mark annotation added by the player.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum FlagState {
    /// No player annotation.
    None = 0,
    /// Flag annotation.
    Flag = 1,
    /// Question mark annotation.
    Question = 2,
}
impl Default for FlagState {
    fn default() -> Self {
        FlagState::None
    }
}
impl From<u8> for FlagState {
    fn from(x: u8) -> Self {
        match x & 0b11 {
            0 => FlagState::None,
            1 => FlagState::Flag,
            2 => FlagState::Question,
            _ => panic!("Invalid FlagState"),
        }
    }
}

/// Underlying state hidden from the player.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum HiddenState {
    /// Possibly a mine, depending on hidden information.
    Unknown = 0,
    /// Definitely safe, based on information revealed to the player.
    Safe = 1,
    /// Definitely a mine, based on information revealed to the player.
    Mine = 2,
}
impl Default for HiddenState {
    fn default() -> Self {
        HiddenState::Unknown
    }
}
impl From<u8> for HiddenState {
    fn from(x: u8) -> Self {
        match x & 0b11 {
            0 => HiddenState::Unknown,
            1 => HiddenState::Safe,
            2 => HiddenState::Mine,
            _ => panic!("Invalid HiddenState"),
        }
    }
}

#[cfg(test)]
#[test]
fn test_packed_tile() {
    let tiles: &[Tile] = &[
        Tile::Mine,
        Tile::Covered(FlagState::None, HiddenState::Unknown),
        Tile::Covered(FlagState::None, HiddenState::Safe),
        Tile::Covered(FlagState::None, HiddenState::Mine),
        Tile::Covered(FlagState::Flag, HiddenState::Unknown),
        Tile::Covered(FlagState::Flag, HiddenState::Safe),
        Tile::Covered(FlagState::Flag, HiddenState::Mine),
        Tile::Covered(FlagState::Question, HiddenState::Unknown),
        Tile::Covered(FlagState::Question, HiddenState::Safe),
        Tile::Covered(FlagState::Question, HiddenState::Mine),
    ];
    for &t in tiles {
        assert_eq!(t, t.pack().unpack());
    }

    for n in 0..32 {
        let t = Tile::Number(n);
        assert_eq!(t, t.pack().unpack());
    }
}
