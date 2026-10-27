# gimp

Rust bindings for libgimp 3, for writing GIMP 3 plug-ins in Rust.

The API is generated from `Gimp-3.0.gir` with [gir](https://github.com/gtk-rs/gir),
the generator of the [gtk-rs](https://gtk-rs.org) crates, and builds on the gtk-rs
`glib`/`gio` crates and on [gegl-rs](https://gitlab.gnome.org/World/Rust/gegl-rs)
(`gegl`, `babl`). On top of the generated code the crate provides:

- `subclass::prelude::PlugInImpl` to implement a plug-in as a `GimpPlugIn`
  subclass, and `gimp::main!` as the equivalent of `GIMP_MAIN()`,
- constructors of `Procedure`, `ImageProcedure`, `LoadProcedure`,
  `ExportProcedure`, `ThumbnailProcedure` and `BatchProcedure` taking Rust
  closures. Panics in these closures are returned to GIMP as execution errors,
- `ValueArray::get`/`ValueArray::set` for procedure arguments and return values,
- `Drawable::format()` returning a `babl::Format`, and pixel access to
  `gegl::Buffer` in any babl format (`GeglBufferExtManual`),
- `ExportOptions::image()`, which deletes the converted export copy on drop.

```rust,ignore
use gimp::{prelude::*, subclass::prelude::*};

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct HelloPlugIn;

    #[glib::object_subclass]
    impl ObjectSubclass for HelloPlugIn {
        const NAME: &'static str = "HelloPlugIn";
        type Type = super::HelloPlugIn;
        type ParentType = gimp::PlugIn;
    }

    impl ObjectImpl for HelloPlugIn {}

    impl PlugInImpl for HelloPlugIn {
        fn query_procedures(&self) -> Vec<String> {
            vec!["plug-in-hello".into()]
        }

        fn create_procedure(&self, name: &str) -> Option<gimp::Procedure> {
            let procedure = gimp::ImageProcedure::new(
                &*self.obj(),
                name,
                gimp::PDBProcType::Plugin,
                |procedure, _run_mode, image, _drawables, _config| {
                    gimp::message(&format!("Hello! {}x{}", image.width(), image.height()));
                    procedure.new_return_values(gimp::PDBStatusType::Success, None)
                },
            );
            procedure.set_image_types("*");
            procedure.set_menu_label("Hello from _Rust");
            procedure.add_menu_path("<Image>/Filters/Development");
            Some(procedure.upcast())
        }
    }
}

glib::wrapper! {
    pub struct HelloPlugIn(ObjectSubclass<imp::HelloPlugIn>) @extends gimp::PlugIn;
}

gimp::main!(HelloPlugIn);
```

GIMP finds plug-ins in `~/.config/GIMP/3.x/plug-ins/<name>/<name>`, the
executable has to have the same name as its directory.
