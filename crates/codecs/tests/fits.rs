//! FITS spec examples generated independently of the decoder, plus hostile inputs.
use photocraft_codecs::*;
use proptest::prelude::*;

fn card(key: &str, value: &str) -> String {
    format!("{key:8}= {value:>20}")
}

fn fixture(bitpix: i32, w: u32, h: u32, extra: &[String], data: &[u8]) -> Vec<u8> {
    fixture_planes(bitpix, w, h, None, extra, data)
}

fn fixture_planes(bitpix: i32, w: u32, h: u32, planes: Option<u32>, extra: &[String], data: &[u8]) -> Vec<u8> {
    let mut cards = vec![
        card("SIMPLE", "T"),
        card("BITPIX", &bitpix.to_string()),
        card("NAXIS", if planes.is_some() { "3" } else { "2" }),
        card("NAXIS1", &w.to_string()),
        card("NAXIS2", &h.to_string()),
    ];
    if let Some(n) = planes {
        cards.push(card("NAXIS3", &n.to_string()));
    }
    cards.extend_from_slice(extra);
    cards.push("END".into());
    let mut b = Vec::new();
    for c in cards {
        assert!(c.len() <= 80);
        b.extend_from_slice(format!("{c:80}").as_bytes());
    }
    b.resize(b.len().div_ceil(2880) * 2880, b' ');
    b.extend_from_slice(data);
    b.resize(b.len().div_ceil(2880) * 2880, 0);
    b
}

#[test]
fn rgb_planes_are_interleaved_scaled_and_oriented_for_all_sample_types() {
    for bitpix in [8, 16, 32, -32] {
        let values = [1u8, 2, 3, 4, 11, 12, 13, 14, 21, 22, 23, 24];
        let raw: Vec<u8> = values
            .into_iter()
            .flat_map(|v| match bitpix {
                8 => vec![v],
                16 => i16::from(v).to_be_bytes().to_vec(),
                32 => i32::from(v).to_be_bytes().to_vec(),
                _ => f32::from(v).to_be_bytes().to_vec(),
            })
            .collect();
        let b = fixture_planes(bitpix, 2, 2, Some(3), &[card("BSCALE", "2"), card("BZERO", "-1")], &raw);
        let image = decode(&b).unwrap();
        assert_eq!(image.layout(), ChannelLayout::Rgb);
        assert_eq!(image.sample_type(), SampleType::F32);
        assert_eq!(floats(&image), [5., 25., 45., 7., 27., 47., 1., 21., 41., 3., 23., 43.]);
        let limits = Limits { max_alloc: 47, ..Limits::default() };
        assert!(matches!(decode_with(&b, &DecodeOptions { limits, ..Default::default() }), Err(CodecError::LimitExceeded(_))));
        assert!(decode(&b[..b.len() - 1]).is_err());
    }
}

#[test]
fn rgb_requires_exactly_three_planes_and_rejects_duplicate_axes() {
    for n in [0, 1, 2, 4, u32::MAX] {
        assert!(matches!(decode(&fixture_planes(8, 1, 1, Some(n), &[], &[1; 3])), Err(CodecError::Unsupported { .. })));
    }
    assert!(decode(&fixture_planes(8, 1, 1, Some(3), &[card("NAXIS3", "3")], &[1; 3])).is_err());
    assert!(decode(&fixture(8, 1, 1, &[card("NAXIS3", "3")], &[1])).is_err());
    let mut missing = fixture_planes(8, 1, 1, Some(3), &[], &[1; 3]);
    missing[400..480].fill(b' ');
    assert!(decode(&missing).is_err());
}

#[test]
fn explicit_row_order_controls_display_orientation() {
    for (order, expected) in [("'TOP-DOWN '", [1., 2., 3., 4.]), ("'BOTTOM-UP'", [3., 4., 1., 2.])] {
        assert_eq!(floats(&decode(&fixture(8, 2, 2, &[card("ROWORDER", order)], &[1, 2, 3, 4])).unwrap()), expected);
        let rgb = decode(&fixture_planes(8, 1, 2, Some(3), &[card("ROWORDER", order)], &[1, 2, 3, 4, 5, 6])).unwrap();
        assert_eq!(floats(&rgb), if order.contains("TOP") { vec![1., 3., 5., 2., 4., 6.] } else { vec![2., 4., 6., 1., 3., 5.] });
    }
    for extra in
        [vec![card("ROWORDER", "'UNKNOWN'")], vec![card("ROWORDER", "TOP-DOWN")], vec![card("ROWORDER", "'TOP-DOWN'"), card("ROWORDER", "'BOTTOM-UP'")]]
    {
        assert!(decode(&fixture(8, 1, 1, &extra, &[1])).is_err());
    }
}

fn floats(img: &Image) -> Vec<f32> {
    img.data().chunks_exact(4).map(|p| f32::from_ne_bytes(p.try_into().unwrap())).collect()
}

