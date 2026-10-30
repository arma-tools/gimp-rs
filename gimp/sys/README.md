# gimp-sys

FFI bindings for libgimp 3 (`gimp-3.0`), generated from `Gimp-3.0.gir` with
[gir](https://github.com/gtk-rs/gir).

These are the raw C declarations. To write GIMP plug-ins use the safe bindings
in the [`gimp`](https://crates.io/crates/gimp) crate.

## Requirements

The GIMP 3 development files (`gimp-3.0.pc`), found with `pkg-config` through
[system-deps](https://crates.io/crates/system-deps). Features `v3_0_8`, `v3_1`,
`v3_2` and `v3_2_4` enable the API of later GIMP versions and require at least
that version.
