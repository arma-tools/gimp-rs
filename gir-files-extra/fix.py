#!/usr/bin/env python3
"""Copy the GIR files of the system installation into this directory and
apply the fixups needed by `gir`.

Equivalent of gir-files' fix.sh, written in python because xmlstarlet is not
commonly available.

Usage: ./fix.py [GIR_DIR]   (GIR_DIR defaults to /usr/share/gir-1.0)

Babl-0.1.gir and Gegl-0.4.gir are taken from gegl-rs and are not touched here,
so the types match the published babl-sys/gegl-sys crates.
"""

import re
import shutil
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

NS = {
    "core": "http://www.gtk.org/introspection/core/1.0",
    "c": "http://www.gtk.org/introspection/c/1.0",
    "glib": "http://www.gtk.org/introspection/glib/1.0",
    "doc": "http://www.gtk.org/introspection/doc/1.0",
}
for prefix, uri in NS.items():
    ET.register_namespace("" if prefix == "core" else prefix, uri)

CORE = "{%s}" % NS["core"]
C = "{%s}" % NS["c"]
GLIB = "{%s}" % NS["glib"]

HERE = Path(__file__).resolve().parent
SRC = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("/usr/share/gir-1.0")
GIR_DIRS = [HERE, HERE.parent / "gir-files"]


def parents(root):
    return {child: parent for parent in root.iter() for child in parent}


def remove(root, xpath):
    parent_map = parents(root)
    for el in root.findall(xpath, NS):
        parent_map[el].remove(el)


def remove_callables_with_untyped_params(root):
    """`gir` fails hard on `<type>` elements without a name."""
    parent_map = parents(root)
    for tag in ("method", "function", "constructor", "virtual-method"):
        for el in list(root.iter(CORE + tag)):
            for t in el.iter(CORE + "type"):
                if "name" not in t.attrib:
                    if el in parent_map:
                        parent_map[el].remove(el)
                    break


class Namespace:
    """The C names of the items of a GIR namespace."""

    def __init__(self, root):
        ns = root.find("core:namespace", NS)
        self.name = ns.get("name")
        self.includes = {i.get("name"): i.get("version") for i in root.findall("core:include", NS)}
        self.types = {}  # name -> C type
        self.c_types = {}  # C type -> name
        self.callables = {}  # (type name or None, name) -> C function
        self.members = {}  # (enum name, member name) -> C identifier
        self.constants = {}  # name -> C identifier
        for el in ns:
            name = el.get("name")
            if el.tag == CORE + "constant":
                self.constants[name] = el.get(C + "type")
            elif el.tag == CORE + "function":
                self.callables[(None, name)] = el.get(C + "identifier")
            elif name:
                ctype = el.get(C + "type") or el.get(GLIB + "type-name")
                if ctype:
                    self.types[name] = ctype
                    self.c_types[ctype] = name
                for child in el:
                    if child.tag in (CORE + "method", CORE + "constructor", CORE + "function"):
                        self.callables[(name, child.get("name"))] = child.get(C + "identifier")
                    elif child.tag == CORE + "member":
                        self.members[(name, child.get("name"))] = child.get(C + "identifier")


class Namespaces:
    """The namespace being fixed and, loaded on demand, the ones it includes
    (directly or indirectly)."""

    def __init__(self, root):
        self.current = Namespace(root)
        self.loaded = {self.current.name: self.current}
        self.versions = dict(self.current.includes)

    def get(self, name):
        if name in self.loaded:
            return self.loaded[name]
        ns = None
        version = self.versions.get(name)
        if version:
            for d in GIR_DIRS:
                path = d / f"{name}-{version}.gir"
                if path.exists():
                    ns = Namespace(ET.parse(path).getroot())
                    for n, v in ns.includes.items():
                        self.versions.setdefault(n, v)
                    break
        self.loaded[name] = ns
        return ns


GI_DOCGEN_LINK = re.compile(
    r"\[(alias|callback|class|const|ctor|enum|error|flags|func|id|iface|method|property|signal|struct|type|vfunc)@([\w.:-]+)(?:#[\w-]+)?\]"
)


