# Asset attribution

Every non-code asset in this repository (images, icons, fonts, example files, presets) is listed here with its author, source and licence. `cargo xtask assets` (part of `cargo xtask ci`) fails if an asset file is missing from this table.

**Policy (mandatory):** Lattice contains **no Microsoft, Adobe, Avid or Autodesk iconography, images, artwork, fonts, templates, themes, table styles or sample files**. Every asset is original work by Lattice contributors or third-party material under an open licence (OSI open source, public domain / CC0, or Creative Commons that allows redistribution). Screenshots of Microsoft software are never committed. Font files are not added here: shared fonts live in [storytold/craft-fonts](https://github.com/storytold/craft-fonts), an optional build input (`CRAFT_FONTS_DIR`). The one exception is the Lattice brand files in `docs/brand/`: NimbusHQ trademarks, not open source, used under `docs/brand/LICENSE-brand.txt`.

Generated-in-code art is original and has no file to list: the UI icon set (`crates/ui-egui/src/icons.rs`), themes (`crates/engine/src/cmd/view.rs`), cell styles (`crates/engine/src/cmd/format.rs`), table styles (`crates/engine/src/tables.rs`), chart palettes and the sample workbooks (`crates/engine/src/sample.rs`).

| Asset | Author | Source | Licence | Notes |
|---|---|---|---|---|
| `docs/brand/lattice-logo.svg` | Lattice contributors | original (hand-written SVG; mark + wordmark) | Lattice brand terms (`docs/brand/LICENSE-brand.txt`) | PLACEHOLDER Lattice logo (light backgrounds); replaced by NimbusHQ brand art later |
| `docs/brand/lattice-logo-white.svg` | Lattice contributors | original (hand-written SVG; mark + wordmark) | Lattice brand terms (`docs/brand/LICENSE-brand.txt`) | PLACEHOLDER Lattice logo (dark backgrounds); replaced by NimbusHQ brand art later |
| `docs/brand/lattice-mark.svg` | Lattice contributors | original (hand-written SVG; woven lattice) | Lattice brand terms (`docs/brand/LICENSE-brand.txt`) | PLACEHOLDER Lattice mark; replaced by NimbusHQ brand art later |
| `docs/brand/lattice-mark-black.svg` | Lattice contributors | original (hand-written SVG; woven lattice) | Lattice brand terms (`docs/brand/LICENSE-brand.txt`) | PLACEHOLDER Lattice mark (one colour); replaced by NimbusHQ brand art later |
| `docs/brand/LICENSE-brand.txt` | (licence text) | original | — | |
| `assets/app-icon/lattice.svg` | Lattice contributors | original (hand-written SVG; `packaging/icons.sh`) | MIT OR Apache-2.0 (`assets/app-icon/LICENSE.txt`) | PLACEHOLDER app icon (ledger page); replaced by the owner's creature art later |
| `packaging/macos/dmg/background.svg` | @XusBadia | original (hand-written SVG, derived from the Lattice app icon `assets/app-icon/lattice.svg`, referenced via `<image>`, not copied; text is outlined glyph paths; `packaging/macos/dmg/generate.py` renders it to `background.tiff`) | MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`) | Lattice macOS DMG window background for the Finder window; added 2026-10-08 |
| `packaging/macos/dmg/background.tiff` | @XusBadia | Rendered from `packaging/macos/dmg/background.svg` by `packaging/macos/dmg/generate.py` | MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`) | 1x + 2x HiDPI TIFF for the macOS DMG window; includes the app icon (see its row) and LittleCMS's built-in sRGB profile ("No copyright, use freely") |
| `assets/app-icon/lattice-1024.png` | Lattice contributors | original (hand-written SVG; `packaging/icons.sh`) | MIT OR Apache-2.0 (`assets/app-icon/LICENSE.txt`) | Placeholder icon, rendered from lattice.svg |
| `assets/app-icon/lattice-macos-512.png` | Lattice contributors | original (hand-written SVG; `packaging/icons.sh`) | MIT OR Apache-2.0 (`assets/app-icon/LICENSE.txt`) | Placeholder icon, macOS grid render |
| `assets/app-icon/lattice.icns` | Lattice contributors | original (hand-written SVG; `packaging/icons.sh`) | MIT OR Apache-2.0 (`assets/app-icon/LICENSE.txt`) | Placeholder icon, macOS bundle icon |
| `assets/app-icon/lattice.ico` | Lattice contributors | original (hand-written SVG; `packaging/icons.sh`) | MIT OR Apache-2.0 (`assets/app-icon/LICENSE.txt`) | Placeholder icon, Windows exe icon |
| `assets/app-icon/hicolor/16x16/apps/app.lattice.nimbushq.xyz.png` | Lattice contributors | original (hand-written SVG; `packaging/icons.sh`) | MIT OR Apache-2.0 (`assets/app-icon/LICENSE.txt`) | Placeholder icon, Linux hicolor render |
| `assets/app-icon/hicolor/24x24/apps/app.lattice.nimbushq.xyz.png` | Lattice contributors | original (hand-written SVG; `packaging/icons.sh`) | MIT OR Apache-2.0 (`assets/app-icon/LICENSE.txt`) | Placeholder icon, Linux hicolor render |
| `assets/app-icon/hicolor/32x32/apps/app.lattice.nimbushq.xyz.png` | Lattice contributors | original (hand-written SVG; `packaging/icons.sh`) | MIT OR Apache-2.0 (`assets/app-icon/LICENSE.txt`) | Placeholder icon, Linux hicolor render |
| `assets/app-icon/hicolor/48x48/apps/app.lattice.nimbushq.xyz.png` | Lattice contributors | original (hand-written SVG; `packaging/icons.sh`) | MIT OR Apache-2.0 (`assets/app-icon/LICENSE.txt`) | Placeholder icon, Linux hicolor render |
| `assets/app-icon/hicolor/64x64/apps/app.lattice.nimbushq.xyz.png` | Lattice contributors | original (hand-written SVG; `packaging/icons.sh`) | MIT OR Apache-2.0 (`assets/app-icon/LICENSE.txt`) | Placeholder icon, Linux hicolor render |
| `assets/app-icon/hicolor/128x128/apps/app.lattice.nimbushq.xyz.png` | Lattice contributors | original (hand-written SVG; `packaging/icons.sh`) | MIT OR Apache-2.0 (`assets/app-icon/LICENSE.txt`) | Placeholder icon, Linux hicolor render |
| `assets/app-icon/hicolor/256x256/apps/app.lattice.nimbushq.xyz.png` | Lattice contributors | original (hand-written SVG; `packaging/icons.sh`) | MIT OR Apache-2.0 (`assets/app-icon/LICENSE.txt`) | Placeholder icon, Linux hicolor render |
| `assets/app-icon/hicolor/512x512/apps/app.lattice.nimbushq.xyz.png` | Lattice contributors | original (hand-written SVG; `packaging/icons.sh`) | MIT OR Apache-2.0 (`assets/app-icon/LICENSE.txt`) | Placeholder icon, Linux hicolor render |
| `assets/app-icon/hicolor/scalable/apps/app.lattice.nimbushq.xyz.svg` | Lattice contributors | original (hand-written SVG; `packaging/icons.sh`) | MIT OR Apache-2.0 (`assets/app-icon/LICENSE.txt`) | Placeholder icon, copy of lattice.svg |
| `assets/app-icon/README.md` | Lattice contributors | original | MIT OR Apache-2.0 | Icon documentation |
| `assets/app-icon/LICENSE.txt` | (licence text) | original | — | Licence of the app icon files |
| `docs/images/hero-sales.png` | Lattice contributors | screenshot of Lattice rendered offscreen (`crates/ui-egui/examples/snapshot.rs`) showing an original sample workbook | MIT OR Apache-2.0 | README screenshot; no third-party content |
| `docs/images/budget.png` | Lattice contributors | screenshot of Lattice rendered offscreen (`crates/ui-egui/examples/snapshot.rs`) showing an original sample workbook | MIT OR Apache-2.0 | README screenshot; no third-party content |
| `docs/images/grades-formulas.png` | Lattice contributors | screenshot of Lattice rendered offscreen (`crates/ui-egui/examples/snapshot.rs`) showing an original sample workbook | MIT OR Apache-2.0 | README screenshot; no third-party content |
| `docs/images/dark-insert.png` | Lattice contributors | screenshot of Lattice rendered offscreen (`crates/ui-egui/examples/snapshot.rs`) showing an original sample workbook | MIT OR Apache-2.0 | README screenshot; no third-party content |
| `docs/images/pivot.png` | Lattice contributors | screenshot of Lattice rendered offscreen (`crates/ui-egui/examples/snapshot.rs`) showing an original sample workbook | MIT OR Apache-2.0 | README screenshot; no third-party content |
| `docs/images/format-chart.png` | Lattice contributors | screenshot of Lattice rendered offscreen (`crates/ui-egui/examples/snapshot.rs`) showing an original sample workbook | MIT OR Apache-2.0 | README screenshot; no third-party content |
| `docs/images/page-break.png` | Lattice contributors | screenshot of Lattice rendered offscreen (`crates/ui-egui/examples/snapshot.rs`) showing an original sample workbook | MIT OR Apache-2.0 | README screenshot; no third-party content |

## Shipped with desktop releases (not in this repository)

The built-in assistant (`docs/assistant.md`) uses third-party files that are never committed here. Release packaging bundles the runtime; the models are downloaded by the app on the user's request and verified against pinned SHA-256 hashes (`crates/assistant/src/models.rs`).

| Component | Author | Source | Licence | Notes |
|---|---|---|---|---|
| `llama-server` and its `libggml*`/`libllama*`/`libmtmd` libraries | The ggml / llama.cpp authors | [ggml-org/llama.cpp](https://github.com/ggml-org/llama.cpp) release b11555 | MIT (its `LICENSE` ships beside the binary) | The assistant's local runtime sidecar; bundled by `packaging/` |
| `lattice-e4b-Q4_K_M.gguf`, `lattice-e2b-Q4_K_M.gguf` | Lattice contributors (fine-tunes); Google (base models) | fine-tuned from Google's Gemma 4 E4B / E2B | Gemma 4 model licence | Downloaded in-app, never bundled or committed; training data includes CC-BY-SA material (attribution in the training repository) |
