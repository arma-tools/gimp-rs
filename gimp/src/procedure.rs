//! Constructors of the procedure types, taking Rust closures as `run()`
//! functions, and helpers to build their return values.

use std::panic::{self, AssertUnwindSafe};

use glib::{prelude::*, translate::*};

use crate::{
    ffi, prelude::*, BatchProcedure, Drawable, ExportOptions, ExportProcedure, Image,
    ImageProcedure, LoadProcedure, Metadata, MetadataLoadFlags, PDBProcType, PDBStatusType, PlugIn,
    Procedure, ProcedureConfig, RunMode, ThumbnailProcedure, ValueArray,
};

/// Error domain of `GIMP_PLUG_IN_ERROR`, used for errors returned by
/// procedures. GIMP does not interpret the error code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[doc(alias = "GIMP_PLUG_IN_ERROR")]
pub enum PlugInError {
    Failed,
}

impl glib::error::ErrorDomain for PlugInError {
    #[inline]
    fn domain() -> glib::Quark {
        unsafe { from_glib(ffi::gimp_plug_in_error_quark()) }
    }

    #[inline]
    fn code(self) -> i32 {
        match self {
            Self::Failed => 0,
        }
    }

    #[inline]
    fn from(_code: i32) -> Option<Self> {
        Some(Self::Failed)
    }
}

pub trait ProcedureExtManual: IsA<Procedure> + 'static {
    /// Creates the return values of this procedure: the status, the error
    /// (for statuses other than [`PDBStatusType::Success`]), followed by
    /// default values for all return values the procedure declares. Set
    /// those with [`ValueArray::set`], index 0 is the status.
    #[doc(alias = "gimp_procedure_new_return_values")]
    fn new_return_values(&self, status: PDBStatusType, error: Option<glib::Error>) -> ValueArray {
        unsafe {
            from_glib_full(ffi::gimp_procedure_new_return_values(
                self.as_ref().to_glib_none().0,
                status.into_glib(),
                error.map_or(std::ptr::null_mut(), |e| e.into_glib_ptr()),
            ))
        }
    }

    /// Shorthand for the return values of a procedure: successful ones for
    /// `Ok`, [`PDBStatusType::ExecutionError`] with the error for `Err`.
    fn return_result(&self, result: Result<ValueArray, glib::Error>) -> ValueArray {
        result.unwrap_or_else(|error| {
            self.new_return_values(PDBStatusType::ExecutionError, Some(error))
        })
    }
}

impl<O: IsA<Procedure>> ProcedureExtManual for O {}

/// Runs `f` and converts a panic into an execution error, as unwinding into
/// libgimp is not allowed and GIMP reports a crashed plug-in only vaguely.
fn guard(procedure: &Procedure, f: impl FnOnce() -> ValueArray) -> *mut ffi::GimpValueArray {
    match panic::catch_unwind(AssertUnwindSafe(f)) {
        Ok(values) => values.into_glib_ptr(),
        Err(payload) => {
            let message = payload
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "unknown panic".to_owned());
            let error = glib::Error::new(
                PlugInError::Failed,
                &format!(
                    "{} panicked: {message}",
                    procedure.name().unwrap_or_default()
                ),
            );
            procedure
                .new_return_values(PDBStatusType::ExecutionError, Some(error))
                .into_glib_ptr()
        }
    }
}

unsafe extern "C" fn destroy_closure<F>(data: glib::ffi::gpointer) {
    unsafe { drop(Box::from_raw(data as *mut F)) }
}

impl Procedure {
    /// Creates a generic procedure. Plug-ins usually want one of the more
    /// specific types, like [`ImageProcedure`] or [`LoadProcedure`].
    #[doc(alias = "gimp_procedure_new")]
    pub fn new<F>(
        plug_in: &impl IsA<PlugIn>,
        name: &str,
        proc_type: PDBProcType,
        run_func: F,
    ) -> Procedure
    where
        F: Fn(&Procedure, &ProcedureConfig) -> ValueArray + 'static,
    {
        unsafe extern "C" fn trampoline<F>(
            procedure: *mut ffi::GimpProcedure,
            config: *mut ffi::GimpProcedureConfig,
            run_data: glib::ffi::gpointer,
        ) -> *mut ffi::GimpValueArray
        where
            F: Fn(&Procedure, &ProcedureConfig) -> ValueArray + 'static,
        {
            unsafe {
                let procedure: Borrowed<Procedure> = from_glib_borrow(procedure);
                let config: Borrowed<ProcedureConfig> = from_glib_borrow(config);
                let f = &*(run_data as *const F);
                guard(&procedure, || f(&procedure, &config))
            }
        }

        unsafe {
            from_glib_full(ffi::gimp_procedure_new(
                plug_in.as_ref().to_glib_none().0,
                name.to_glib_none().0,
                proc_type.into_glib(),
                Some(trampoline::<F>),
                Box::into_raw(Box::new(run_func)) as glib::ffi::gpointer,
                Some(destroy_closure::<F>),
            ))
        }
    }
}