def gi_docgen_to_gtk_doc(namespaces, kind, target):
    """Converts a gi-docgen link to the gtk-doc syntax, e.g.
    `[method@Gimp.Image.get_width]` to `gimp_image_get_width()`."""
    member = None
    if "::" in target:
        target, member = target.split("::", 1)
        member = "::" + member
    elif ":" in target:
        target, member = target.split(":", 1)
        member = ":" + member

    parts = target.split(".")
    ns = namespaces.current
    if len(parts) > 1:
        other = namespaces.get(parts[0])
        if other:
            ns = other
            parts = parts[1:]
    # Some links use the C type name, like [vfunc@GimpPlugIn.quit].
    parts[0] = ns.c_types.get(parts[0], parts[0])

    converted = None
    if kind in ("property", "signal") and len(parts) == 1 and parts[0] in ns.types:
        converted = f"#{ns.types[parts[0]]}{member}"
    elif kind in ("method", "ctor", "func"):
        key = (parts[0], parts[1]) if len(parts) == 2 else (None, parts[0])
        if ns.callables.get(key):
            converted = f"{ns.callables[key]}()"
    elif kind == "const":
        converted = ns.constants.get(parts[0]) and f"%{ns.constants[parts[0]]}"
    elif kind == "id":
        converted = f"{target}()"
    elif len(parts) == 1:
        converted = ns.types.get(parts[0]) and f"#{ns.types[parts[0]]}"
    elif len(parts) == 2 and kind in ("enum", "flags", "error"):
        member_id = ns.members.get((parts[0], parts[1].lower()))
        converted = member_id and f"%{member_id}"

    if converted:
        return converted
    # Unbound, like vfuncs: as code, without the broken link.
    suffix = "()" if kind in ("method", "ctor", "func", "vfunc") else ""
    return f"`{target}{member or ''}{suffix}`"


def convert_gi_docgen_links(root):
    """The GIMP and GExiv2 docs mix gtk-doc (`#GimpImage`, `@param`) and
    gi-docgen (`[class@Gimp.Image]`) syntax, but the GIR files declare neither
    (`<doc:format name="unknown"/>`). gir then only understands gtk-doc, so
    the gi-docgen links are converted to it."""
    namespaces = Namespaces(root)
    for tag in ("doc", "doc-deprecated"):
        for doc in root.iter(CORE + tag):
            if doc.text:
                doc.text = GI_DOCGEN_LINK.sub(
                    lambda m: gi_docgen_to_gtk_doc(namespaces, m[1], m[2]), doc.text
                )


ULINK = re.compile(r'<ulink url="([^"]+)">(.*?)</ulink>', re.S)


def fix_gexiv2(root):
    remove_callables_with_untyped_params(root)
    convert_gi_docgen_links(root)

    # DocBook links, to markdown.
    for doc in root.iter(CORE + "doc"):
        if doc.text:
            doc.text = ULINK.sub(lambda m: f"[{m[2]}]({m[1]})" if m[2] else f"<{m[1]}>", doc.text)


CONSTANT_VALUES = {
    "CONFIG_PARAM_AGGREGATE": 2048,
    "CONFIG_PARAM_CONFIRM": 8192,
    "CONFIG_PARAM_DEFAULTS": 16384,
    "CONFIG_PARAM_DONT_COMPARE": 65536,
    "CONFIG_PARAM_FLAGS": 1255,
    "CONFIG_PARAM_FLAG_SHIFT": 17,
    "CONFIG_PARAM_IGNORE": 32768,
    "CONFIG_PARAM_RESTART": 4096,
    "CONFIG_PARAM_SERIALIZE": 1024,
    "PARAM_DONT_SERIALIZE": 512,
    "PARAM_FLAG_SHIFT": 10,
    "PARAM_NO_VALIDATE": 256,
    "PARAM_READABLE": 225,
    "PARAM_READWRITE": 227,
    "PARAM_WRITABLE": 226,
    "PARASITE_GRANDPARENT_PERSISTENT": 65536,
    "PARASITE_GRANDPARENT_UNDOABLE": 131072,
    "PARASITE_PARENT_PERSISTENT": 256,
    "PARASITE_PARENT_UNDOABLE": 512,
}


