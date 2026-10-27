//! Traits for subclassing GIMP types, most importantly [`PlugIn`](crate::PlugIn).

mod plug_in;

pub use plug_in::I18n;

pub mod prelude {
    #[doc(hidden)]
    pub use gio::subclass::prelude::*;

    pub use super::plug_in::{I18n, PlugInImpl, PlugInImplExt};
}
