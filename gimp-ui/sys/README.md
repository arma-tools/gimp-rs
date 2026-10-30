# gimp-ui-sys

FFI bindings for libgimpui 3 (`gimpui-3.0`), generated from `GimpUi-3.0.gir`
with [gir](https://github.com/gtk-rs/gir).

These are the raw C declarations. For the safe bindings use the
[`gimp-ui`](https://crates.io/crates/gimp-ui) crate.

## Requirements

The GIMP 3 development files (`gimpui-3.0.pc`) and GTK 3, found with
`pkg-config` through [system-deps](https://crates.io/crates/system-deps). The
feature `v3_2` enables the API of GIMP 3.2.
