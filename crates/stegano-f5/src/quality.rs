//! JPEG quality estimation from quantization tables.
//!
//! Implements the ImageMagick heuristic algorithm that compares quantization
//! table hash sums and total sums against reference values derived from the
//! standard IJG (Independent JPEG Group) tables.

/// Deduplicate per-component quantization tables into distinct tables.
/// In a YCbCr JPEG, Cb and Cr typically share one table, so 3 component
/// tables become 2 distinct ones — matching PIL/ImageMagick behavior.
fn deduplicate_tables(tables: &[[u16; 64]]) -> Vec<&[u16; 64]> {
    let mut unique: Vec<&[u16; 64]> = Vec::with_capacity(tables.len());
    for table in tables {
        if !unique.iter().any(|u| u.as_slice() == table.as_slice()) {
            unique.push(table);
        }
    }
    unique
}

/// Estimate JPEG quality (1–100) from quantization tables.
///
/// Expects the quantization tables as extracted by the decoder's
/// `RawCoefficients::quantization_tables` (one per component, 64 entries each).
///
/// Returns the estimated quality, or 90 as a fallback if tables are empty.
pub fn estimate_quality(quantization_tables: &[[u16; 64]]) -> u8 {
    if quantization_tables.is_empty() {
        return 90; // fallback
    }

    let is_color = quantization_tables.len() >= 2;

    // Compute qvalue: hash of specific quantization table positions
    let qvalue: i32 = if is_color {
        let luma = &quantization_tables[0];
        let chroma = &quantization_tables[1];
        luma[2] as i32 + luma[53] as i32 + chroma[0] as i32 + chroma[63] as i32
    } else {
        let luma = &quantization_tables[0];
        luma[2] as i32 + luma[53] as i32
    };

    // Compute qsum: total of distinct quantization table values.
    // The decoder returns per-component tables (e.g. 3 for YCbCr where Cb and Cr
    // share one table), but the ImageMagick algorithm operates on distinct table
    // indices (typically 1 for grayscale, 2 for color). Deduplicate before summing.
    let unique_tables = deduplicate_tables(quantization_tables);
    let qsum: i32 = unique_tables
        .iter()
        .flat_map(|t| t.iter())
        .map(|&v| v as i32)
        .sum();

    let (hashes, sums) = if is_color {
        (&HASH_COLOR[..], &SUMS_COLOR[..])
    } else {
        (&HASH_GRAY[..], &SUMS_GRAY[..])
    };

    // ImageMagick threshold scan: iterate from quality 0 and return
    // the first quality where the measured values meet the reference.
    for i in 0..100 {
        if qvalue < hashes[i] && qsum < sums[i] {
            continue;
        }
        if (qvalue <= hashes[i] && qsum <= sums[i]) || i >= 50 {
            return (i + 1) as u8;
        }
        break;
    }

    // Fallback: no match found (non-standard tables)
    90
}

// Reference arrays from ImageMagick (coders/jpeg.c), derived from the
// standard IJG quantization tables at each quality level 0–100.

/// Hash values for color JPEGs (2+ quantization tables).
/// qvalue = luma[2] + luma[53] + chroma[0] + chroma[63]
#[rustfmt::skip]
const HASH_COLOR: [i32; 101] = [
    1020, 1015,  932,  848,  780,  735,  702,  679,  660,  645,
     632,  623,  613,  607,  600,  594,  589,  585,  581,  571,
     555,  542,  529,  514,  494,  474,  457,  439,  424,  410,
     397,  386,  373,  364,  351,  341,  334,  324,  317,  309,
     299,  294,  287,  279,  274,  267,  262,  257,  251,  247,
     243,  237,  232,  227,  222,  217,  213,  207,  202,  198,
     192,  188,  183,  177,  173,  168,  163,  157,  153,  148,
     143,  139,  132,  128,  125,  119,  115,  108,  104,   99,
      94,   90,   84,   79,   74,   70,   64,   59,   55,   49,
      45,   40,   34,   30,   25,   20,   15,   11,    6,    4,
       0,
];

