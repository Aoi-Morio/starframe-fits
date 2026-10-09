# FITS image import v1

PhotoCraft opens `.fits`, `.fit` and `.fts` in File > Open, drag and drop, the CLI,
and the Web app using the existing image import path. Files are identified by the
`SIMPLE` signature, so renamed files also work. No plug-in is required.

## Supported data

- A SIMPLE=T primary HDU containing a nonempty 2D grayscale image, or a
  three-plane RGB image (NAXIS=3, NAXIS3=3, consecutive R/G/B planes).
- BITPIX 8 (unsigned), 16 and 32 (signed), and -32 (IEEE float), big endian.
- BSCALE and BZERO, including the unsigned-16 BZERO=32768 convention and D exponents.
- 80-byte cards and 2880-byte header/data blocks; bounded multi-block headers.
- A 32-bit float grayscale or RGB document with an opaque Background layer.
- Explicit ROWORDER='TOP-DOWN'/'BOTTOM-UP' display orientation.

Physical samples are computed in f64 as `stored * BSCALE + BZERO` and rounded
once to f32. The decoder does not normalize, clamp, or stretch valid samples.
Negative values and values above one are retained in the Background layer. If the
positive peak is above one or below 0.001, document import adds a visible, removable
Exposure adjustment named "FITS display scaling". It scales by a power of two so
detector counts and very small physical values display in range, without changing
the Background samples. Hide/remove it to view the original composite. Layered
saves retain it; flat exports bake the visible adjustments. This is simple range
scaling, not a background-subtracting or nonlinear astronomical stretch.
Apply an appropriate exposure/levels adjustment for editing, or stretch externally
before import if a display-ready image is needed. This is an image editor import,
not a replacement for scientific FITS analysis software.

FITS axis 1 varies fastest. Axis 2 is mapped from FITS bottom-to-top to PhotoCraft
top-to-bottom unless ROWORDER specifies TOP-DOWN. Three-plane arrays are interpreted
as RGB even without CTYPE3, matching common Siril/GraXpert output; generic three-slice
scientific cubes are not automatically distinguished from RGB. WCS is not interpreted.
Undefined integer BLANK samples and floating
NaN/positive or negative infinity become zero, with a warning counting replacements.
Finite samples that overflow f32 after scaling fail import rather than silently clipping.
Rounding and additional HDUs produce warnings. Routine successful imports do not
post a general informational warning; astronomical metadata is not retained.

## Limits

No FITS export, general multidimensional cubes, empty-primary/image-extension selection, tables, random
groups, BITPIX 64/-64, gzip/tile compression, WCS, astronomical header preservation,
checksum validation, Bayer demosaic, or automatic display stretch. HDUs following a
valid primary image are ignored with a warning. Headers must have the mandatory
cards in FITS order, printable ASCII, a valid END and complete block padding.
Interpretation-affecting duplicate keywords and invalid scaling/BLANK values fail
with a codec error. The header scan is capped at 1 MiB; the existing codec pixel,
dimension and decoded-allocation limits apply before allocation.

Save the edited document as `.pcraft`, PSD or 32-bit float TIFF. File > Save As
offers supported output formats; attempting a FITS export returns Unsupported.
f32 cannot represent all signed 32-bit integer values exactly (integers above 2^24
may round), nor all f64-scaled values. FITS is not a scientific lossless round trip.

## Implementation and validation

The decoder is in `crates/codecs/src/codecs/fits.rs`, with no new runtime dependency,
filesystem access or unsafe code. Format capabilities explicitly declare the read-only
exception. Existing `photocraft-io` converts the image to the document; the native and
Web open filters include all three extensions. Existing open commands remain the entrypoint.

`crates/codecs/tests/fits.rs` covers all supported sample types, scaling, endianness,
orientation, undefined values, rounding, multi-block headers, duplicate/invalid cards,
unsupported types/dimensions, limits, every truncation of a sample, random bytes and
an independent signed-16 scaling property oracle. `crates/io/tests/fits.rs` covers
document values and save/reopen in `.pcraft`, TIFF and PSD. A `decode_fits` cargo-fuzz
target is provided; a bounded property test does not substitute for a prolonged fuzz run.

Out-of-range display scaling also exercises the existing compositor. Full-opacity
adjustments assign their result directly to avoid cancellation when detector units
are much larger than the displayed value. Regression tests cover Gray/RGB, clipped
and ordinary adjustments, large integer counts and tiny floating-point values.
The GPU compositor likewise avoids cancellation at full opacity and uploads F32
surfaces as RGBA32F instead of half float, preserving their range and precision.
GPU parity tests cover whole textures and pages; float resident texture storage
uses 16 bytes per pixel, reflected in the existing memory accounting.
Exposure editors preserve the imported finite range (bounded to ±126 stops) when
another parameter is edited; newly created adjustments retain the usual ±20 range.

Implementation follows the public [IAU FITS standard](https://fits.gsfc.nasa.gov/fits_standard.html).
RGB and ROWORDER conventions are documented by [Siril](https://siril.readthedocs.io/en/latest/file-formats/FITS.html).
External test samples should stay in an ignored corpus; do not commit third-party files
without confirmed redistribution rights and attribution.

## Before publication / PR

- [x] Review final diff; unrelated baseline lint/format fixes are supplied separately.
- [x] `cargo test -p photocraft-codecs -p photocraft-io`.
- [x] `cargo clippy -p photocraft-codecs -p photocraft-io --all-targets -- -D warnings`.
- [x] `cargo fmt --all -- --check`, `cargo xtask layers`, `cargo xtask wasm`.
- [x] `cargo xtask test-corpus` for existing codec/PSD regressions.
- [x] Open representative real FITS in the native app; inspect depth, orientation,
      warnings and pixels, save/reopen, and capture a screenshot.
- [x] Validate Web build and open filter. Browser interaction is still a follow-up.
- [ ] Run a prolonged `decode_fits` fuzz session and Web browser interaction checks.
- [x] Measure 24 MP release import timing: Gray median 0.342 s, RGB 0.955 s on this PC.
      Output buffers are 96 MB / 288 MB; process peak memory is not measured. The base has no FITS import for a same-format comparison.
- [ ] Measure whole-process peak memory and obtain a matching-machine performance baseline.
- [x] Confirm contribution license MIT OR Apache-2.0; include required notices if distributing.
- [ ] Review upstream changes and any accessible common `craftrules` before submitting.
- [ ] Attach validation evidence and clearly state the limits above in the PR.

Checked items record local validation on 2026-10-08. The corpus and quick-performance
release runs use LTO disabled and codegen-units=16; the 24 MP FITS timing used the
normal release profile. Native checks used both the CPU compatibility compositor
and the AMD Radeon/Vulkan GPU canvas on this PC; other adapters remain untested.
Unchecked items remain explicit follow-ups before broader publication.
