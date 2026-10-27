#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

pub use babl;
pub use ffi;
pub use gegl;
pub use gexiv2;
pub use gio;
pub use glib;

#[allow(clippy::too_many_arguments)]
#[allow(clippy::let_unit_value)]
#[allow(clippy::type_complexity)]
#[allow(clippy::let_and_return)]
#[allow(unused_imports)]
mod auto;
pub use auto::functions::*;
pub use auto::*;

mod drawable;
mod export_options;
mod gegl_buffer;
mod image;
mod plug_in_main;
mod procedure;
mod value_array;

pub use export_options::ExportImage;
pub use plug_in_main::main;
pub use procedure::PlugInError;

pub mod prelude;
pub mod subclass;
