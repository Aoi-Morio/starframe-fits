//! External, independently generated oracles stay outside the source tree.
//! Set FITS_ORACLE_DIR to a folder containing .fits files and matching .oracle
//! files: little-endian u32 width/height, then little-endian f32 samples in
//! top-to-bottom order (BSCALE/BZERO applied, undefined samples replaced by zero).
//! RGB oracles have three interleaved samples per pixel; grayscale has one.
use photocraft_codecs::{ChannelLayout, SampleType, decode};

#[test]
#[ignore = "requires independently generated real FITS oracles; set FITS_ORACLE_DIR"]
fn external_fits_samples_match_every_physical_pixel() {
    let root = std::env::var_os("FITS_ORACLE_DIR").expect("set FITS_ORACLE_DIR");
    let mut checked = 0;
    for entry in std::fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "fits") {
            continue;
        }
        let expected = std::fs::read(path.with_extension("oracle")).unwrap();
        let width = u32::from_le_bytes(expected[0..4].try_into().unwrap());
        let height = u32::from_le_bytes(expected[4..8].try_into().unwrap());
        let img = decode(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(img.dimensions(), (width, height), "{}", path.display());
        let channels = (expected.len() - 8) / (width as usize * height as usize * 4);
        assert_eq!(
            img.layout(),
            match channels {
                1 => ChannelLayout::Gray,
                3 => ChannelLayout::Rgb,
                _ => panic!("invalid oracle layout"),
            }
        );
        assert_eq!(img.sample_type(), SampleType::F32);
        assert_eq!(img.data().len(), expected.len() - 8);
        for (i, (actual, expected)) in img.data().chunks_exact(4).zip(expected[8..].chunks_exact(4)).enumerate() {
            assert_eq!(f32::from_ne_bytes(actual.try_into().unwrap()).to_le_bytes(), expected, "{}: sample {i}", path.display());
        }
        println!("{}: {width}x{height} physical samples match", path.display());
        checked += 1;
    }
    assert!(checked > 0, "no external FITS samples checked");
}
