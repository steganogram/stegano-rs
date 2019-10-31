use speculate::speculate;
use std::fs;
use stegano::{Decoder, Encoder, SteganoEncoder, SteganoDecoderV2, SteganoRawDecoder};

speculate! {
    describe "SteganoEncoder::new()" {
        it "should write to a png file" {
            SteganoEncoder::new().write_to("/tmp/out-test-image.png");
        }
        #[should_panic(expected = "Source file was not readable.")]
        it "should panic on a invalid source file" {
            SteganoEncoder::new().take_data_to_hide_from("foofile");
        }
        #[should_panic(expected = "Carrier image was not readable.")]
        it "should panic for invalid carrier image file" {
            SteganoEncoder::new().use_carrier_image("HelloWorld_no_passwd_v2.x.png");
        }
    }
    describe "Hide feature" {
        it "should hide the Cargo.toml on a png carrier to a new png file" {
            SteganoEncoder::new()
                .take_data_to_hide_from("Cargo.toml")
                .use_carrier_image("resources/HelloWorld_no_passwd_v2.x.png")
                .write_to("/tmp/out-test-image.png")
                .hide();

            let l = fs::metadata("/tmp/out-test-image.png")
                .expect("Output image was not written.")
                .len();

            assert_ne!(l, 0, "Output image was empty.");
        }
    }

    describe "Hide and Unveil e2e feature" {
        it "should unveil the Cargo.toml of a png" {
            SteganoEncoder::new()
                .take_data_to_hide_from("Cargo.toml")
                .use_carrier_image("resources/HelloWorld_no_passwd_v2.x.png")
                .write_to("/tmp/out-test-image.png")
                .hide();

            let l = fs::metadata("/tmp/out-test-image.png")
                .expect("Output image was not written.")
                .len();
            assert!(l > 0, "File is not supposed to be empty");

            SteganoDecoderV2::new()
                .use_source_image("/tmp/out-test-image.png")
                .write_to_file("/tmp/Cargo.toml")
                .unveil();

            let expected = fs::metadata("Cargo.toml")
                .expect("Source file is not available.")
                .len();
            let given = fs::metadata("/tmp/Cargo.toml")
                .expect("Output image was not written.")
                .len();

            assert_eq!(given, expected, "Unveiled file size differs to the original");
        }

        it "should unveil 'Hello World!' to stdout" {
            SteganoDecoderV2::new()
               .use_source_image("resources/HelloWorld_no_passwd_v2.x.png")
               .write_to_file("/tmp/HelloWorld.txt")
//               .write_to_stdout(io::stdout())
//               .write_to_vec(&b)
               .unveil();

//            let decipher = str::from_utf8(&*b).unwrap();
//            assert_eq!(decipher, "Hello World!", "unveiled text is not hello world");
        }

// TODO finish that test
//        it "should test the Read impl" {
//            let dec = SteganoDecoder::new()
//               .use_source_image("resources/HelloWorld_no_passwd_v2.x.png")
//               .write_to_file("/tmp/HelloWorld.txt");
//
//            let mut buf = Vec::new();
//            dec.read(buf);
//
//            print!("{:?}", buf);
//
//            let decipher = str::from_utf8(&*b).unwrap();
//            assert_eq!(decipher, "Hello World!", "unveiled text is not hello world");
//        }

        it "should raw unveil data contained in the image" {
            let dec = SteganoRawDecoder::new()
               .use_source_image("resources/HelloWorld_no_passwd_v2.x.png")
               .write_to_file("/tmp/HelloWorld.bin")
               .unveil();

            let l = fs::metadata("/tmp/HelloWorld.bin")
                .expect("Output file was not written.")
                .len();

            assert_ne!(l, 0, "Output raw data file was empty.");
        }

    }
}
