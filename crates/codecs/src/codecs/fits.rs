//! Primary grayscale and three-plane RGB FITS images, from the public IAU FITS 4.0 standard.
//! Physical samples are preserved, without an implicit display stretch.

use crate::{ChannelLayout, CodecError, DecodeWarning, Format, Image, Limits, SampleType};

const F: Format = Format::Fits;
const BLOCK: usize = 2880;
const MAX_HEADER: usize = 1 << 20;

fn err(message: impl std::fmt::Display) -> CodecError {
    CodecError::malformed(F, message)
}

fn value(card: &[u8]) -> Result<&str, CodecError> {
    if card.get(8..10) != Some(b"= ") {
        return Err(err("missing FITS value indicator"));
    }
    let text = std::str::from_utf8(card.get(10..).ok_or_else(|| err("short card"))?).map_err(err)?;
    Ok(text.split('/').next().unwrap_or("").trim())
}

fn integer(card: &[u8]) -> Result<i64, CodecError> {
    value(card)?.parse().map_err(err)
}

fn real(card: &[u8]) -> Result<f64, CodecError> {
    let number: f64 = value(card)?.replace('D', "E").parse().map_err(err)?;
    if !number.is_finite() {
        return Err(err("non-finite FITS scaling value"));
    }
    Ok(number)
}

fn set_once<T>(slot: &mut Option<T>, value: T, key: &str) -> Result<(), CodecError> {
    if slot.replace(value).is_some() {
        return Err(err(format!("duplicate {key}")));
    }
    Ok(())
}

