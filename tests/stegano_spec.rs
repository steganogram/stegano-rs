use speculate::speculate;
use std::fs;
use stegano::Steganogramm;

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
                .use_carrier_image("ressources/HelloWorld_no_passwd_v2.x.png")
                .write_to("/tmp/out-test-image.png")
                .hide();

            let l = fs::metadata("/tmp/out-test-image.png")
                .expect("Output image was not written.")
                .len();

            assert_ne!(l, 0, "Output image was empty.");
        }
    }
}
