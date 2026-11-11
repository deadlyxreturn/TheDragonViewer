<div align="center">
<img src="assets/branding/dragon-viewer-icon.png" alt="Dragon Viewer" width="160">
</div>

# Dragon Viewer

Dragon Viewer is an open-source Windows viewer for Second Life derived from
[Prototype](https://github.com/deadlyxreturn/Prototype) and Moon Viewer by
Ryan Johnson. It has deep night glass with gold rims, blazing orange
buttons and tabs, and an orb slider, from the login screen to Preferences,
Conversations and the other windows.

## Skin

Every picture of the interface comes from the [`skin`](skin/README.md)
folder: the login wallpaper, the orbs, the button faces, the ornaments of
the login card, the messenger background and the icons. Replace a file
there (same name, PNG or SVG) and commit: the next build ships it. The
viewer also reads a `skin` folder next to `dragon-viewer.exe`, to try
pictures without building.

This software is not provided or supported by Linden Lab, the makers of Second Life.

## Downloads and support

- Verified packages, when available, appear on the [releases page](https://github.com/deadlyxreturn/TheDragonViewer/releases).
- Report bugs and ask for help through [GitHub Issues](https://github.com/deadlyxreturn/TheDragonViewer/issues). There is no dedicated customer support service.
- Review the [privacy policy](PRIVACY.md) and [viewer policy notes](docs/DRAGON-VIEWER-POLICY.md) before installing. The viewer retains Prototype's optional RLVa controls, third-party grid support and link previews. Imported media can depend on external plugins, screen recording needs FFmpeg, and OpenXR needs a compatible runtime and headset.
- The Windows installer is per user. It uses `%LOCALAPPDATA%\Programs\DragonViewer` by default and adds a Start menu entry and optional desktop shortcut. Windows Settings can uninstall it. The installer is unsigned.

## Build on Windows x64

Use a current Vulkan graphics driver and Visual Studio 2022 Build Tools with
the C++ workload and Windows SDK. Install Rustup with the
`x86_64-pc-windows-msvc` host, plus CMake and NSIS. The repository pins
Rust 1.99.1. Run the build script from a Developer PowerShell with Cargo,
CMake and MakeNSIS on the path.

```powershell
./scripts/build-release.ps1 -Tag v0.1.0
```

Cargo uses the checked-in lockfile. The build script places the installer,
portable ZIP, update policy and SHA-256 files under
`target/dragon-viewer-dist/v0.1.0`. The same script runs in the Windows GitHub
Actions workflow. See [asset provenance](assets/branding/DRAGON-VIEWER.md)
for where the pictures come from.

The local viewer build and test commands use the CI profile.

```powershell
cargo build --locked --profile ci -p prototype-viewer --bin dragon-viewer
cargo test --locked --profile ci --workspace
```

The package keeps the inherited Cargo crate names. The viewer executable
is `dragon-viewer.exe`. The local build above uses the system OpenXR loader
when available. The Windows release script builds with the static loader.

## Licence and source

Dragon Viewer is distributed under the GNU General Public License version 3
or later. Moon Viewer started at Prototype revision
[`eede23767d`](https://github.com/deadlyxreturn/Prototype/commit/eede23767d4421f182ee4f0ab60f7eccaf417cbd).
The [GPL text](LICENSE) and [original credits and notices](NOTICE.md)
document its lineage and third-party components.

Second Life® is a trademark of Linden Research, Inc.