pub(crate) fn decode(bytes: &[u8], limits: &Limits) -> Result<Image, CodecError> {
    let mut bitpix = None;
    let mut width = None;
    let mut height = None;
    let mut axes = None;
    let mut planes = None;
    let mut top_down = None;
    let mut scale = None;
    let mut zero = None;
    let mut blank = None;
    let mut groups = None;
    let mut offset = None;
    // Only interpretation-affecting keys are retained, so a hostile header cannot
    // allocate an unbounded map. Duplicates are ambiguous and are rejected.
    for (i, card) in bytes.get(..bytes.len().min(MAX_HEADER)).unwrap_or(&[]).chunks_exact(80).enumerate() {
        if !card.iter().all(|c| (b' '..=b'~').contains(c)) {
            return Err(err("FITS header must contain printable ASCII"));
        }
        let key = card.get(..8).ok_or_else(|| err("short card"))?;
        let expected = [b"SIMPLE  ", b"BITPIX  ", b"NAXIS   ", b"NAXIS1  ", b"NAXIS2  ", b"NAXIS3  "];
        if let Some(required) = expected.get(i).filter(|_| i < 5 || axes == Some(3))
            && key != *required
        {
            return Err(err("missing or out-of-order mandatory FITS card"));
        }
        match key {
            b"SIMPLE  " if i != 0 || value(card)? != "T" => {
                return Err(err("only SIMPLE=T primary HDUs are supported"));
            }
            b"BITPIX  " => {
                set_once(&mut bitpix, integer(card)?, "BITPIX")?;
            }
            b"NAXIS   " => {
                if i != 2 {
                    return Err(err("duplicate NAXIS"));
                }
                let n = integer(card)?;
                if !matches!(n, 2 | 3) {
                    return Err(CodecError::unsupported(F, "only 2D grayscale or three-plane RGB primary images are supported"));
                }
                axes = Some(n);
            }
            b"NAXIS3  " => {
                if axes != Some(3) {
                    return Err(err("NAXIS3 requires NAXIS=3"));
                }
                set_once(&mut planes, integer(card)?, "NAXIS3")?;
                if planes != Some(3) {
                    return Err(CodecError::unsupported(F, "only three-plane RGB images are supported for NAXIS=3"));
                }
            }
            b"NAXIS1  " | b"NAXIS2  " => {
                let n = u32::try_from(integer(card)?).map_err(|_| err("invalid FITS axis length"))?;
                let axis = if key == b"NAXIS1  " { &mut width } else { &mut height };
                if axis.replace(n).is_some() {
                    return Err(err("duplicate axis length"));
                }
            }
            b"BSCALE  " | b"BZERO   " => {
                let target = if key == b"BSCALE  " { &mut scale } else { &mut zero };
                if target.replace(real(card)?).is_some() {
                    return Err(err("duplicate FITS scaling keyword"));
                }
            }
            b"BLANK   " => {
                set_once(&mut blank, integer(card)?, "BLANK")?;
            }
            b"GROUPS  " => {
                let v = value(card)?;
                if !matches!(v, "T" | "F") || groups.replace(v == "T").is_some() {
                    return Err(err("invalid or duplicate GROUPS"));
                }
                if v == "T" {
                    return Err(CodecError::unsupported(F, "random groups are not supported"));
                }
            }
            b"ROWORDER" => {
                let text = value(card)?;
                let order = text.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')).ok_or_else(|| err("invalid ROWORDER string"))?.trim();
                let top = match order {
                    "TOP-DOWN" => true,
                    "BOTTOM-UP" => false,
                    _ => return Err(err("unsupported ROWORDER value")),
                };
                set_once(&mut top_down, top, "ROWORDER")?;
            }
            b"END     " => {
                if card.get(8..).is_none_or(|rest| rest.iter().any(|b| *b != b' ')) {
                    return Err(err("invalid END card"));
                }
                let end = (i + 1).checked_mul(80).ok_or_else(|| err("header overflow"))?;
                let padded = end.div_ceil(BLOCK).checked_mul(BLOCK).ok_or_else(|| err("header overflow"))?;
                let padding = bytes.get(end..padded).ok_or_else(|| err("truncated header block"))?;
                if padding.iter().any(|b| *b != b' ') {
                    return Err(err("non-space FITS header padding"));
                }
                offset = Some(padded);
                break;
            }
            _ => {}
        }
    }
    let offset = offset.ok_or_else(|| err("missing END or FITS header exceeds 1 MiB"))?;
    let bitpix = bitpix.ok_or_else(|| err("missing BITPIX"))?;
    let step: usize = match bitpix {
        8 => 1,
        16 => 2,
        32 | -32 => 4,
        _ => return Err(CodecError::unsupported(F, "supported BITPIX values are 8, 16, 32 and -32")),
    };
    let width = width.ok_or_else(|| err("missing NAXIS1"))?;
    let height = height.ok_or_else(|| err("missing NAXIS2"))?;
    let (layout, channels) = match (axes, planes) {
        (Some(2), None) => (ChannelLayout::Gray, 1usize),
        (Some(3), Some(3)) => (ChannelLayout::Rgb, 3usize),
        _ => return Err(err("missing or invalid FITS axes")),
    };
    limits.check(width, height, layout, SampleType::F32)?;
    if let Some(v) = blank {
        let valid = match bitpix {
            8 => u8::try_from(v).is_ok(),
            16 => i16::try_from(v).is_ok(),
            32 => i32::try_from(v).is_ok(),
            _ => false,
        };
        if !valid {
            return Err(err("BLANK is invalid for this BITPIX"));
        }
    }
    let pixels = (width as usize).checked_mul(height as usize).ok_or_else(|| err("pixel count overflow"))?;
    let count = pixels.checked_mul(channels).ok_or_else(|| err("sample count overflow"))?;
    let plane_len = pixels.checked_mul(step).ok_or_else(|| err("plane size overflow"))?;
    let length = count.checked_mul(step).ok_or_else(|| err("data length overflow"))?;
    let end = offset.checked_add(length).ok_or_else(|| err("data offset overflow"))?;
    let padded_end = end.div_ceil(BLOCK).checked_mul(BLOCK).ok_or_else(|| err("data padding overflow"))?;
    if bytes.get(end..padded_end).is_none() {
        return Err(err("truncated FITS data block"));
    }
    let data = bytes.get(offset..end).ok_or_else(|| err("truncated FITS image"))?;
    let output_len = count.checked_mul(4).ok_or_else(|| err("output size overflow"))?;
    let mut output = Vec::new();
    output.try_reserve_exact(output_len).map_err(|_| CodecError::LimitExceeded("cannot allocate FITS image".into()))?;
    if channels == 3 {
        output.resize(output_len, 0);
    }
    let scale = scale.unwrap_or(1.0);
    let zero = zero.unwrap_or(0.0);
    let row_len = (width as usize).checked_mul(step).ok_or_else(|| err("row size overflow"))?;
    let mut undefined = 0u64;
    let mut rounded = false;
    // FITS axis 1 varies fastest; axis 2 starts at the bottom of the image.
    // Three-plane images use consecutive R, G and B planes, not interleaved samples.
    for (plane, plane_data) in data.chunks_exact(plane_len).enumerate() {
        let rows = plane_data.chunks_exact(row_len);
        let mut rows: Box<dyn Iterator<Item = &[u8]> + '_> = if top_down.unwrap_or(false) { Box::new(rows) } else { Box::new(rows.rev()) };
        for (y, row) in rows.by_ref().enumerate() {
            for (x, sample) in row.chunks_exact(step).enumerate() {
                let raw = match bitpix {
                    8 => f64::from(*sample.first().ok_or_else(|| err("short sample"))?),
                    16 => f64::from(i16::from_be_bytes(sample.try_into().map_err(err)?)),
                    32 => f64::from(i32::from_be_bytes(sample.try_into().map_err(err)?)),
                    -32 => f64::from(f32::from_be_bytes(sample.try_into().map_err(err)?)),
                    _ => return Err(err("invalid sample type")),
                };
                let invalid = !raw.is_finite() || (bitpix > 0 && blank.is_some_and(|v| raw == v as f64));
                let physical = raw * scale + zero;
                let converted = if invalid {
                    undefined += 1;
                    0.0
                } else {
                    let v = physical as f32;
                    if !physical.is_finite() || !v.is_finite() {
                        return Err(err("scaled FITS sample exceeds finite 32-bit float range"));
                    }
                    rounded |= f64::from(v) != physical;
                    v
                };
                if channels == 1 {
                    output.extend_from_slice(&converted.to_ne_bytes());
                } else {
                    let index = y
                        .checked_mul(width as usize)
                        .and_then(|v| v.checked_add(x))
                        .and_then(|v| v.checked_mul(channels))
                        .and_then(|v| v.checked_add(plane))
                        .and_then(|v| v.checked_mul(4))
                        .ok_or_else(|| err("output offset overflow"))?;
                    let end = index.checked_add(4).ok_or_else(|| err("output offset overflow"))?;
                    output.get_mut(index..end).ok_or_else(|| err("output offset out of bounds"))?.copy_from_slice(&converted.to_ne_bytes());
                }
            }
        }
    }
    let mut image = Image::from_raw(width, height, layout, SampleType::F32, output)?;
    if undefined > 0 {
        image.warnings.push(DecodeWarning::FitsUndefined { pixels: undefined });
    }
    if rounded {
        image.warnings.push(DecodeWarning::FitsPrecisionReduced);
    }
    if bytes.len() > padded_end {
        image.warnings.push(DecodeWarning::FitsExtensionsIgnored);
    }
    Ok(image)
}
