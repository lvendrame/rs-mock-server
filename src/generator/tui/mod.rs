//! Ratatui terminal wizard for the generator feature.

use std::io;

mod app;
mod components;
mod render;
mod terminal;

/// Runs the interactive generator wizard.
pub fn run_generator(folder: &str) -> io::Result<()> {
    terminal::run(folder)
}
