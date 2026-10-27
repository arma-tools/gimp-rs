# gexiv2

Rust bindings for [GExiv2](https://gitlab.gnome.org/GNOME/gexiv2), generated
from `GExiv2-0.10.gir` with [gir](https://github.com/gtk-rs/gir).

This crate exists mainly because `GimpMetadata` is a subclass of
`GExiv2Metadata`, and the `gimp` crate needs it as a parent type.
