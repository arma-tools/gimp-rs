use std::ops::Deref;

use glib::translate::*;

use crate::{ffi, ExportOptions, ExportReturn, Image};

/// The image to export, returned by [`ExportOptions::image`].
///
/// Dereferences to the image. When GIMP had to create a converted copy
/// ([`ExportReturn::Export`]), the copy is deleted on drop.
#[derive(Debug)]
pub struct ExportImage {
    image: Image,
    export_return: ExportReturn,
}

impl ExportImage {
    /// Whether the image is the original one ([`ExportReturn::Ignore`]),
    /// which must not be modified, or a converted copy
    /// ([`ExportReturn::Export`]).
    pub fn export_return(&self) -> ExportReturn {
        self.export_return
    }

    /// Takes the image out of the guard, the caller is responsible to
    /// [`delete`](Image::delete) a copy, e.g. after displaying it.
    pub fn into_inner(self) -> (Image, ExportReturn) {
        let this = std::mem::ManuallyDrop::new(self);
        // SAFETY: `this` is never dropped, so `image` is moved out only once.
        (unsafe { std::ptr::read(&this.image) }, this.export_return)
    }
}

impl Deref for ExportImage {
    type Target = Image;

    fn deref(&self) -> &Image {
        &self.image
    }
}

impl Drop for ExportImage {
    fn drop(&mut self) {
        if self.export_return == ExportReturn::Export {
            self.image.delete();
        }
    }
}

impl ExportOptions {
    /// Returns `image` converted according to the export options, for example
    /// flattened when the format cannot store layers. See [`ExportImage`].
    #[doc(alias = "gimp_export_options_get_image")]
    #[doc(alias = "get_image")]
    pub fn image(&self, image: &Image) -> ExportImage {
        unsafe {
            let mut image_ptr: *mut ffi::GimpImage = image.to_glib_none().0;
            let export_return: ExportReturn = from_glib(ffi::gimp_export_options_get_image(
                self.to_glib_none().0,
                &mut image_ptr,
            ));
            ExportImage {
                image: from_glib_none(image_ptr),
                export_return,
            }
        }
    }
}
