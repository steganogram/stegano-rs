use itertools::Itertools;
use std::fs::File;
use std::io::BufReader;
use stegano_f5_jpeg_decoder::{Decoder, PixelFormat};
use stegano_f5_jpeg_encoder::{ColorType, Encoder};

#[test]
fn jpeg_introspection_foreign_format() {
    let f = File::open("resources/twitter/test_8x8_255_100.jpeg").unwrap();
    let mut decoder = Decoder::new(BufReader::new(f));
    let mut _data = decoder.decode().expect("Decoding failed. If other software can successfully decode the specified JPEG image, then it's likely that there is a bug in jpeg-decoder");
    let info = decoder.info().unwrap();

    assert_eq!(info.width, 8);
    assert_eq!(info.height, 8);
    assert_eq!(info.pixel_format, PixelFormat::RGB24);
}

#[test]
fn jpeg_introspection_origin_format() {
    let f = File::open("resources/samples/test_8x8_255_80.jpg").unwrap();
    let mut decoder = Decoder::new(BufReader::new(f));
    let mut _data = decoder.decode().expect("Decoding failed. If other software can successfully decode the specified JPEG image, then it's likely that there is a bug in jpeg-decoder");
    let info = decoder.info().unwrap();

    assert_eq!(info.width, 8);
    assert_eq!(info.height, 8);
    assert_eq!(info.pixel_format, PixelFormat::RGB24);
}

fn prepare_chess_image(width: usize, height: usize, channels: usize) -> Vec<u8> {
    let stride = width * channels;
    let color = 255_u8;
    let mut img = vec![0_u8; width * height * channels];
    for (x, y) in (0..width).cartesian_product(0..height) {
        if (x + y) % 2 == 0 {
            let px = x * channels;
            let py = y * stride;
            for c in 0..channels {
                img[px + py + c] = color;
            }
        }
    }
    img
}

fn prepare_fixture_image(w: usize, h: usize) {
    let img = prepare_chess_image(w, h, 4);

    for q in (0..101).step_by(10) {
        let encoder =
            Encoder::new_file(format!("resources/samples/test_{w}x{h}_255_{q}.jpg"), q).unwrap();
        encoder
            .encode(&img, w as _, h as _, ColorType::Rgba)
            .unwrap();
    }
}

#[test]
#[ignore] // Fixture generator - run manually when needed
fn generate_8x8_fixtures() {
    prepare_fixture_image(8, 8);
}

#[test]
#[ignore] // Fixture generator - run manually when needed
fn generate_512x512_fixtures() {
    prepare_fixture_image(512, 512);
}
