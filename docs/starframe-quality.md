# Release quality — Starframe FITS 0.1.0

## FITS and GPU validation

The importer/display implementation was validated before the distribution changes:

- 865 codec/I/O/text tests and 1816 corpus-enabled tests passed (counts overlap).
- 47 GPU tests passed, including full-float HDR exposure, paged uploads, sparse edge tiles, alpha, tiny/large values and CPU/GPU comparison.
- Malformed headers, truncation, duplicated keywords, non-finite/overflowing scales, unsupported layouts and allocation limits are rejected safely.
- Synthetic FITS cases and three public NASA sample arrays were compared against an independent Astropy oracle; 1,320,720 sample values matched.
- Two additional user-provided grayscale/RGB files were checked privately. No user images, FITS headers, original image filenames or screenshots of them are published here.
- Native import, saving/reopening, exposure editing and malformed-file failure without losing the existing document were verified.
- Formatting, clippy, dependency-layer checks, Wasm checks and panic-hunt validation passed for the importer work.

These counts are evidence for the FITS changes, not a claim that the entire application is feature-complete or fully validated on all hardware.

## Distribution changes

This fork isolates its identity, preferences, icon and support links. The installer is per-user and does not change FITS default applications. C runtime linkage is static; the installer/portable ZIP include licence notices and a synthetic demo only.

The Windows binaries are unsigned. Smart App Control or an enterprise application policy may refuse them. This cannot be solved by compression, a ZIP, checksums or an unsigned installer. Trusted signing remains a distribution limitation.

## Release gate

- [x] Original copyright/dual licence and third-party attribution retained.
- [x] ArtCraft mark/logo image assets removed; new original geometric icon.
- [x] README identifies the independent fork and names its exact upstream base.
- [x] Known FITS limitations and unsigned Windows restrictions described before download.
- [x] Installer script includes uninstall, isolated settings, optional desktop shortcut and Open-with registration.
- [x] Windows resources/version/icon embedded and static runtime linkage verified.
- [x] Fork UI/app regression tests and focused FITS tests passed.
- [x] Installer install/open/uninstall and portable startup smoke tests passed.
- [x] Synthetic-demo screenshot inspected for usable display.
- [x] Source/package checksums and private-image/credential exclusions verified.
- [ ] Published release downloads independently fetched and checked against local SHA256SUMS.

Final Windows checks on 2026-10-10: 52 native app tests, 730 UI tests (3 opt-in tests skipped), 16 codec FITS tests and 4 document FITS tests passed. All 28 dependency layers and 23 Wasm targets/configurations passed. The Windows CI build also passed. Wasm emits existing unused-constant warnings; this is not a browser distribution.

Installer and extracted ZIP both opened the synthetic FITS on the AMD Radeon hardware GPU with default settings, saved a layered document, and reopened it. The installer registered the Start Menu/Open-with entries without admin rights, excluded the portable marker, and removed its own entries on uninstall. PE imports contain Windows system DLLs and no external VCRUNTIME/MSVCP dependency. PDBs, credentials and private image data are not packaged. A final rebuild remaps personal build paths to `/build`; verification of that rebuild is pending.

The application tests cover the code in `a2dcf30`; final packaging adds compiler path sanitization. [Synthetic demo screenshot](images/starframe-demo.png).

The release notes record the completed gate. Windows x64 is the only binary distribution tested in this release. Existing upstream limitations in multi-layer editing/performance remain. WCS/header preservation, Bayer demosaicing, FITS export and nonlinear astronomical auto-stretch remain outside v1.
