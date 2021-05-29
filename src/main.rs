//! Infinite Minesweeper with a variety of other features.

#![allow(dead_code)] // TODO: remove this line
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(clippy::all)]
#![deny(clippy::correctness)]

mod grid;
mod gui;

const TITLE: &str = "HMines Infinite";

fn main() {
    gui::show_gui();
}
