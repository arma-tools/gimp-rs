#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

pub use ffi;
pub use gio;
pub use glib;

#[allow(clippy::too_many_arguments)]
#[allow(clippy::doc_lazy_continuation)]
#[allow(clippy::let_unit_value)]
#[allow(clippy::wrong_self_convention)]
mod auto;
pub use auto::*;

pub mod prelude;
