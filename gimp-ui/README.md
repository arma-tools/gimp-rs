# gimp-ui

Rust bindings for libgimpui 3 (procedure dialogs and GIMP widgets), generated
from `GimpUi-3.0.gir` with [gir](https://github.com/gtk-rs/gir). Builds on the
[`gimp`](https://crates.io/crates/gimp) crate and gtk-rs' `gtk` 0.19 (GTK 3).

The documentation of the generated API is taken from the GIR file, see also
the [libgimpui reference](https://developer.gimp.org/api/3.0/libgimpui/).

## Requirements

The GIMP 3 development files (`gimp-3.0.pc`, `gimpui-3.0.pc`) and GTK 3, see
the [`gimp`](https://crates.io/crates/gimp) crate.

```toml
[dependencies]
gimp = "0.1"
gimp-ui = "0.1"
glib = "0.22"
```

`gimp-ui` re-exports the versions of `gimp`, `gtk`, `gdk` and the other
gtk-rs crates it uses.

## Features

| feature | GIMP |
|---------|------|
| `v3_2`  | 3.2  |

## Example

A `ProcedureDialog` shows a widget for each argument of a procedure and
stores the values in the procedure's config:

```rust,ignore
use gimp_ui::prelude::*;

let procedure = gimp::ImageProcedure::new(
    plug_in,
    name,
    gimp::PDBProcType::Plugin,
    |procedure, run_mode, image, drawables, config| {
        if run_mode == gimp::RunMode::Interactive {
            gimp_ui::init("my-plug-in");
            let dialog = gimp_ui::ProcedureDialog::new(procedure, config, None);
            dialog.fill_list(&[]); // all arguments
            // gtk::Dialog and gimp_ui::Dialog have a run() method as well.
            let ok = ProcedureDialogExt::run(&dialog);
            // gtk_widget_destroy()
            unsafe { dialog.destroy() };
            if !ok {
                return procedure.new_return_values(gimp::PDBStatusType::Cancel, None);
            }
        }
        let radius: f64 = config.property("radius");
        // ...
        procedure.new_return_values(gimp::PDBStatusType::Success, None)
    },
);
procedure.add_double_argument(
    "radius",
    "_Radius",
    Some("Radius in pixels"),
    0.0,
    100.0,
    5.0,
    glib::ParamFlags::READWRITE,
);
```
