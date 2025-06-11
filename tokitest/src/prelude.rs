//! Prelude module for tokitest
//!
//! Import this module to get all the essential traits and types:
//! ```rust
//! use tokitest::prelude::*;
//! ```

// Re-export all traits that your macros need
pub use crate::controller::Nestable;
pub use crate::label_spec::LabelTrait;
