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
#[cfg(feature = "desktop")]
pub mod object_layout;
#[cfg(feature = "desktop")]
pub mod page_layout;
pub mod pagination;
#[cfg(feature = "desktop")]
pub mod pdf_output;
pub mod preferences;
pub mod presentation;
pub mod printing;
pub mod ruler;
pub mod search;
pub mod session;
pub mod storage;
pub mod structure;
mod structure_edit;
pub mod style_filter;
pub mod tab_ruler;
pub mod tab_sources;

pub type Result<T> = std::result::Result<T, String>;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