#[test]
fn all_sample_types_are_big_endian_scaled_float_gray_and_bottom_up() {
    for (bitpix, raw, expected) in [
        (8, vec![0, 127, 128, 255], vec![257.0, 511.0, 1.0, 255.0]),
        (16, [-32768i16, -1, 0, 32767].into_iter().flat_map(i16::to_be_bytes).collect(), vec![1.0, 65535.0, -65535.0, -1.0]),
        (32, [-1_000_000i32, -1, 0, 1_000_000].into_iter().flat_map(i32::to_be_bytes).collect(), vec![1.0, 2_000_001.0, -1_999_999.0, -1.0]),
        (-32, [-1.0f32, 0.25, 0.5, 2.0].into_iter().flat_map(f32::to_be_bytes).collect(), vec![2.0, 5.0, -1.0, 1.5]),
    ] {
        let bytes = fixture(bitpix, 2, 2, &[card("BSCALE", "2.0D+00 / scale"), card("BZERO", "1")], &raw);
        assert_eq!(detect(&bytes), Some(Format::Fits));
        let img = decode(&bytes).unwrap();
        assert_eq!(img.dimensions(), (2, 2));
        assert_eq!(img.layout(), ChannelLayout::Gray);
        assert_eq!(img.sample_type(), SampleType::F32);
        assert_eq!(floats(&img), expected, "BITPIX={bitpix}");
    }
}

#[test]
fn unsigned_16_convention_and_defaults() {
    let raw: Vec<_> = [-32768i16, 0, 32767].into_iter().flat_map(i16::to_be_bytes).collect();
    let img = decode(&fixture(16, 3, 1, &[card("BZERO", "32768")], &raw)).unwrap();
    assert_eq!(floats(&img), [0.0, 32768.0, 65535.0]);
    assert_eq!(floats(&decode(&fixture(8, 2, 1, &[], &[0, 255])).unwrap()), [0.0, 255.0]);
}

#[test]
fn undefined_samples_never_enter_the_document() {
    let raw: Vec<_> = [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 0.75].into_iter().flat_map(f32::to_be_bytes).collect();
    let img = decode(&fixture(-32, 4, 1, &[], &raw)).unwrap();
    assert_eq!(floats(&img), [0.0, 0.0, 0.0, 0.75]);
    assert!(img.warnings.contains(&DecodeWarning::FitsUndefined { pixels: 3 }));
    let img = decode(&fixture(8, 2, 1, &[card("BLANK", "255"), card("BZERO", "100")], &[255, 1])).unwrap();
    assert_eq!(floats(&img), [0.0, 101.0]);
    assert!(img.warnings.contains(&DecodeWarning::FitsUndefined { pixels: 1 }));
}

#[test]
fn precision_and_additional_hdus_are_reported() {
    let mut bytes = fixture(32, 1, 1, &[], &16_777_217i32.to_be_bytes());
    bytes.extend_from_slice(&[0; 2880]);
    let img = decode(&bytes).unwrap();
    assert_eq!(floats(&img), [16_777_216.0]);
    assert!(img.warnings.contains(&DecodeWarning::FitsPrecisionReduced));
    assert!(img.warnings.contains(&DecodeWarning::FitsExtensionsIgnored));
}

#[test]
fn extensions_caps_and_read_only_contract() {
    for ext in ["fits", "FIT", "x.FTS"] {
        assert_eq!(from_extension(ext), Some(Format::Fits));
    }
    assert!(caps(Format::Fits).read);
    assert!(!caps(Format::Fits).write);
    let img = decode(&fixture(8, 1, 1, &[], &[1])).unwrap();
    assert!(matches!(encode(&img, Format::Fits, &EncodeOptions::default()), Err(CodecError::Unsupported { .. })));
}

#[test]
fn multi_block_header_and_unknown_cards() {
    let extra = vec!["COMMENT public specification fixture".to_string(); 80];
    assert_eq!(floats(&decode(&fixture(8, 1, 1, &extra, &[42])).unwrap()), [42.0]);
}

#[test]
fn overlong_header_is_rejected_with_a_bounded_scan() {
    let extra = vec!["COMMENT header size regression".to_string(); 14_000];
    assert!(decode(&fixture(8, 1, 1, &extra, &[42])).is_err());
}

#[test]
#[cfg(not(debug_assertions))]
#[ignore = "24 MP release performance measurement"]
fn fits_24mp_import_timing() {
    let raw: Vec<_> = (0..24_000_000u32).flat_map(|i| ((i % 65536) as i32 - 32768).to_be_bytes().into_iter().skip(2)).collect();
    let bytes = fixture(16, 6000, 4000, &[card("BZERO", "32768")], &raw);
    for run in 0..3 {
        let start = std::time::Instant::now();
        let image = decode(&bytes).unwrap();
        let elapsed = start.elapsed();
        assert_eq!(image.data().len(), 96_000_000);
        println!("24 MP FITS decode run {run}: {elapsed:?}; input {} bytes; output {} bytes", bytes.len(), image.data().len());
        std::hint::black_box(image);
    }
    let rgb = fixture_planes(16, 6000, 4000, Some(3), &[card("BZERO", "32768")], &raw.repeat(3));
    for run in 0..3 {
        let start = std::time::Instant::now();
        let image = decode(&rgb).unwrap();
        assert_eq!(image.data().len(), 288_000_000);
        println!("24 MP RGB FITS decode run {run}: {:?}; input {} bytes; output {} bytes", start.elapsed(), rgb.len(), image.data().len());
        std::hint::black_box(image);
    }
}

