# Third-party notices

## Design reference

Dutchy web design system by skitsanos, MIT licensed:
https://github.com/skitsanos/dutchy-design-system

The colors, typography roles, component treatments, and behavior inform this
Rust implementation. No third-party widget/helper library code has been vendored.

## Bundled fonts

Source: Google Fonts repository revision
`24ddc1f324cac5d9f5aedca126d298f56e3f1640`.

| Source family | Distributed derivative | Original license |
| --- | --- | --- |
| Inter, by the Inter Project Authors | Dutchy Body | [Body-OFL.txt](assets/fonts/Body-OFL.txt) |
| Space Grotesk, by the Space Grotesk Project Authors | Dutchy Display | [Display-OFL.txt](assets/fonts/Display-OFL.txt) |
| JetBrains Mono, by the JetBrains Mono Project Authors | Dutchy Mono | [Mono-OFL.txt](assets/fonts/Mono-OFL.txt) |

All three use the SIL Open Font License 1.1. Original copyright and license
records are preserved. The variable upstream sources were instantiated into
upright static weights 400, 500, 600, and 700, then given distinct primary family,
full, and PostScript names. Inter optical size uses the source default.

Source paths, checksums, generated checksums, and the FontTools version are in
[manifest.json](assets/fonts/manifest.json). Reproduce with:

```sh
uv run --with fonttools==4.68.0 scripts/prepare_fonts.py
```

The script reuses the manifest's source revision. FontTools is a preparation
tool, not a runtime dependency. No network/font preparation is required to build
the package. Distributed font files may not be sold on their own; retain the
OFL notices with redistributed fonts.

## Crate dependencies

egui is the default direct runtime crate. Optional `url-images` adds ehttp
0.7.3 (HTTP transport) and image 0.25.10 (PNG/JPEG codecs). eframe and egui_kittest
are development-only. The file-selection showcase additionally uses
MIT-licensed rfd 0.17.2 as a development-only native picker. No rfd/widget helper
implementation is vendored. These crates use MIT OR Apache-2.0; their transitive
dependency licenses remain applicable. The loader integration is original code;
no widget/helper library implementation was copied.

## Online showcase photos

The optional Picsum mode requests images from https://picsum.photos/ at runtime.
No third-party photographs are bundled in this package. Visible captions credit
the photographers; the showcase links to their original Unsplash photo pages.
Provider metadata was verified on 2026-11-06 for IDs 10 (Paul Jarvis), 28 (Jerry
Adney), 237 (André Spieker), 870 (Joshua Hibbert), 1025 (Matthew Wiebe), and 1043
(Christian Joudrey). Original offline illustrations remain repository-owned.
Provider availability and photo rights remain subject to their source terms.
