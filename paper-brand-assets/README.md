# Paper brand assets

A small, flat identity kit for **Paper by Brekquater**. The mark combines a document page, a folded corner, text lines, and an insertion caret so the logo signals document editing rather than a generic file or notes app.

## What's included

- `logos/` — horizontal and stacked SVG lockups, wordmark-only versions, light/day/dark/monochrome variants, and PNG exports.
- `icons/` — scalable SVG app mark.
- `favicons/` — SVG favicon, 16/32/48/64/128 px PNGs, multi-size ICO, Safari pinned-tab SVG, and web manifest.
- `app-icons/` — light and dark app icon PNGs from 64 px through 1024 px, plus PWA, maskable and Apple touch icon files.
- `social/` — 1200×630 social preview image.
- `themes/` — CSS custom properties and JSON theme tokens for light, day, dark and system modes.
- `branding/` — color and typography tokens plus brand guidance.
- `examples/` — HTML head snippet and a React logo component example.

## Quick use

For a website header, use `logos/paper-logo-horizontal-light.svg` on a light background and `logos/paper-logo-horizontal-dark.svg` on a dark background. For a small UI surface, use `icons/paper-icon.svg`. For browser setup, copy the complete `paper-brand-assets/` directory to your app's public folder as `public/brand/`, preserving its subdirectories. The paths in `examples/html-head-snippet.html` then work as written.

## Theme intent

- **Light:** warm off-white canvas (`#F7F6F2`) and charcoal mark.
- **Day:** pure-white canvas and charcoal mark. This is intentionally a brighter light-theme option, not a separate logo design.
- **Dark:** charcoal canvas and white mark.
- **System:** use the host application's `prefers-color-scheme` decision and swap the light/dark asset as needed.

The logo files themselves are transparent. The background belongs to the host app, except for the app-icon PNGs which include a background.

## Brand colors

| Token | Hex | Use |
|---|---|---|
| Ink | `#17191C` | Primary mark and text |
| Muted | `#666A70` | Secondary text |
| Paper | `#F7F6F2` | Warm light background |
| White | `#FFFFFF` | Dark-mode logo and surfaces |
| Border | `#D9D8D3` | Subtle separators |
| Dark surface | `#15171A` | Dark-mode background |

## Logo usage

Keep clear space around the lockup at least equal to the icon's stroke width multiplied by four. Do not add gradients, glows, shadows, outlines, perspective, or extra decoration. Do not rearrange the folded corner, text lines, or caret. Use the icon alone only where the Paper wordmark is already nearby or where space is limited (favicon/app icon).

The smallest recommended use for the full horizontal lockup is 240 px wide. Below that, use the mark-only asset. The icon is designed to remain legible at 16 px, but verify it in the target browser and OS because favicon rendering varies.

## Technical notes

- SVG wordmarks use live text with the stack `Inter, Helvetica Neue, Arial, sans-serif`; no font files are bundled. Rendering may vary slightly if Inter is unavailable.
- For formal print work, open the SVG in a vector editor, select the approved font, and convert text to outlines before sending to a printer.
- `favicons/site.webmanifest` uses paths relative to its own location. Preserve the folder structure or update its paths before publishing.
- `maskable-512.png` is generated with the mark inside a safe inset. Test it with the target PWA launcher before release.
- These are brand assets only; they do not imply the product's editor APIs or package names are finalized.