#[test]
fn malformed_interpretation_keywords_are_rejected() {
    for extra in [
        vec![card("BSCALE", "NaN")],
        vec![card("BZERO", "1E999")],
        vec![card("BSCALE", "1"), card("BSCALE", "2")],
        vec![card("BZERO", "0"), card("BZERO", "1")],
        vec![card("BITPIX", "8")],
        vec![card("NAXIS1", "2")],
        vec![card("NAXIS", "2")],
        vec![card("SIMPLE", "T")],
        vec![card("BLANK", "256")],
        vec![card("BLANK", "0"), card("BLANK", "1")],
        vec![card("GROUPS", "garbage")],
        vec![card("GROUPS", "T")],
        vec!["END     =                    T".into()],
    ] {
        assert!(decode(&fixture(8, 1, 1, &extra, &[1])).is_err(), "{extra:?}");
    }
    assert!(decode(&fixture(-32, 1, 1, &[card("BLANK", "0")], &0f32.to_be_bytes())).is_err());
    assert!(decode(&fixture(-32, 1, 1, &[card("BSCALE", "1E38")], &100f32.to_be_bytes())).is_err());
    for bitpix in [0, 64, -64, -16] {
        assert!(matches!(decode(&fixture(bitpix, 1, 1, &[], &[0; 8])), Err(CodecError::Unsupported { .. })));
    }
}

#[test]
fn missing_header_padding_order_and_unsupported_dimensions() {
    let original = fixture(8, 2, 2, &[], &[1, 2, 3, 4]);
    for i in [0, 1, 2, 3, 4] {
        let mut b = original.clone();
        b[i * 80] = b'X';
        assert!(decode_as(Format::Fits, &b).is_err());
    }
    for naxis in ["0", "1", "4"] {
        let mut b = original.clone();
        b[160..240].copy_from_slice(format!("{:80}", card("NAXIS", naxis)).as_bytes());
        assert!(matches!(decode(&b), Err(CodecError::Unsupported { .. })));
    }
    let mut b = original.clone();
    b[480] = b'X';
    assert!(decode(&b).is_err());
    b = original;
    b[400..480].fill(b' ');
    assert!(decode(&b).is_err());
}

#[test]
fn limits_apply_before_pixels_are_allocated() {
    let b = fixture(8, 2, 2, &[], &[1, 2, 3, 4]);
    for limits in [
        Limits { max_width: 1, ..Limits::default() },
        Limits { max_height: 1, ..Limits::default() },
        Limits { max_pixels: 3, ..Limits::default() },
        Limits { max_alloc: 15, ..Limits::default() },
    ] {
        assert!(matches!(decode_with(&b, &DecodeOptions { limits, ..Default::default() }), Err(CodecError::LimitExceeded(_))));
    }
    assert!(decode(&fixture(8, 0, 1, &[], &[])).is_err());
    assert!(decode(&fixture(32, u32::MAX, u32::MAX, &[], &[])).is_err());
}

#[test]
fn every_truncation_fails_and_header_mutations_do_not_panic() {
    let b = fixture(16, 2, 2, &[], &[0; 8]);
    for n in 0..b.len() {
        assert!(decode_as(Format::Fits, &b[..n]).is_err(), "truncation {n}");
    }
    let opts = DecodeOptions { limits: Limits { max_width: 8, max_height: 8, max_pixels: 64, max_alloc: 256 }, ..Default::default() };
    for i in 0..480 {
        let mut mutated = b.clone();
        mutated[i] ^= 0xff;
        let _ = decode_as_with(Format::Fits, &mutated, &opts);
    }
}

proptest! {
    #[test]
    fn arbitrary_input_never_panics(bytes in prop::collection::vec(any::<u8>(), 0..8000)) {
        let _ = decode_as(Format::Fits, &bytes);
    }
    #[test]
    fn scaled_signed16_matches_independent_oracle(values in prop::collection::vec(any::<i16>(), 1..100), scale in -16i32..16, zero in -100i32..100) {
        let raw: Vec<_> = values.iter().flat_map(|v| v.to_be_bytes()).collect();
        let bytes = fixture(16, values.len() as u32, 1, &[card("BSCALE", &scale.to_string()), card("BZERO", &zero.to_string())], &raw);
        let img = decode(&bytes).unwrap();
        let expected: Vec<_> = values.iter().map(|v| (i32::from(*v) * scale + zero) as f32).collect();
        prop_assert_eq!(floats(&img), expected);
    }
}
