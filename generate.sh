#!/bin/sh
# Regenerates the sys and high-level crates from the GIR files and embeds the
# GIR documentation into the generated code of the high-level crates.
#
# To update the GIR files from the system installation first run
# `gir-files-extra/fix.py`.
#
# Embedding the documentation requires rustdoc-stripper
# (`cargo install rustdoc-stripper`).
set -e
cd "$(dirname "$0")"

if ! command -v rustdoc-stripper >/dev/null; then
    echo "rustdoc-stripper not found, run \`cargo install rustdoc-stripper\`" >&2
    exit 1
fi

cargo build --release --manifest-path gir/Cargo.toml
GIR="$PWD/gir/target/release/gir"

for crate in gexiv2/sys gexiv2 gimp/sys gimp gimp-ui/sys gimp-ui; do
    [ -f "$crate/Gir.toml" ] || continue
    echo "==> $crate"
    (cd "$crate" && "$GIR" -c Gir.toml)

    case "$crate" in
    */sys) ;;
    *)
        echo "==> $crate: docs"
        # gir writes the docs of all generated items to docs.md,
        # rustdoc-stripper inserts them as doc comments (and deletes
        # docs.md). Only into src/auto/, the hand-written code is documented
        # by hand.
        (cd "$crate" &&
            "$GIR" -c Gir.toml -m doc --doc-target-path docs.md &&
            rustdoc-stripper -g -m -x -d src/auto -o docs.md)
        ;;
    esac
done

cargo fmt --all
