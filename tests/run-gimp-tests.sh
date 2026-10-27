#!/bin/sh
# Runs the example and test plug-ins in a headless GIMP with a temporary
# profile (the user's GIMP configuration is not touched).
#
set -e
cd "$(dirname "$0")/.."

cargo build -p gimp-rs-examples

TMP=${GIMP_RS_TEST_DIR:-$(mktemp -d)}
for plugin in gimp-rs-example gimp-rs-test-plugin; do
    mkdir -p "$TMP/profile/plug-ins/$plugin"
    cp "target/debug/$plugin" "$TMP/profile/plug-ins/$plugin/"
done

GIMP_CONSOLE=${GIMP_CONSOLE:-$(command -v gimp-console-3 || command -v gimp-console)}
GIMP3_DIRECTORY="$TMP/profile" GIMP_RS_TEST_DIR="$TMP" "$GIMP_CONSOLE" \
    -i -n --quit --batch-interpreter=python-fu-eval \
    -b "exec(open('tests/gimp_test.py').read())" \
    2>&1 | tee "$TMP/log.txt" | grep -E '^(PASS|FAIL)'

! grep -q '^FAIL' "$TMP/log.txt" && grep -q '^PASS all' "$TMP/log.txt"
