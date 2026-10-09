# Changelog

## 0.1.0 — 2026-10-09

- Import Primary-HDU FITS grayscale and three-plane RGB with BITPIX 8/16/32/-32.
- Apply BSCALE/BZERO and ROWORDER; validate malformed headers and allocation limits.
- Preserve physical values in F32 documents; add removable exposure scaling for out-of-display-range images.
- Preserve HDR and tiny F32 samples in CPU exposure blending and GPU upload; avoid redundant F32 tile decoding.
- Independent Starframe FITS identity, original icon, isolated preferences and project support links.
- Windows x64 optimized builds with static C runtime, a per-user Japanese/English installer and a portable ZIP.
- Synthetic sample, licensing/attribution, SHA-256 checksums and reproducible release scripts.

Known limitations: unsigned Windows binaries; FITS export, WCS/astronomical metadata, general cubes/extensions/compression, BITPIX 64/-64, demosaicing and astronomical nonlinear auto-stretch are not implemented. Existing upstream alpha-stage editing/performance limitations remain.