def fix_gimp(root):
    remove_callables_with_untyped_params(root)

    # #define GIMP_MAX_MEMSIZE ((guint64) 1 << 42), not a gint.
    for t in root.findall("core:namespace/core:constant[@name='MAX_MEMSIZE']/core:type", NS):
        t.set("name", "guint64")
        t.set(C + "type", "guint64")

    # GimpCpuAccelFlags is a set of flags (and contains 0x80000000, which
    # overflows an enum's c_int).
    for el in root.findall("core:namespace/core:enumeration[@name='CpuAccelFlags']", NS):
        el.tag = CORE + "bitfield"

    # `const GimpDrawable**` style object arrays: glib-rs can only marshal
    # `GimpDrawable**` (identical ABI).
    for arr in root.iter(CORE + "array"):
        ctype = arr.get(C + "type", "")
        if ctype.startswith("const Gimp") and ctype.endswith("**"):
            arr.set(C + "type", ctype[len("const "):])

    # The sensitivity mask is declared as gint, but takes
    # GimpProcedureSensitivityMask flags.
    for method in ("get_sensitivity_mask", "set_sensitivity_mask"):
        m = root.find(f"core:namespace/core:class[@name='Procedure']/core:method[@name='{method}']", NS)
        for t in m.findall("core:parameters/core:parameter/core:type", NS) + m.findall("core:return-value/core:type", NS):
            if t.get("name") == "gint":
                t.set("name", "ProcedureSensitivityMask")
                t.set(C + "type", "GimpProcedureSensitivityMask")

    # The out string is the procedure's own (`const gchar**`), but lacks a
    # `(transfer none)` annotation, so g-ir-scanner assumes transfer full.
    for p in root.findall("core:namespace/core:class[@name='FileProcedure']/core:method[@name='get_meta']/core:parameters/core:parameter[@name='meta_extensions']", NS):
        p.set("transfer-ownership", "none")

    # g-ir-scanner cannot evaluate macros built on G_PARAM_USER_SHIFT and
    # emits wrong values. These are the values the C compiler computes (the
    # gimp-sys ABI test `cross_validate_constants_with_c` checks them).
    for name, value in CONSTANT_VALUES.items():
        for el in root.findall(f"core:namespace/core:constant[@name='{name}']", NS):
            el.set("value", str(value))

    # Types from these headers are part of the GIR but not reachable via
    # libgimp/gimp.h; the ABI tests of gimp-sys need them.
    add_c_includes(root, ["libgimpmodule/gimpmodule.h", "libgimpbase/gimpparasiteio.h"])

    convert_gi_docgen_links(root)


def add_c_includes(root, headers):
    existing = root.findall("c:include", NS)
    index = list(root).index(existing[-1]) + 1
    for i, header in enumerate(headers):
        el = ET.Element(C + "include", {"name": header})
        root.insert(index + i, el)


def fix_gimp_ui(root):
    remove_callables_with_untyped_params(root)
    # The GIR advertises a pkg-config name that does not exist.
    for pkg in root.findall("core:package", NS):
        if pkg.get("name") == "gimp-ui-3.0":
            pkg.set("name", "gimpui-3.0")

    convert_gi_docgen_links(root)


FILES = {
    "GExiv2-0.10.gir": fix_gexiv2,
    "Gimp-3.0.gir": fix_gimp,
    "GimpUi-3.0.gir": fix_gimp_ui,
}


def main():
    for name, fix in FILES.items():
        dst = HERE / name
        shutil.copyfile(SRC / name, dst)
        tree = ET.parse(dst)
        fix(tree.getroot())
        ET.indent(tree, space="  ")
        tree.write(dst, encoding="UTF-8", xml_declaration=True)
        print(f"fixed {dst.name}")


if __name__ == "__main__":
    main()
