use std::ffi::CString;

use glib::{prelude::*, translate::*};

use crate::{ffi, PlugIn};

/// Runs the plug-in of type `T`. This is the equivalent of the `GIMP_MAIN()`
/// macro and has to be called from the executable's `main()`, see
/// [`gimp::main!`](crate::main!).
///
/// GIMP starts the plug-in executable with its own arguments, which are
/// passed through to `gimp_main()`.
#[doc(alias = "gimp_main")]
#[doc(alias = "GIMP_MAIN")]
pub fn main<T: StaticType + IsA<PlugIn>>() -> glib::ExitCode {
    #[cfg(unix)]
    let args: Vec<CString> = {
        use std::os::unix::ffi::OsStringExt;
        std::env::args_os()
            .map(|arg| CString::new(arg.into_vec()).expect("argument contains a NUL byte"))
            .collect()
    };
    #[cfg(not(unix))]
    let args: Vec<CString> = std::env::args()
        .map(|arg| CString::new(arg).expect("argument contains a NUL byte"))
        .collect();

    let mut argv: Vec<*mut libc::c_char> = args
        .iter()
        .map(|arg| arg.as_ptr() as *mut libc::c_char)
        .chain(std::iter::once(std::ptr::null_mut()))
        .collect();

    let status = unsafe {
        ffi::gimp_main(
            T::static_type().into_glib(),
            args.len() as libc::c_int,
            argv.as_mut_ptr(),
        )
    };
    glib::ExitCode::try_from(status).unwrap_or(glib::ExitCode::FAILURE)
}

/// Generates the `main()` function of a plug-in executable, running the
/// [`PlugIn`] subclass `$plug_in`:
///
/// ```ignore
/// glib::wrapper! {
///     pub struct MyPlugIn(ObjectSubclass<imp::MyPlugIn>) @extends gimp::PlugIn;
/// }
///
/// gimp::main!(MyPlugIn);
/// ```
#[macro_export]
macro_rules! main {
    ($plug_in:ty) => {
        fn main() -> $crate::glib::ExitCode {
            $crate::main::<$plug_in>()
        }
    };
}
