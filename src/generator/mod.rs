//! Interactive mock file and configuration generator.

pub mod content;
pub mod domain;
pub mod main_config;
pub mod paths;
pub mod tui;
pub mod writer;

pub use tui::run_generator;
