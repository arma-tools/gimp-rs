use babl::ObjectType;
use glib::{prelude::*, translate::*};

use crate::{ffi, Drawable};

pub trait DrawableExtManual: IsA<Drawable> + 'static {
    /// The babl format of the drawable's pixels, in the image's color space.
    ///
    /// Use its [`space`](babl::Format::space) with
    /// [`babl::Format::format_with_space`] to read or write pixels in another
    /// encoding without changing the color space.
    #[doc(alias = "gimp_drawable_get_format")]
    #[doc(alias = "get_format")]
    fn format(&self) -> babl::Format {
        unsafe {
            babl::Format::from_raw_full(ffi::gimp_drawable_get_format(
                self.as_ref().to_glib_none().0,
            ))
        }
    }

    /// The babl format of thumbnails of the drawable (8 bit, perceptual).
    #[doc(alias = "gimp_drawable_get_thumbnail_format")]
    #[doc(alias = "get_thumbnail_format")]
    fn thumbnail_format(&self) -> babl::Format {
        unsafe {
            babl::Format::from_raw_full(ffi::gimp_drawable_get_thumbnail_format(
                self.as_ref().to_glib_none().0,
            ))
        }
    }
}

impl<O: IsA<Drawable>> DrawableExtManual for O {}
