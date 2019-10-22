use speculate::speculate;
use std::fs;
use stegano::{BitIterator, Decoder, Encoder, SteganoDecoderV2, SteganoEncoder};
use bitstream_io::{BitReader, LittleEndian};

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
    }

    describe "BitIterator::next()" {
        // String: H           e           l
        // Hex   : 0x48        0x61        0x6C
        // Binary: 0b01001000  0b01100001  0b01101100
        it "should return the 8 bits of 'H' in LittleEndian byte order" {
            let b = vec![0b0100_1000, 0b0110_0001, 0b0110_1100];
            let mut it = BitIterator::new(&b[..]);

            assert_eq!(it.next().unwrap(), 0, "1st bit not correct");
            assert_eq!(it.next().unwrap(), 0, "2nd bit not correct");
            assert_eq!(it.next().unwrap(), 0, "3rd bit not correct");
            assert_eq!(it.next().unwrap(), 1, "4th bit not correct");
            assert_eq!(it.next().unwrap(), 0, "5th bit not correct");
            assert_eq!(it.next().unwrap(), 0, "6th bit not correct");
            assert_eq!(it.next().unwrap(), 1, "7th bit not correct");
            assert_eq!(it.next().unwrap(), 0, "8th bit not correct");
        }

        // String: H           e           l
        // Hex   : 0x48        0x61        0x6C
        // Binary: 0b01001000  0b01100001  0b01101100
        it "should return 8 bits of 'e' in LittleEndian byte order after skip(8)" {
            let b = vec![0b0100_1000, 0b0110_0001];
            let mut it = BitIterator::new(&b[..]).skip(8);

            assert_eq!(it.next().unwrap(), 1, "1st bit not correct");
            assert_eq!(it.next().unwrap(), 0, "2nd bit not correct");
            assert_eq!(it.next().unwrap(), 0, "3rd bit not correct");
            assert_eq!(it.next().unwrap(), 0, "4th bit not correct");
            assert_eq!(it.next().unwrap(), 0, "5th bit not correct");
            assert_eq!(it.next().unwrap(), 1, "6th bit not correct");
            assert_eq!(it.next().unwrap(), 1, "7th bit not correct");
            assert_eq!(it.next().unwrap(), 0, "8th bit not correct");
            assert_eq!(it.next(), None, "it should end after the last bit on the last byte");
        }

        it "should behave as the BitReader" {
            let b = vec![0b0100_1000, 0b0110_0001];
            let mut it = BitIterator::new(&b[..]);
            let mut reader = BitReader::endian(
                &b[..],
                LittleEndian
            );

            for i in 0..16 {
                assert_eq!(
                    it.next().unwrap(),
                    if reader.read_bit().unwrap() { 1 } else { 0 },
                    "{} bit not correct", i
                );
            }
        }
    }
}
