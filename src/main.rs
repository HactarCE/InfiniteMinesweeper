//! Infinite Minesweeper with a variety of other features.

#![allow(dead_code)] // TODO: remove this line
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(clippy::all)]
#![deny(clippy::correctness)]

mod game;
mod gui;
mod render;

use gui::DISPLAY;

const TITLE: &str = "Infinite Minesweeper";

fn main() {
    gui::show_gui();
}
