//! Plug-in used by `tests/run-gimp-tests.sh` to test the parts of the
//! bindings the example does not cover.

// GIMP runs plug-ins in the background, without a console window.
#![windows_subsystem = "windows"]

use gimp::{prelude::*, subclass::prelude::*};

const PANIC_PROC: &str = "plug-in-gimp-rs-test-panic";
const VALUES_PROC: &str = "plug-in-gimp-rs-test-values";

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct TestPlugIn;

    #[glib::object_subclass]
    impl ObjectSubclass for TestPlugIn {
        const NAME: &'static str = "GimpRsTestPlugIn";
        type Type = super::TestPlugIn;
        type ParentType = gimp::PlugIn;
    }

    impl ObjectImpl for TestPlugIn {}

    impl PlugInImpl for TestPlugIn {
        fn query_procedures(&self) -> Vec<String> {
            vec![PANIC_PROC.into(), VALUES_PROC.into()]
        }

        fn create_procedure(&self, name: &str) -> Option<gimp::Procedure> {
            let plug_in = &*self.obj();
            match name {
                PANIC_PROC => Some(gimp::Procedure::new(
                    plug_in,
                    name,
                    gimp::PDBProcType::Plugin,
                    |_procedure, _config| panic!("test panic"),
                )),
                VALUES_PROC => {
                    let procedure = gimp::Procedure::new(
                        plug_in,
                        name,
                        gimp::PDBProcType::Plugin,
                        |procedure, _config| {
                            let values =
                                procedure.new_return_values(gimp::PDBStatusType::Success, None);
                            assert_eq!(values.len(), 2);
                            // Wrong type is rejected, without touching the value.
                            assert!(values.set(1, &1.5f64).is_err());
                            assert!(values.set(2, &1i32).is_err());
                            values.set(1, &42i32).unwrap();
                            assert_eq!(values.get::<i32>(1).unwrap(), 42);
                            values
                        },
                    );
                    procedure.add_int_return_value(
                        "answer",
                        "Answer",
                        None,
                        0,
                        100,
                        0,
                        glib::ParamFlags::READWRITE,
                    );
                    Some(procedure)
                }
                _ => None,
            }
        }

        fn set_i18n(&self, _procedure_name: &str) -> Option<I18n> {
            None
        }
    }
}

glib::wrapper! {
    pub struct TestPlugIn(ObjectSubclass<imp::TestPlugIn>) @extends gimp::PlugIn;
}

gimp::main!(TestPlugIn);
