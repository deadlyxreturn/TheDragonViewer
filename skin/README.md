# Dragon Viewer skin

Every picture here is built into `dragon-viewer.exe`. Replace a file (same name and folder) and commit: the next build uses it. PNG, JPEG, WebP and SVG are read; when two files share a name, the SVG is used first. A missing file falls back to the drawn look.

To try pictures without building, put them in a `skin` folder next to `dragon-viewer.exe` (same names). Those win over the built-in ones.

| File | Size | Where it shows |
|---|---|---|
| `background.png` | 2730 × 1536 | Login wallpaper |
| `login-crest.png` | 852 × 414 | On top of the login card |
| `cloud-scroll.png` | 572 × 341 | Lower corners of the login card |
| `gold-dragon.png` | 852 × 232 | Title bar ornament |
| `gold-flare.png` | 845 × 131 | Under window titles |
| `login-frame.svg/.png` | any | Face of the login card (stretched) |
| `login-button.svg/.png` | ~1140 × 246 | LOG IN button face, no text |
| `button-gold.svg/.png` | ~600 × 196 | OK and other main buttons, no text |
| `button-red.svg/.png` | ~600 × 196 | Cancel, no text |
| `balls/ball-1` … `ball-7` | 256 × 256 | The orbs: orb slider, slider knobs, title orbs, LOG IN ends |
| `messenger-bg.png` | 1600 × 1000 | Behind the messenger's glass |
| `icons/dragon-*.png` | 256 × 256 | Toolbar, messenger and window icons (see the icon list) |
| `app-icon.png` | 1024 × 1024 | The app icon: desktop shortcut, taskbar, installer (made into the .ico at build time) |
