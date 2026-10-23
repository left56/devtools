# Vendored dependency patches

## xattr-0.2.3

`gpui` 0.2.2 pulls `zed-async-tar` → `xattr 0.2`. On Linux, `libc` 0.2.191 no
longer exports `ENOATTR` (use `ENODATA`). This tree is stock 0.2.3 with that
one alias fixed so the Linux X11 build can link.

Remove this patch when upstream ships a compatible `xattr` 0.2.x or GPUI stops
depending on it.
