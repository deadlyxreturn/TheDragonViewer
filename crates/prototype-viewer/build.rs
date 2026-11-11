//! Build script: tells the viewer which cargo profile built it, for the
//! window title ("Prototype <version> (Dev)"): cargo only exposes `debug` /
//! `release` (PROFILE), while the profile's target directory names the
//! custom ones (`debugging`, `ci`). On Windows it also embeds the resources
//! of dragon-viewer.exe: its icon and its version block.

use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    // OUT_DIR = <target>/<profile dir>/build/<crate>-<hash>/out
    let out = std::env::var("OUT_DIR").unwrap_or_default();
    let parts: Vec<String> = Path::new(&out)
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    let dir = parts
        .iter()
        .rposition(|p| p == "build")
        .and_then(|i| i.checked_sub(1))
        .map(|i| parts[i].clone())
        .unwrap_or_default();
    println!("cargo:rustc-env=PROTOTYPE_BUILD_PROFILE={dir}");
    dictionaries();
    skin_files();

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        windows_resources();
        // The hybrid-GPU hints of main.rs (gpu_hints) in the exe's export
        // table, where the NVIDIA and AMD drivers look for them: a Rust
        // executable exports nothing on its own
        if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
            for symbol in ["NvOptimusEnablement", "AmdPowerXpressRequestHighPerformance"] {
                println!("cargo:rustc-link-arg-bin=dragon-viewer=/EXPORT:{symbol},DATA");
            }
        }
    }
}

/// The spelling dictionaries (assets/dictionaries/<name>/<name>.aff and
/// .dic, kept as text in the repository) gzipped into OUT_DIR, where
/// spelling/dictionaries.rs embeds them: 22 MB of words take about 5 MB of
/// the exe, inflated on the spelling thread only for the languages in use.
fn dictionaries() {
    use std::io::Write as _;
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("Cargo supplies CARGO_MANIFEST_DIR");
    let source = Path::new(&manifest).join("assets/dictionaries");
    println!("cargo:rerun-if-changed={}", source.display());
    let out = Path::new(&std::env::var("OUT_DIR").expect("Cargo supplies OUT_DIR")).join("dictionaries");
    std::fs::create_dir_all(&out).expect("the build folder is writable");
    let folders = std::fs::read_dir(&source).expect("assets/dictionaries is in the repository");
    for folder in folders.flatten().filter(|entry| entry.path().is_dir()) {
        for file in std::fs::read_dir(folder.path()).expect("a dictionary folder is readable").flatten() {
            let path = file.path();
            if !matches!(path.extension().and_then(|e| e.to_str()), Some("aff" | "dic")) {
                continue;
            }
            let bytes = std::fs::read(&path).expect("a dictionary file is readable");
            let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            gz.write_all(&bytes).expect("gzip in memory");
            let name = format!("{}.gz", path.file_name().and_then(|n| n.to_str()).expect("an ASCII file name"));
            std::fs::write(out.join(name), gz.finish().expect("gzip in memory")).expect("the build folder is writable");
        }
    }
}

/// The window sets its icon at run time (app.rs), but the file itself needs
/// an icon resource: without one, Explorer, shortcuts and the taskbar pin
/// show the generic executable icon. The version block comes with it:
/// Properties > Details, and the name Windows gives the program (Task
/// Manager, volume mixer, firewall prompts) is its FileDescription.
///
/// The version is the one prototype-version resolved from the Git tag (its
/// `links` metadata), never CARGO_PKG_VERSION. A failure stops the build: a
/// release must not ship without its icon again.
fn windows_resources() {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("Cargo supplies CARGO_MANIFEST_DIR");
    let icon = Path::new(&manifest).join("../../assets/branding/dragon-viewer.ico");
    println!("cargo:rerun-if-changed={}", icon.display());

    let display = std::env::var("DEP_PROTOTYPE_VERSION_DISPLAY").expect("prototype-version's build script supplies the version");
    let numeric = std::env::var("DEP_PROTOTYPE_VERSION_NUMERIC").expect("prototype-version's build script supplies the version");
    // "major.minor.patch.distance" packed as four 16-bit fields
    let packed = numeric
        .split('.')
        .chain(std::iter::repeat("0"))
        .take(4)
        .fold(0u64, |acc, part| (acc << 16) | part.parse::<u64>().unwrap_or(0).min(0xFFFF));

    let mut res = winresource::WindowsResource::new();
    res.set_icon(&icon.to_string_lossy())
        .set("ProductName", "Dragon Viewer")
        .set("FileDescription", "Dragon Viewer")
        .set("CompanyName", "Ryan Johnson")
        .set("LegalCopyright", "Dragon Viewer is based on Prototype by Ryan Johnson. GPL-3.0-or-later.")
        .set("OriginalFilename", "dragon-viewer.exe")
        .set("ProductVersion", &display)
        .set("FileVersion", &display)
        .set_version_info(winresource::VersionInfo::PRODUCTVERSION, packed)
        .set_version_info(winresource::VersionInfo::FILEVERSION, packed);
    // the viewer only: the test binaries of this crate need no resources
    if let Err(error) = res.compile_for("dragon-viewer") {
        panic!("Cannot embed the icon and version of dragon-viewer.exe (rc.exe of the Windows SDK): {error}");
    }
}

/// Dragon Viewer's skin (the repository's `skin/` folder: login background,
/// orbs, buttons, ornaments, icons) embedded in the exe, so a picture
/// committed there ships in the next build without any code change.
fn skin_files() {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("Cargo supplies CARGO_MANIFEST_DIR");
    let root = Path::new(&manifest).join("../../skin");
    println!("cargo:rerun-if-changed={}", root.display());
    let mut files = Vec::new();
    fn walk(dir: &Path, root: &Path, files: &mut Vec<(String, std::path::PathBuf)>) {
        let Ok(entries) = std::fs::read_dir(dir) else { return };
        let mut entries: Vec<_> = entries.flatten().collect();
        entries.sort_by_key(|e| e.file_name());
        for e in entries {
            let path = e.path();
            println!("cargo:rerun-if-changed={}", path.display());
            if path.is_dir() {
                walk(&path, root, files);
            } else if let Ok(rel) = path.strip_prefix(root) {
                let rel = rel.to_string_lossy().replace('\\', "/");
                if !rel.ends_with(".md") && !rel.starts_with('.') {
                    files.push((rel, path.canonicalize().unwrap_or(path)));
                }
            }
        }
    }
    walk(&root, &root, &mut files);
    let mut code = String::from("pub static SKIN_FILES: &[(&str, &[u8])] = &[\n");
    for (name, path) in files {
        code.push_str(&format!("    ({name:?}, include_bytes!({:?})),\n", path.display().to_string()));
    }
    code.push_str("];\n");
    let out = Path::new(&std::env::var("OUT_DIR").expect("Cargo supplies OUT_DIR")).join("skin_files.rs");
    std::fs::write(out, code).expect("the build folder is writable");
}
