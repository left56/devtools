# Security patch overlay

Source: crates.io `grid` **1.0.1** (CVE-2026-42199 / GHSA-38c5-483c-4qqp fix).

`version` in Cargo.toml is set to **0.18.0** so `[patch.crates-io]` replaces
taffy 0.9's `grid = "0.18.0"` pin. Do not publish this path crate.
