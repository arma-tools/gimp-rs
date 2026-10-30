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

The documentation of the generated API is taken from the GIR file, see also
the [libgimp reference](https://developer.gimp.org/api/3.0/libgimp/). Dialogs
and widgets for plug-ins are in the [`gimp-ui`](https://crates.io/crates/gimp-ui)
crate.

## Requirements

The GIMP 3 development files (`gimp-3.0.pc`), for example the `gimp-devel`
package on openSUSE and Fedora, `libgimp-3.0-dev` on Debian and Ubuntu, `gimp`
on Arch Linux, or `mingw-w64-ucrt-x86_64-gimp` in MSYS2 (UCRT64) on Windows.

```toml
[dependencies]
gimp = "0.1"
glib = "0.22"
```

`glib` is needed for the `glib::wrapper!` and `#[glib::object_subclass]`
macros. `gimp` re-exports the versions of `glib`, `gio`, `gegl`, `babl`,
`gexiv2`, `gdk_pixbuf`, `cairo` and `pango` it uses.

## Features

Without features, only the API of GIMP 3.0 is available. API added in later
versions is enabled with these features, each requires at least that GIMP
version at build and run time:

| feature  | GIMP  |
|----------|-------|
| `v3_0_8` | 3.0.8 |
| `v3_1`   | 3.1   |
| `v3_2`   | 3.2   |
| `v3_2_4` | 3.2.4 |

## Example

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

A larger example, with a file loader and exporter, is in the
[repository](https://github.com/arma-tools/gimp-rs/blob/main/examples/src/main.rs).

## Installing a plug-in

GIMP looks for plug-ins in the `plug-ins` folder of the user's profile, each in
a directory of the same name as the executable:

- Linux: `~/.config/GIMP/3.x/plug-ins/<name>/<name>`
- Windows: `%APPDATA%\GIMP\3.x\plug-ins\<name>\<name>.exe`
- macOS: `~/Library/Application Support/GIMP/3.x/plug-ins/<name>/<name>`

Other folders can be added in Preferences → Folders → Plug-ins. GIMP runs
plug-ins without a console, on Windows add `#![windows_subsystem = "windows"]`
to keep a console window from opening.

## Panics

Unwinding into libgimp is not allowed. Panics in the `run` closures of
procedures are caught and returned to GIMP as execution errors, panics
elsewhere (for example in `PlugInImpl` methods) abort the plug-in.
