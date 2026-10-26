# Vendored dependency patches

## xattr-0.2.3

`gpui` 0.2.2 pulls `zed-async-tar` → `xattr 0.2`. On Linux, `libc` 0.2.191 no
longer exports `ENOATTR` (use `ENODATA`). This tree is stock 0.2.3 with that
one alias fixed so the Linux X11 build can link.

Remove this patch when upstream ships a compatible `xattr` 0.2.x or GPUI stops
depending on it.

## grid-1.0.1

`gpui` 0.2.2 → `taffy` 0.9.0 pins `grid` 0.18.0, which is in the
CVE-2026-42199 / GHSA-38c5-483c-4qqp range (`>=0.17.0, <=1.0.0`). Stock
`grid` 1.0.1 adds overflow checks in `expand_rows` / dimension math.

Remove this patch when taffy/gpui depend on `grid >= 1.0.1`.
