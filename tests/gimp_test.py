# Executed by tests/run-gimp-tests.sh inside GIMP's python-fu-eval.
import os
import gi

gi.require_version("Gimp", "3.0")
gi.require_version("Gegl", "0.4")
from gi.repository import Gimp, Gio, Gegl

tmp = os.environ["GIMP_RS_TEST_DIR"]
pdb = Gimp.get_pdb()
failures = []


def check(name, condition, detail=""):
    print(("PASS " if condition else "FAIL ") + name + (f": {detail}" if detail else ""))
    if not condition:
        failures.append(name)


def run(name, **args):
    proc = pdb.lookup_procedure(name)
    config = proc.create_config()
    for key, value in args.items():
        if isinstance(value, list):
            config.set_core_object_array(key, value)
        else:
            config.set_property(key.replace("_", "-"), value)
    return proc.run(config)


for name in [
    "plug-in-gimp-rs-hello",
    "file-gimp-rs-rawrgba-load",
    "file-gimp-rs-rawrgba-export",
    "plug-in-gimp-rs-test-panic",
    "plug-in-gimp-rs-test-values",
]:
    check(f"registered {name}", pdb.procedure_exists(name))

image = Gimp.Image.new(3, 2, Gimp.ImageBaseType.RGB)
layer = Gimp.Layer.new(image, "bg", 3, 2, Gimp.ImageType.RGBA_IMAGE, 100, Gimp.LayerMode.NORMAL)
image.insert_layer(layer, None, 0)
Gimp.context_set_foreground(Gegl.Color.new("red"))
layer.fill(Gimp.FillType.FOREGROUND)

result = run("plug-in-gimp-rs-hello", run_mode=Gimp.RunMode.NONINTERACTIVE, image=image, drawables=[layer])
check("image procedure", result.index(0) == Gimp.PDBStatusType.SUCCESS)

path = os.path.join(tmp, "test.rawrgba")
file = Gio.File.new_for_path(path)
check("export", Gimp.file_save(Gimp.RunMode.NONINTERACTIVE, image, file, None))
data = open(path, "rb").read()
check("export header", data[:12] == b"RGBA\x03\x00\x00\x00\x02\x00\x00\x00", data[:12])
check("export pixels", data[12:] == bytes([255, 0, 0, 255]) * 6, list(data[12:]))

loaded = Gimp.file_load(Gimp.RunMode.NONINTERACTIVE, file)
check("load size", (loaded.get_width(), loaded.get_height()) == (3, 2))
pixels = loaded.get_layers()[0].get_buffer().get(
    Gegl.Rectangle.new(0, 0, 3, 2), 1.0, "R'G'B'A u8", Gegl.AbyssPolicy.NONE
)
check("load pixels", bytes(pixels) == bytes([255, 0, 0, 255]) * 6, list(pixels))

bad = os.path.join(tmp, "bad.rawrgba")
open(bad, "wb").write(b"RGBA\x05\x00\x00\x00\x05\x00\x00\x00abc")
result = run("file-gimp-rs-rawrgba-load", run_mode=Gimp.RunMode.NONINTERACTIVE, file=Gio.File.new_for_path(bad))
check("load error status", result.index(0) == Gimp.PDBStatusType.EXECUTION_ERROR, result.index(0))
check("load error message", "Expected 5x5 RGBA pixels" in (result.index(1) or ""), result.index(1))

result = run("plug-in-gimp-rs-test-panic")
check("panic status", result.index(0) == Gimp.PDBStatusType.EXECUTION_ERROR, result.index(0))
check("panic message", "test panic" in (result.index(1) or ""), result.index(1))
check("plug-in still usable after panic", run("plug-in-gimp-rs-test-values").index(0) == Gimp.PDBStatusType.SUCCESS)

result = run("plug-in-gimp-rs-test-values")
check("return values", result.index(0) == Gimp.PDBStatusType.SUCCESS and result.index(1) == 42,
      [result.index(i) for i in range(result.length())])

print("FAIL some" if failures else "PASS all")
