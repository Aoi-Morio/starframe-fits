# Windows release build

Requirements: Rust 1.95.0 with `x86_64-pc-windows-msvc`, Visual Studio C++ Build Tools and Windows SDK, Inno Setup 6.7.3.

```powershell
cargo fmt --all --check
cargo test -p photocraft -p photocraft-ui-egui --lib --bins
./packaging/starframe/build.ps1 -Iscc 'C:/Program Files (x86)/Inno Setup 6/ISCC.exe'
```

The script uses optimized release code (`opt-level=3`, no LTO, 16 codegen units) and `+crt-static` so no separately installed VC++ runtime is needed. It remaps the builder's user-folder paths to `/build` and rejects executables containing the original path before packaging. Windows icon/version resources are required. For a portable SDK, set `RC_PATH` to its `x64/rc.exe`.
Generated files are under `dist/`. Do not package `StarframeData`, private images, credentials, PDBs or user preferences.
Only the synthetic `examples/fits/demo-stars.fits` is included.

The installer is per-user, does not require elevation, adds a Start Menu shortcut and optional desktop shortcut, and offers FITS under “Open with” without taking over the user's default application. Uninstall leaves user images/settings intact.
The installer excludes `portable.txt`, so installed settings go to `%APPDATA%/StarframeFITS`; the portable ZIP includes it and uses `StarframeData` beside the executable.

The inherited upstream multi-platform release scripts are retained as upstream reference material, not this fork's supported distribution pipeline. Use this directory and `.github/workflows/starframe.yml`.
This release is unsigned. Trusted Windows code signing is a separate requirement for unrestricted deployment to PCs enforcing Smart App Control. No security-disable workaround is part of this project.

Before another release: bump the workspace version, update installer/readme download filenames and changelog, run the tests and installer smoke test, confirm every checksum, then tag the tested commit. Never overwrite an already published version's assets.
