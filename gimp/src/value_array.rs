use glib::{prelude::*, translate::*};

use crate::{ffi, ValueArray};

impl ValueArray {
    #[doc(alias = "gimp_value_array_new_from_values")]
    pub fn from_values(values: &[glib::Value]) -> ValueArray {
        unsafe {
            from_glib_full(ffi::gimp_value_array_new_from_values(
                ToGlibContainerFromSlice::to_glib_none_from_slice(values).0,
                values.len() as libc::c_int,
            ))
        }
    }

    /// Number of values.
    #[doc(alias = "gimp_value_array_length")]
    pub fn len(&self) -> usize {
        self.length() as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets the value at `index` as `T`.
    pub fn get<T>(&self, index: usize) -> Result<T, glib::BoolError>
    where
        T: for<'b> glib::value::FromValue<'b> + 'static,
    {
        let value = self
            .index(index as i32)
            .filter(|_| index < self.len())
            .ok_or_else(|| glib::bool_error!("Index {index} out of bounds"))?;
        value
            .get::<T>()
            .map_err(|e| glib::bool_error!("Value {index}: {e}"))
    }

    /// Replaces the value at `index`. The value must be compatible with the
    /// type of the existing value, which for return values of a procedure is
    /// the declared type of the return value.
    ///
    /// For return values, index 0 is the PDB status, the declared return
    /// values start at index 1.
    pub fn set<T: ToValue + ?Sized>(&self, index: usize, value: &T) -> Result<(), glib::BoolError> {
        if index >= self.len() {
            return Err(glib::bool_error!(
                "Index {index} out of bounds, the array has {} values",
                self.len()
            ));
        }
        let value = value.to_value();
        unsafe {
            let dest = ffi::gimp_value_array_index(self.to_glib_none().0, index as libc::c_int);
            let dest_type: glib::Type = from_glib((*dest).g_type);
            if !value.type_().is_a(dest_type) {
                return Err(glib::bool_error!(
                    "Value {index} has type {dest_type}, which is incompatible with {}",
                    value.type_()
                ));
            }
            glib::gobject_ffi::g_value_copy(value.to_glib_none().0, dest);
        }
        Ok(())
    }

    /// Copies of all values.
    pub fn values(&self) -> Vec<glib::Value> {
        (0..self.length()).filter_map(|i| self.index(i)).collect()
    }
}