impl ImageProcedure {
    /// Creates a procedure that works on an image and its selected drawables,
    /// typically a filter.
    ///
    /// `run_func` receives the procedure, run mode, image, selected
    /// drawables and the procedure's config.
    #[doc(alias = "gimp_image_procedure_new")]
    pub fn new<F>(
        plug_in: &impl IsA<PlugIn>,
        name: &str,
        proc_type: PDBProcType,
        run_func: F,
    ) -> ImageProcedure
    where
        F: Fn(&ImageProcedure, RunMode, &Image, &[Drawable], &ProcedureConfig) -> ValueArray
            + 'static,
    {
        unsafe extern "C" fn trampoline<F>(
            procedure: *mut ffi::GimpProcedure,
            run_mode: ffi::GimpRunMode,
            image: *mut ffi::GimpImage,
            drawables: *mut *mut ffi::GimpDrawable,
            config: *mut ffi::GimpProcedureConfig,
            run_data: glib::ffi::gpointer,
        ) -> *mut ffi::GimpValueArray
        where
            F: Fn(&ImageProcedure, RunMode, &Image, &[Drawable], &ProcedureConfig) -> ValueArray
                + 'static,
        {
            unsafe {
                let procedure: Borrowed<Procedure> = from_glib_borrow(procedure);
                let image: Borrowed<Image> = from_glib_borrow(image);
                let drawables: Vec<Drawable> = FromGlibPtrContainer::from_glib_none(drawables);
                let config: Borrowed<ProcedureConfig> = from_glib_borrow(config);
                let f = &*(run_data as *const F);
                guard(&procedure, || {
                    f(
                        procedure.unsafe_cast_ref(),
                        from_glib(run_mode),
                        &image,
                        &drawables,
                        &config,
                    )
                })
            }
        }

        unsafe {
            Procedure::from_glib_full(ffi::gimp_image_procedure_new(
                plug_in.as_ref().to_glib_none().0,
                name.to_glib_none().0,
                proc_type.into_glib(),
                Some(trampoline::<F>),
                Box::into_raw(Box::new(run_func)) as glib::ffi::gpointer,
                Some(destroy_closure::<F>),
            ))
            .unsafe_cast()
        }
    }
}

