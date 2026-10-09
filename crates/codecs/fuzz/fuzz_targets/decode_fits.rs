#![no_main]
//! Explicit FITS decode reaches malformed headers even without a valid signature.
use libfuzzer_sys::fuzz_target;
use photocraft_codecs::{DecodeOptions, Format, Limits, decode_as_with};

fuzz_target!(|bytes: &[u8]| {
    let opts = DecodeOptions { limits: Limits { max_width: 4096, max_height: 4096, max_pixels: 1 << 22, max_alloc: 64 << 20 }, ..Default::default() };
    let _ = decode_as_with(Format::Fits, bytes, &opts);
});
