#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

pub use cairo;
pub use ffi;
pub use gdk;
pub use gdk_pixbuf;
pub use gegl;
pub use gimp;
pub use gio;
pub use glib;
pub use gtk;
pub use pango;

#[allow(clippy::too_many_arguments)]
#[allow(clippy::doc_lazy_continuation)]
#[allow(clippy::let_unit_value)]
#[allow(clippy::type_complexity)]
#[allow(clippy::let_and_return)]
#[allow(unused_imports)]
mod auto;
pub use auto::functions::*;
pub use auto::*;

pub mod prelude;