impl LoadProcedure {
    /// Creates a file load procedure. Register file extensions, mime types
    /// and magics with the [`FileProcedureExt`](crate::prelude::FileProcedureExt)
    /// methods.
    ///
    /// `run_func` receives the procedure, run mode, file, metadata, metadata
    /// load flags and the procedure's config. The image is returned as value
    /// 1 of the return values:
    ///
    /// ```ignore
    /// let values = procedure.new_return_values(PDBStatusType::Success, None);
    /// values.set(1, &image)?;
    /// ```
    ///
    /// Clear bits in the flags to stop GIMP from loading the corresponding
    /// metadata after the image is loaded.
    #[doc(alias = "gimp_load_procedure_new")]
    pub fn new<F>(
        plug_in: &impl IsA<PlugIn>,
        name: &str,
        proc_type: PDBProcType,
        run_func: F,
    ) -> LoadProcedure
    where
        F: Fn(
                &LoadProcedure,
                RunMode,
                &gio::File,
                Option<&Metadata>,
                &mut MetadataLoadFlags,
                &ProcedureConfig,
            ) -> ValueArray
            + 'static,
    {
        unsafe extern "C" fn trampoline<F>(
            procedure: *mut ffi::GimpProcedure,
            run_mode: ffi::GimpRunMode,
            file: *mut gio::ffi::GFile,
            metadata: *mut ffi::GimpMetadata,
            flags: *mut ffi::GimpMetadataLoadFlags,
            config: *mut ffi::GimpProcedureConfig,
            run_data: glib::ffi::gpointer,
        ) -> *mut ffi::GimpValueArray
        where
            F: Fn(
                    &LoadProcedure,
                    RunMode,
                    &gio::File,
                    Option<&Metadata>,
                    &mut MetadataLoadFlags,
                    &ProcedureConfig,
                ) -> ValueArray
                + 'static,
        {
            unsafe {
                let procedure: Borrowed<Procedure> = from_glib_borrow(procedure);
                let file: Borrowed<gio::File> = from_glib_borrow(file);
                let metadata: Borrowed<Option<Metadata>> = from_glib_borrow(metadata);
                let config: Borrowed<ProcedureConfig> = from_glib_borrow(config);
                let mut load_flags = if flags.is_null() {
                    MetadataLoadFlags::empty()
                } else {
                    from_glib(*flags)
                };
                let f = &*(run_data as *const F);
                let values = guard(&procedure, || {
                    f(
                        procedure.unsafe_cast_ref(),
                        from_glib(run_mode),
                        &file,
                        metadata.as_ref().as_ref(),
                        &mut load_flags,
                        &config,
                    )
                });
                if !flags.is_null() {
                    *flags = load_flags.into_glib();
                }
                values
            }
        }

        unsafe {
            Procedure::from_glib_full(ffi::gimp_load_procedure_new(
                plug_in.as_ref().to_glib_none().0,
                name.to_glib_none().0,
                proc_type.into_glib(),
                Some(trampoline::<F>),
                Box::into_raw(Box::new(run_func)) as glib::ffi::gpointer,
                Some(destroy_closure::<F>),
            ))
            .unsafe_cast()
        }
    }
}

impl ExportProcedure {
    /// Creates a file export procedure. Set what the format supports with
    /// [`ExportProcedure::set_capabilities`], GIMP converts the image
    /// accordingly before calling `run_func`.
    ///
    /// With `export_metadata`, GIMP handles the export of metadata, `run_func`
    /// may update the metadata it receives.
    ///
    /// `run_func` receives the procedure, run mode, image, file, export
    /// options, metadata and the procedure's config.
    #[doc(alias = "gimp_export_procedure_new")]
    pub fn new<F>(
        plug_in: &impl IsA<PlugIn>,
        name: &str,
        proc_type: PDBProcType,
        export_metadata: bool,
        run_func: F,
    ) -> ExportProcedure
    where
        F: Fn(
                &ExportProcedure,
                RunMode,
                &Image,
                &gio::File,
                &ExportOptions,
                Option<&Metadata>,
                &ProcedureConfig,
            ) -> ValueArray
            + 'static,
    {
        #[allow(clippy::too_many_arguments)]
        unsafe extern "C" fn trampoline<F>(
            procedure: *mut ffi::GimpProcedure,
            run_mode: ffi::GimpRunMode,
            image: *mut ffi::GimpImage,
            file: *mut gio::ffi::GFile,
            options: *mut ffi::GimpExportOptions,
            metadata: *mut ffi::GimpMetadata,
            config: *mut ffi::GimpProcedureConfig,
            run_data: glib::ffi::gpointer,
        ) -> *mut ffi::GimpValueArray
        where
            F: Fn(
                    &ExportProcedure,
                    RunMode,
                    &Image,
                    &gio::File,
                    &ExportOptions,
                    Option<&Metadata>,
                    &ProcedureConfig,
                ) -> ValueArray
                + 'static,
        {
            unsafe {
                let procedure: Borrowed<Procedure> = from_glib_borrow(procedure);
                let image: Borrowed<Image> = from_glib_borrow(image);
                let file: Borrowed<gio::File> = from_glib_borrow(file);
                let options: Borrowed<ExportOptions> = from_glib_borrow(options);
                let metadata: Borrowed<Option<Metadata>> = from_glib_borrow(metadata);
                let config: Borrowed<ProcedureConfig> = from_glib_borrow(config);
                let f = &*(run_data as *const F);
                guard(&procedure, || {
                    f(
                        procedure.unsafe_cast_ref(),
                        from_glib(run_mode),
                        &image,
                        &file,
                        &options,
                        metadata.as_ref().as_ref(),
                        &config,
                    )
                })
            }
        }

        unsafe {
            Procedure::from_glib_full(ffi::gimp_export_procedure_new(
                plug_in.as_ref().to_glib_none().0,
                name.to_glib_none().0,
                proc_type.into_glib(),
                export_metadata.into_glib(),
                Some(trampoline::<F>),
                Box::into_raw(Box::new(run_func)) as glib::ffi::gpointer,
                Some(destroy_closure::<F>),
            ))
            .unsafe_cast()
        }
    }
}

