use argh::FromArgs;
use stegano_f5_jpeg_decoder::{CodingProcess, Decoder, PixelFormat};
use tabled::{
    settings::{Alignment, Style},
    builder::Builder,
};

#[derive(FromArgs)]
/// Inspecting jpeg image files
struct SteganoInspectArgs {
    #[argh(subcommand)]
    command: Command,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum Command {
    Quantization(QuantizationArgs),
    Summary(SummaryArgs),
}

#[derive(FromArgs, PartialEq, Debug)]
/// Shows the quantization tables of jpeg files
#[argh(subcommand, name = "quantization")]
struct QuantizationArgs {
    /// the jpeg image file to inspect
    #[argh(positional)]
    jpeg_files: Vec<String>,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Shows a JPEG internals summary
#[argh(subcommand, name = "summary")]
struct SummaryArgs {
    /// the jpeg image file to inspect
    #[argh(positional)]
    jpeg_file: String,
}

fn main() {
    let args: SteganoInspectArgs = argh::from_env();

    match &args.command {
        Command::Quantization(args) => cmd_quantization(args),
        Command::Summary(args) => cmd_summary(args),
    }
}

fn cmd_quantization(args: &QuantizationArgs) {
    for file_name in &args.jpeg_files {
        let data = std::fs::read(file_name).expect("cannot open file");
        let mut decoder = Decoder::new(&data[..]);
        let raw = decoder
            .decode_raw_coefficients()
            .expect("Failed to decode JPEG coefficients");

        let display_name = std::fs::canonicalize(file_name)
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .unwrap_or_else(|| file_name.clone());

        println!("# Quantization Tables of `{}`\n", display_name);

        for (i, table) in raw.quantization_tables.iter().enumerate() {
            println!("## Table {}\n", i);
            println!("{}\n", format_quant_table(table));
        }
    }
}

fn format_quant_table(table: &[u16; 64]) -> String {
    let mut builder = Builder::default();

    for row in 0..8 {
        let row_data: Vec<String> = (0..8)
            .map(|col| format!("{}", table[row * 8 + col]))
            .collect();
        builder.push_record(row_data);
    }

    let mut tbl = builder.build();
    tbl.with(Style::markdown());
    tbl.with(Alignment::right());

    tbl.to_string()
}

fn cmd_summary(args: &SummaryArgs) {
    let file_name = &args.jpeg_file;
    let data = std::fs::read(file_name).expect("cannot open file");
    let file_size = data.len();

    let mut decoder = Decoder::new(&data[..]);
    let raw = decoder
        .decode_raw_coefficients()
        .expect("Failed to decode JPEG");

    let info = decoder.info().expect("no image info available");
    let frame = decoder.frame_info().expect("no frame info available");

    let display_name = std::fs::canonicalize(file_name)
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| file_name.clone());

    println!("# JPEG Summary: `{}`\n", display_name);

    let mut rows: Vec<[String; 2]> = Vec::new();

    // File size
    rows.push(["File size".into(), format_file_size(file_size)]);

    // Dimensions
    rows.push([
        "Dimensions".into(),
        format!("{} x {}", info.width, info.height),
    ]);

    // Coding process
    let coding = match info.coding_process {
        CodingProcess::DctSequential => "Sequential DCT",
        CodingProcess::DctProgressive => "Progressive DCT",
        CodingProcess::Lossless => "Lossless",
    };
    rows.push(["Coding process".into(), coding.into()]);

    // Baseline
    rows.push([
        "Baseline".into(),
        if frame.is_baseline { "Yes" } else { "No" }.into(),
    ]);

    // Color model
    let color_model = match info.pixel_format {
        PixelFormat::L8 | PixelFormat::L16 => "Grayscale".into(),
        PixelFormat::RGB24 => "YCbCr (RGB24)".into(),
        PixelFormat::CMYK32 => "CMYK".into(),
    };
    rows.push(["Color model".into(), color_model]);

    // Components with sampling factors
    let components_str = format_components(&frame.components);
    rows.push(["Components".into(), components_str]);

    // Quantization tables
    let quant_count = raw.quantization_tables.len();
    let precision = if raw.quantization_tables.iter().all(|t| t.iter().all(|&v| v <= 255)) {
        "8-bit"
    } else {
        "16-bit"
    };
    rows.push([
        "Quantization tables".into(),
        format!(
            "{} ({}) — `stegano-inspect quantization {}`",
            quant_count, precision, file_name
        ),
    ]);

    // Metadata presence
    let exif = if decoder.exif_data().is_some() {
        "Present"
    } else {
        "Not present"
    };
    rows.push(["EXIF".into(), exif.into()]);

    let xmp = if decoder.xmp_data().is_some() {
        "Present"
    } else {
        "Not present"
    };
    rows.push(["XMP".into(), xmp.into()]);

    let icc = if decoder.icc_profile().is_some() {
        "Present"
    } else {
        "Not present"
    };
    rows.push(["ICC Profile".into(), icc.into()]);

    // F5 capacity
    let capacity = stegano_f5::jpeg_capacity(&data)
        .map(|c| format!("~{} bytes", format_number(c)))
        .unwrap_or_else(|_| "N/A".into());
    rows.push(["F5 capacity".into(), capacity]);

    // Build table
    let mut builder = Builder::default();
    builder.push_record(["Property", "Value"]);
    for row in &rows {
        builder.push_record(row);
    }

    let mut table = builder.build();
    table.with(Style::markdown());

    println!("{}", table);
}

fn format_file_size(bytes: usize) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} bytes", bytes)
    }
}

fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

fn component_name(id: u8) -> &'static str {
    match id {
        1 => "Y",
        2 => "Cb",
        3 => "Cr",
        4 => "I",
        5 => "Q",
        _ => "?",
    }
}

fn format_components(components: &[stegano_f5_jpeg_decoder::Component]) -> String {
    let parts: Vec<String> = components
        .iter()
        .map(|c| {
            format!(
                "{}({}x{})",
                component_name(c.identifier),
                c.horizontal_sampling_factor,
                c.vertical_sampling_factor
            )
        })
        .collect();

    format!("{} — {}", components.len(), parts.join(" "))
}
