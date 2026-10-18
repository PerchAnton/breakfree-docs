//! Independent document engine. Desktop dependencies are feature gated.
#[cfg(feature = "desktop")]
mod chrome;
pub mod commands;
pub mod formatting;
#[cfg(feature = "desktop")]
pub mod gui;
pub mod integration;
#[cfg(feature = "desktop")]
pub mod layout;
pub mod model;
pub mod preferences;
pub mod presentation;
pub mod search;
pub mod session;
pub mod storage;
pub mod style_filter;

pub type Result<T> = std::result::Result<T, String>;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
