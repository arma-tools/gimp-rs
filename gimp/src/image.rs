use glib::translate::*;

use crate::{ffi, Image, Metadata, MetadataSaveFlags};

impl Image {
    /// Prepares the image's metadata for exporting it in `mime_type`.
    ///
    /// Returns the metadata and `suggested_flags` adjusted to the image's
    /// metadata and the user's preferences.
    #[doc(alias = "gimp_image_metadata_save_prepare")]
    pub fn metadata_save_prepare(
        &self,
        mime_type: &str,
        suggested_flags: MetadataSaveFlags,
    ) -> (Option<Metadata>, MetadataSaveFlags) {
        unsafe {
            let mut flags = suggested_flags.into_glib();
            let metadata = from_glib_full(ffi::gimp_image_metadata_save_prepare(
                self.to_glib_none().0,
                mime_type.to_glib_none().0,
                &mut flags,
            ));
            (metadata, from_glib(flags))
        }
    }
}
