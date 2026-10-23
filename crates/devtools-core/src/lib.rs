//! Catalog types shared by the shell and the pure tool crate.
//!
//! This crate must not depend on GPUI. Tool behavior that can be tested
//! without a window lives in `devtools-tools`.

#![forbid(unsafe_code)]

mod category;
mod registry;
mod tool;

pub use category::Category;
pub use registry::Registry;
pub use tool::{Availability, Confidence, Detection, NetworkUse, ToolMeta, ids};
