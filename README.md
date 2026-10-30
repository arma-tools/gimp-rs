# gimp-rs

Rust bindings for GIMP 3's plug-in libraries, generated from the GObject
introspection (GIR) files with [gir](https://github.com/gtk-rs/gir), the way the
[gtk-rs](https://gtk-rs.org) and [gegl-rs](https://gitlab.gnome.org/World/Rust/gegl-rs)
crates are made. The crates use the gtk-rs crates (`glib`, `gio`, `gtk` 0.19, ...)
and gegl-rs (`gegl`, `babl`) for all types from other libraries.

| crate | |
|---|---|
| [`gimp`](gimp) / `gimp-sys` | libgimp: plug-ins, procedures, images, drawables, ... |
| [`gimp-ui`](gimp-ui) / `gimp-ui-sys` | libgimpui: procedure dialogs and widgets |
| [`gimp-gexiv2`](gexiv2) / `gimp-gexiv2-sys` | GExiv2, needed as `GimpMetadata` is a `GExiv2Metadata` |
| [`examples`](examples) | example plug-in |

Requires GIMP 3 development files (`gimp-3.0.pc`, `gimpui-3.0.pc`).

## Layout

- `gir/`, `gir-files/`: submodules, the generator and gtk-rs' GIR files, at the
  revisions gegl-rs uses (gtk-rs 0.22).
- `gir-files-extra/`: GIR files of GIMP, GExiv2 (fixed with `fix.py`) and of
  Babl/Gegl (copied from gegl-rs).
- `*/Gir.toml`: the generator config of each crate. `src/auto/` and the sys
  crates are generated, everything else in `src/` is hand-written.

## Regenerating

```sh
git submodule update --init
cargo install rustdoc-stripper
gir-files-extra/fix.py      # only to update the GIR files from /usr/share/gir-1.0
./generate.sh
```

`generate.sh` also embeds the documentation of the GIR files as doc comments
into `src/auto/` of the high-level crates (with `gir -m doc` and
`rustdoc-stripper`, as gtk-rs does).

`fix.py` copies the GIMP and GExiv2 GIR files and fixes what g-ir-scanner got
wrong (constants built from `G_PARAM_USER_SHIFT`, the gimpui pkg-config name,
`const` object arrays, ...). The sys crates' tests compare struct layouts and
constants with the C headers:

```sh
cargo test -p gimp-sys -p gimp-ui-sys -p gimp-gexiv2-sys
```

## Testing in GIMP

```sh
tests/run-gimp-tests.sh
```

builds the example and test plug-ins, installs them in a temporary GIMP
profile and runs `tests/gimp_test.py` in a headless `gimp-console`. Set
`GIMP_RS_TEST_DIR` to reuse a profile.

CI (`.github/workflows/build.yml`) builds and tests on Linux (openSUSE
Tumbleweed container, including these GIMP tests) and Windows (MSYS2 UCRT64).

## Not (yet) bound

Mostly APIs gir cannot express, see the `ignore`/`manual` entries in the
`Gir.toml` files: `GimpVectorLoadProcedure`, the `Gimp.Param*` param specs,
GimpConfig (de)serialization via `GScanner`, the by-value `GimpVector2/3`
math functions, functions taking or returning `Babl` formats other than
`Drawable::format()`, and a few cairo helpers.
