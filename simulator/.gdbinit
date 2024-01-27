python

# NOTE: Sourced from ~/.rustup/toolchains/esp/lib/rustlib/etc/gdb_load_rust_pretty_printers.py
# with some small changes related to setting self_dir

# Add this folder to the python sys path; GDB Python-interpreter will now find modules in this path
import sys
from os import path, environ

self_dir = path.dirname(path.realpath(f"{environ['HOME']}/.rustup/toolchains/esp/lib/rustlib/etc/gdb_load_rust_pretty_printers.py"))
sys.path.append(self_dir)

# ruff: noqa: E402
import gdb
import gdb_lookup

# current_objfile can be none; even with `gdb foo-app`; sourcing this file after gdb init now works
try:
    gdb_lookup.register_printers(gdb.current_objfile())
except Exception:
    gdb_lookup.register_printers(gdb.selected_inferior().progspace)

end

# NOTE: When using rr, it always asks if debuginfod should be enabled.
# To stop that spam, just set it to on automatically.
set debuginfod enabled on
