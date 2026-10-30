# gimp-gexiv2-sys

FFI bindings for [GExiv2](https://gitlab.gnome.org/GNOME/gexiv2), generated
from `GExiv2-0.10.gir` with [gir](https://github.com/gtk-rs/gir).

These are the raw C declarations, the library is called `gexiv2_sys`. For the
safe bindings use the [`gimp-gexiv2`](https://crates.io/crates/gimp-gexiv2)
crate. Unrelated to the hand-written
[`gexiv2-sys`](https://crates.io/crates/gexiv2-sys) crate.

## Requirements

The GExiv2 development files (`gexiv2.pc`, GExiv2 0.10 or later), found with
`pkg-config` through [system-deps](https://crates.io/crates/system-deps).
Features `v0_10_3` to `v0_14` enable the API of later GExiv2 versions.
