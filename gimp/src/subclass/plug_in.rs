use std::ptr;

use glib::{prelude::*, subclass::prelude::*, translate::*};

use crate::{ffi, PlugIn, Procedure};

/// Localization settings returned by [`PlugInImpl::set_i18n`].
///
/// `None` fields use GIMP's defaults: the plug-in name as gettext domain and
/// the `locale/` folder next to the plug-in executable as catalog directory.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct I18n {
    pub gettext_domain: Option<String>,
    /// Must be a relative path, below the directory of the plug-in executable.
    pub catalog_dir: Option<String>,
}

/// Virtual methods of `GimpPlugIn`.
///
/// Every plug-in subclasses [`PlugIn`], lists its procedures in
/// [`query_procedures`](Self::query_procedures) and creates them on demand in
/// [`create_procedure`](Self::create_procedure).
pub trait PlugInImpl: ObjectImpl + ObjectSubclass<Type: IsA<PlugIn>> {
    /// Names of the procedures this plug-in registers. Queried only once,
    /// after installation or when the plug-in changed.
    fn query_procedures(&self) -> Vec<String> {
        self.parent_query_procedures()
    }

    /// Names of the procedures this plug-in registers at every startup,
    /// typically ones that depend on runtime conditions.
    fn init_procedures(&self) -> Vec<String> {
        self.parent_init_procedures()
    }

    /// Creates the procedure `name`, which is one of the names returned by
    /// [`query_procedures`](Self::query_procedures) or
    /// [`init_procedures`](Self::init_procedures).
    fn create_procedure(&self, name: &str) -> Option<Procedure>;

    /// Called when the plug-in quits.
    fn quit(&self) {
        self.parent_quit()
    }

    /// Localization of `procedure_name`. Return `None` to disable
    /// localization.
    fn set_i18n(&self, procedure_name: &str) -> Option<I18n> {
        self.parent_set_i18n(procedure_name)
    }
}

pub trait PlugInImplExt: PlugInImpl {
    fn parent_query_procedures(&self) -> Vec<String> {
        unsafe {
            let data = Self::type_data();
            let parent_class = data.as_ref().parent_class() as *mut ffi::GimpPlugInClass;
            match (*parent_class).query_procedures {
                Some(f) => FromGlibPtrContainer::from_glib_full(f(self
                    .obj()
                    .unsafe_cast_ref::<PlugIn>()
                    .to_glib_none()
                    .0)),
                None => Vec::new(),
            }
        }
    }

    fn parent_init_procedures(&self) -> Vec<String> {
        unsafe {
            let data = Self::type_data();
            let parent_class = data.as_ref().parent_class() as *mut ffi::GimpPlugInClass;
            match (*parent_class).init_procedures {
                Some(f) => FromGlibPtrContainer::from_glib_full(f(self
                    .obj()
                    .unsafe_cast_ref::<PlugIn>()
                    .to_glib_none()
                    .0)),
                None => Vec::new(),
            }
        }
    }

    fn parent_create_procedure(&self, name: &str) -> Option<Procedure> {
        unsafe {
            let data = Self::type_data();
            let parent_class = data.as_ref().parent_class() as *mut ffi::GimpPlugInClass;
            let f = (*parent_class).create_procedure?;
            from_glib_full(f(
                self.obj().unsafe_cast_ref::<PlugIn>().to_glib_none().0,
                name.to_glib_none().0,
            ))
        }
    }

    fn parent_quit(&self) {
        unsafe {
            let data = Self::type_data();
            let parent_class = data.as_ref().parent_class() as *mut ffi::GimpPlugInClass;
            if let Some(f) = (*parent_class).quit {
                f(self.obj().unsafe_cast_ref::<PlugIn>().to_glib_none().0)
            }
        }
    }

    fn parent_set_i18n(&self, procedure_name: &str) -> Option<I18n> {
        unsafe {
            let data = Self::type_data();
            let parent_class = data.as_ref().parent_class() as *mut ffi::GimpPlugInClass;
            let f = (*parent_class).set_i18n?;
            let mut gettext_domain = ptr::null_mut();
            let mut catalog_dir = ptr::null_mut();
            let enabled: bool = from_glib(f(
                self.obj().unsafe_cast_ref::<PlugIn>().to_glib_none().0,
                procedure_name.to_glib_none().0,
                &mut gettext_domain,
                &mut catalog_dir,
            ));
            let i18n = I18n {
                gettext_domain: from_glib_full(gettext_domain),
                catalog_dir: from_glib_full(catalog_dir),
            };
            enabled.then_some(i18n)
        }
    }
}

impl<T: PlugInImpl> PlugInImplExt for T {}

unsafe impl<T: PlugInImpl> IsSubclassable<T> for PlugIn {
    fn class_init(class: &mut glib::Class<Self>) {
        Self::parent_class_init::<T>(class);

        let klass = class.as_mut();
        klass.query_procedures = Some(plug_in_query_procedures::<T>);
        klass.init_procedures = Some(plug_in_init_procedures::<T>);
        klass.create_procedure = Some(plug_in_create_procedure::<T>);
        klass.quit = Some(plug_in_quit::<T>);
        klass.set_i18n = Some(plug_in_set_i18n::<T>);
    }
}

unsafe extern "C" fn plug_in_query_procedures<T: PlugInImpl>(
    ptr: *mut ffi::GimpPlugIn,
) -> *mut glib::ffi::GList {
    unsafe {
        let instance = &*(ptr as *mut T::Instance);
        instance.imp().query_procedures().to_glib_full()
    }
}

unsafe extern "C" fn plug_in_init_procedures<T: PlugInImpl>(
    ptr: *mut ffi::GimpPlugIn,
) -> *mut glib::ffi::GList {
    unsafe {
        let instance = &*(ptr as *mut T::Instance);
        instance.imp().init_procedures().to_glib_full()
    }
}

unsafe extern "C" fn plug_in_create_procedure<T: PlugInImpl>(
    ptr: *mut ffi::GimpPlugIn,
    name: *const libc::c_char,
) -> *mut ffi::GimpProcedure {
    unsafe {
        let instance = &*(ptr as *mut T::Instance);
        let name: Borrowed<glib::GString> = from_glib_borrow(name);
        instance
            .imp()
            .create_procedure(name.as_str())
            .into_glib_ptr()
    }
}

unsafe extern "C" fn plug_in_quit<T: PlugInImpl>(ptr: *mut ffi::GimpPlugIn) {
    unsafe {
        let instance = &*(ptr as *mut T::Instance);
        instance.imp().quit()
    }
}

unsafe extern "C" fn plug_in_set_i18n<T: PlugInImpl>(
    ptr: *mut ffi::GimpPlugIn,
    procedure_name: *const libc::c_char,
    gettext_domain: *mut *mut libc::c_char,
    catalog_dir: *mut *mut libc::c_char,
) -> glib::ffi::gboolean {
    unsafe {
        let instance = &*(ptr as *mut T::Instance);
        let procedure_name: Borrowed<glib::GString> = from_glib_borrow(procedure_name);
        match instance.imp().set_i18n(procedure_name.as_str()) {
            Some(i18n) => {
                if !gettext_domain.is_null() {
                    *gettext_domain = i18n.gettext_domain.to_glib_full();
                }
                if !catalog_dir.is_null() {
                    *catalog_dir = i18n.catalog_dir.to_glib_full();
                }
                glib::ffi::GTRUE
            }
            None => glib::ffi::GFALSE,
        }
    }
}