/// Sum of all quantization values for color JPEGs at each quality level.
#[rustfmt::skip]
const SUMS_COLOR: [i32; 101] = [
    32640, 32635, 32266, 31495, 30665, 29804, 29146, 28599, 28104,
    27670, 27225, 26725, 26210, 25716, 25240, 24789, 24373, 23946,
    23572, 22846, 21801, 20842, 19949, 19121, 18386, 17651, 16998,
    16349, 15800, 15247, 14783, 14321, 13859, 13535, 13081, 12702,
    12423, 12056, 11779, 11513, 11135, 10955, 10676, 10392, 10208,
     9928,  9747,  9564,  9369,  9193,  9017,  8822,  8639,  8458,
     8270,  8084,  7896,  7710,  7527,  7347,  7156,  6977,  6788,
     6607,  6422,  6236,  6054,  5867,  5684,  5495,  5305,  5128,
     4945,  4751,  4638,  4442,  4248,  4065,  3888,  3698,  3509,
     3326,  3139,  2957,  2775,  2586,  2405,  2216,  2037,  1846,
     1666,  1483,  1297,  1109,   927,   735,   554,   375,   201,
      128,     0,
];

/// Hash values for grayscale JPEGs (1 quantization table).
/// qvalue = luma[2] + luma[53]
#[rustfmt::skip]
const HASH_GRAY: [i32; 101] = [
     510,  505,  422,  380,  355,  338,  326,  318,  311,  305,
     300,  297,  293,  291,  288,  286,  284,  283,  281,  280,
     279,  278,  277,  273,  262,  251,  243,  233,  225,  218,
     211,  205,  198,  193,  186,  181,  177,  172,  168,  164,
     158,  156,  152,  148,  145,  142,  139,  136,  133,  131,
     129,  126,  123,  120,  118,  115,  113,  110,  107,  105,
     102,  100,   97,   94,   92,   89,   87,   83,   81,   79,
      76,   74,   70,   68,   66,   63,   61,   57,   55,   52,
      50,   48,   44,   42,   39,   37,   34,   31,   29,   26,
      24,   21,   18,   16,   13,   11,    8,    6,    3,    2,
       0,
];

/// Sum of all quantization values for grayscale JPEGs at each quality level.
#[rustfmt::skip]
const SUMS_GRAY: [i32; 101] = [
    16320, 16315, 15946, 15277, 14655, 14073, 13623, 13230, 12859,
    12560, 12240, 11861, 11456, 11081, 10714, 10360, 10027,  9679,
     9368,  9056,  8680,  8331,  7995,  7668,  7376,  7084,  6823,
     6562,  6345,  6125,  5939,  5756,  5571,  5421,  5240,  5086,
     4976,  4829,  4719,  4616,  4463,  4393,  4280,  4166,  4092,
     3980,  3909,  3835,  3755,  3688,  3621,  3541,  3467,  3396,
     3323,  3247,  3170,  3096,  3021,  2952,  2874,  2804,  2727,
     2657,  2583,  2509,  2437,  2362,  2290,  2211,  2136,  2068,
     1996,  1915,  1858,  1773,  1692,  1620,  1552,  1477,  1398,
     1326,  1251,  1179,  1109,  1031,   961,   884,   814,   736,
      667,   592,   518,   441,   369,   292,   221,   151,    86,
       64,     0,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_tables_returns_fallback() {
        assert_eq!(estimate_quality(&[]), 90);
    }

    #[test]
    fn test_quality_100_color() {
        // At quality 100, all quantization values are 1
        let luma = [1u16; 64];
        let chroma = [1u16; 64];
        let q = estimate_quality(&[luma, chroma]);
        assert!(q >= 98, "Expected ~100, got {q}");
    }
}
