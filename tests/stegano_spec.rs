use speculate::speculate;
use std::fs;
use stegano::{BitIterator, Encoder, Steganogramm};

speculate! {
    describe "Steganogramm::new()" {
        it "should write to a png file" {
            Steganogramm::new().write_to("/tmp/out-test-image.png");
        }
        #[should_panic(expected = "Source file was not readable.")]
        it "should panic on a invalid source file" {
            Steganogramm::new().take_data_to_hide_from("foofile");
        }
        #[should_panic(expected = "Carrier image was not readable.")]
        it "should panic for invalid carrier image file" {
            Steganogramm::new().use_carrier_image("HelloWorld_no_passwd_v2.x.png");
        }
    }
    describe "Hide feature" {
        it "should hide the Cargo.toml on a png carrier to a new png file" {
            Steganogramm::new()
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
    describe "BitIterator::next()" {
        // String: H           e           l
        // Hex   : 0x48        0x61        0x6C
        // Binary: 0b01001000  0b01100001  0b01101100
        it "should return the 8 bits of 'H' in LittleEndian byte order" {
            let b = vec![0b01001000, 0b01100001];
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
            let b = vec![0b01001000, 0b01100001];
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
    }
}
