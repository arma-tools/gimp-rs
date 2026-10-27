#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

pub use ffi;
pub use gimp;
pub use gtk;

#[allow(clippy::too_many_arguments)]
#[allow(clippy::let_unit_value)]
#[allow(clippy::type_complexity)]
#[allow(clippy::let_and_return)]
#[allow(unused_imports)]
mod auto;
pub use auto::functions::*;
pub use auto::*;

pub mod prelude;