impl ThumbnailProcedure {
    /// Creates a procedure that loads a thumbnail of a file.
    ///
    /// `run_func` receives the procedure, file, requested size and the
    /// procedure's config.
    #[doc(alias = "gimp_thumbnail_procedure_new")]
    pub fn new<F>(
        plug_in: &impl IsA<PlugIn>,
        name: &str,
        proc_type: PDBProcType,
        run_func: F,
    ) -> ThumbnailProcedure
    where
        F: Fn(&ThumbnailProcedure, &gio::File, i32, &ProcedureConfig) -> ValueArray + 'static,
    {
        unsafe extern "C" fn trampoline<F>(
            procedure: *mut ffi::GimpProcedure,
            file: *mut gio::ffi::GFile,
            size: libc::c_int,
            config: *mut ffi::GimpProcedureConfig,
            run_data: glib::ffi::gpointer,
        ) -> *mut ffi::GimpValueArray
        where
            F: Fn(&ThumbnailProcedure, &gio::File, i32, &ProcedureConfig) -> ValueArray + 'static,
        {
            unsafe {
                let procedure: Borrowed<Procedure> = from_glib_borrow(procedure);
                let file: Borrowed<gio::File> = from_glib_borrow(file);
                let config: Borrowed<ProcedureConfig> = from_glib_borrow(config);
                let f = &*(run_data as *const F);
                guard(&procedure, || {
                    f(procedure.unsafe_cast_ref(), &file, size, &config)
                })
            }
        }

        unsafe {
            Procedure::from_glib_full(ffi::gimp_thumbnail_procedure_new(
                plug_in.as_ref().to_glib_none().0,
                name.to_glib_none().0,
                proc_type.into_glib(),
                Some(trampoline::<F>),
                Box::into_raw(Box::new(run_func)) as glib::ffi::gpointer,
                Some(destroy_closure::<F>),
            ))
            .unsafe_cast()
        }
    }
}

impl BatchProcedure {
    /// Creates a batch procedure, which runs `command`s of an interpreter
    /// (`gimp -b`).
    ///
    /// `run_func` receives the procedure, run mode, command and the
    /// procedure's config.
    #[doc(alias = "gimp_batch_procedure_new")]
    pub fn new<F>(
        plug_in: &impl IsA<PlugIn>,
        name: &str,
        interpreter_name: &str,
        proc_type: PDBProcType,
        run_func: F,
    ) -> BatchProcedure
    where
        F: Fn(&BatchProcedure, RunMode, &str, &ProcedureConfig) -> ValueArray + 'static,
    {
        unsafe extern "C" fn trampoline<F>(
            procedure: *mut ffi::GimpProcedure,
            run_mode: ffi::GimpRunMode,
            command: *const libc::c_char,
            config: *mut ffi::GimpProcedureConfig,
            run_data: glib::ffi::gpointer,
        ) -> *mut ffi::GimpValueArray
        where
            F: Fn(&BatchProcedure, RunMode, &str, &ProcedureConfig) -> ValueArray + 'static,
        {
            unsafe {
                let procedure: Borrowed<Procedure> = from_glib_borrow(procedure);
                let command: Borrowed<glib::GString> = from_glib_borrow(command);
                let config: Borrowed<ProcedureConfig> = from_glib_borrow(config);
                let f = &*(run_data as *const F);
                guard(&procedure, || {
                    f(
                        procedure.unsafe_cast_ref(),
                        from_glib(run_mode),
                        command.as_str(),
                        &config,
                    )
                })
            }
        }

        unsafe {
            Procedure::from_glib_full(ffi::gimp_batch_procedure_new(
                plug_in.as_ref().to_glib_none().0,
                name.to_glib_none().0,
                interpreter_name.to_glib_none().0,
                proc_type.into_glib(),
                Some(trampoline::<F>),
                Box::into_raw(Box::new(run_func)) as glib::ffi::gpointer,
                Some(destroy_closure::<F>),
            ))
            .unsafe_cast()
        }
    }
}
