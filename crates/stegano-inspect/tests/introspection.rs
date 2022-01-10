use itertools::Itertools;
use jpeg_decoder::PixelFormat;
use jpeg_encoder::ColorType;
use std::fs::File;
use std::io::BufReader;

#[test]
fn jpeg_introspection_foreign_format() {
    let f = File::open("resources/twitter/test_8x8_255_100.jpeg").unwrap();
    let mut decoder = jpeg_decoder::Decoder::new(BufReader::new(f));
    let mut _data = decoder.decode().expect("Decoding failed. If other software can successfully decode the specified JPEG image, then it's likely that there is a bug in jpeg-decoder");
    let info = decoder.info().unwrap();

    assert_eq!(info.width, 8);
    assert_eq!(info.height, 8);
    assert_eq!(info.pixel_format, PixelFormat::RGB24);
}

#[test]
fn jpeg_introspection_origin_format() {
    let f = File::open("resources/samples/test_8x8_255_80.jpg").unwrap();
    let mut decoder = jpeg_decoder::Decoder::new(BufReader::new(f));
    let mut _data = decoder.decode().expect("Decoding failed. If other software can successfully decode the specified JPEG image, then it's likely that there is a bug in jpeg-decoder");
    let info = decoder.info().unwrap();

    assert_eq!(info.width, 8);
    assert_eq!(info.height, 8);
    assert_eq!(info.pixel_format, PixelFormat::RGB24);
}

fn prepare_chess_image<const H: usize, const W: usize, const C: usize, const S: usize>() -> [u8; S]
{
    let stride: usize = W * C;
    let color = [255_u8; C];
    let mut img = [0_u8; S];
    for (x, y) in (0..W).into_iter().cartesian_product((0..H).into_iter()) {
        if (x + y) % 2 == 0 {
            // let color = rgb_to_ycbcr(200_u8, 200, 200);
            let px = x * C;
            let py = y * stride;
            for c in 0..C {
                img[px + py + c] = color[c];
            }
        }
    }

    img
}

#[test]
fn should_prepare_some_images() {
    let img = prepare_chess_image::<8, 8, 4, { 8 * 8 * 4 }>();

    for q in (0..101).step_by(10) {
        let encoder = jpeg_encoder::Encoder::new_file(
            format!("resources/samples/test_8x8_255_{q}.jpg", q = q),
            q,
        )
        .unwrap();
        encoder.encode(&img, 8, 8, ColorType::Rgba).unwrap();
    }
}

#[test]
fn should_prepare_some_bigger_images() {
    let img = prepare_chess_image::<512, 512, 4, { 512 * 512 * 4 }>();

    for q in (0..101).step_by(10) {
        let encoder = jpeg_encoder::Encoder::new_file(
            format!("resources/samples/test_512x512_255_{q}.jpg", q = q),
            q,
        )
        .unwrap();
        encoder.encode(&img, 512, 512, ColorType::Rgba).unwrap();
    }
}
