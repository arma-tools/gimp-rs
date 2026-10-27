//! Pixel access for [`gegl::Buffer`] with babl formats.
//!
//! The `gegl` crate only binds the introspectable variants of
//! `gegl_buffer_get()`/`gegl_buffer_set()`, which take format names and so
//! cannot express the color space of a drawable
//! (`babl_format_with_space()`).

use babl::ObjectType;
use glib::translate::*;

fn required_len(rect: &gegl::Rectangle, format: &babl::Format, rowstride: Option<usize>) -> usize {
    let width = usize::try_from(rect.width()).expect("negative rectangle width");
    let height = usize::try_from(rect.height()).expect("negative rectangle height");
    let row_len = width * format.bytes_per_pixel() as usize;
    let stride = rowstride.unwrap_or(row_len);
    assert!(
        stride >= row_len,
        "rowstride {stride} is smaller than a row ({row_len} bytes)"
    );
    match height {
        0 => 0,
        _ => stride * (height - 1) + row_len,
    }
}

fn rowstride_arg(rowstride: Option<usize>) -> libc::c_int {
    rowstride.map_or(gegl::ffi::GEGL_AUTO_ROWSTRIDE, |s| {
        libc::c_int::try_from(s).expect("rowstride too large")
    })
}

pub trait GeglBufferExtManual {
    /// Writes the pixels of `rect` from `src`, which is in `format`.
    ///
    /// `rowstride` is the distance between rows in bytes, `None` for tightly
    /// packed rows.
    ///
    /// # Panics
    ///
    /// If `src` is too small for `rect`.
    #[doc(alias = "gegl_buffer_set")]
    fn set_pixels(
        &self,
        rect: &gegl::Rectangle,
        mipmap_level: i32,
        format: &babl::Format,
        src: &[u8],
        rowstride: Option<usize>,
    );

    /// Reads the pixels of `rect` (in coordinates after `scale` is applied)
    /// into `dest`, converted to `format`.
    ///
    /// # Panics
    ///
    /// If `dest` is too small for `rect`.
    #[doc(alias = "gegl_buffer_get")]
    fn get_pixels_into(
        &self,
        rect: &gegl::Rectangle,
        scale: f64,
        format: &babl::Format,
        dest: &mut [u8],
        rowstride: Option<usize>,
        repeat_mode: gegl::AbyssPolicy,
    );

    /// Reads the pixels of `rect`, converted to `format`, tightly packed.
    #[doc(alias = "gegl_buffer_get")]
    fn get_pixels(
        &self,
        rect: &gegl::Rectangle,
        scale: f64,
        format: &babl::Format,
        repeat_mode: gegl::AbyssPolicy,
    ) -> Vec<u8> {
        let mut data = vec![0u8; required_len(rect, format, None)];
        self.get_pixels_into(rect, scale, format, &mut data, None, repeat_mode);
        data
    }
}

impl GeglBufferExtManual for gegl::Buffer {
    fn set_pixels(
        &self,
        rect: &gegl::Rectangle,
        mipmap_level: i32,
        format: &babl::Format,
        src: &[u8],
        rowstride: Option<usize>,
    ) {
        let required = required_len(rect, format, rowstride);
        assert!(
            src.len() >= required,
            "source has {} bytes, {required} are required",
            src.len()
        );
        unsafe {
            gegl::ffi::gegl_buffer_set(
                self.to_glib_none().0,
                rect.to_glib_none().0,
                mipmap_level,
                format.inner(),
                // gegl_buffer_set() does not write to src.
                src.as_ptr() as *mut libc::c_void,
                rowstride_arg(rowstride),
            )
        }
    }

    fn get_pixels_into(
        &self,
        rect: &gegl::Rectangle,
        scale: f64,
        format: &babl::Format,
        dest: &mut [u8],
        rowstride: Option<usize>,
        repeat_mode: gegl::AbyssPolicy,
    ) {
        let required = required_len(rect, format, rowstride);
        assert!(
            dest.len() >= required,
            "destination has {} bytes, {required} are required",
            dest.len()
        );
        unsafe {
            gegl::ffi::gegl_buffer_get(
                self.to_glib_none().0,
                rect.to_glib_none().0,
                scale,
                format.inner(),
                dest.as_mut_ptr() as glib::ffi::gpointer,
                rowstride_arg(rowstride),
                repeat_mode.into_glib(),
            )
        }
    }
}
