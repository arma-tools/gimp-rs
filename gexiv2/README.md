# gexiv2

Rust bindings for [GExiv2](https://gitlab.gnome.org/GNOME/gexiv2), the GObject
wrapper of [Exiv2](https://exiv2.org) for reading and writing Exif, IPTC and
XMP metadata. Generated from `GExiv2-0.10.gir` with
[gir](https://github.com/gtk-rs/gir).

This crate exists mainly because `GimpMetadata` is a subclass of
`GExiv2Metadata`, and the [`gimp`](https://crates.io/crates/gimp) crate needs it
as a parent type, but it can be used on its own as well.

The documentation of the generated API is taken from the GIR file, see also
the [GExiv2 reference](https://gnome.pages.gitlab.gnome.org/gexiv2/docs/).

## Requirements

The GExiv2 development files (`gexiv2.pc`, GExiv2 0.10 or later), for example
the `libgexiv2-devel` package on openSUSE, `gexiv2-devel` on Fedora,
`libgexiv2-dev` on Debian and Ubuntu, or `libgexiv2` on Arch Linux.

```toml
[dependencies]
gexiv2 = "0.1"
```

## Features

| feature   | GExiv2 |
|-----------|--------|
| `v0_10_3` | 0.10.3 |
| `v0_10_6` | 0.10.6 |
| `v0_12_1` | 0.12.1 |
| `v0_12_2` | 0.12.2 |
| `v0_14`   | 0.14   |

## Example

```rust,no_run
use gexiv2::prelude::*;

let metadata = gexiv2::Metadata::new();
metadata.open_path("photo.jpg").unwrap();
if let Some(model) = metadata.tag_string("Exif.Image.Model") {
    println!("Camera: {model}");
}
```
