#!/bin/sh
# Regenerates the sys and high-level crates from the GIR files.
#
# To update the GIR files from the system installation first run
# `gir-files-extra/fix.py`.
set -e
cd "$(dirname "$0")"

cargo build --release --manifest-path gir/Cargo.toml
GIR="$PWD/gir/target/release/gir"

for crate in gexiv2/sys gexiv2 gimp/sys gimp gimp-ui/sys gimp-ui; do
    [ -f "$crate/Gir.toml" ] || continue
    echo "==> $crate"
    (cd "$crate" && "$GIR" -c Gir.toml)
done

cargo fmt --all
