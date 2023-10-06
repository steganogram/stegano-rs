use std::fs;
use tempfile::TempDir;

use stegano_core::commands::unveil;
use stegano_core::*;

pub const DEMO_IMG_V1_TEXT_ONLY_WITHOUT_PASSWD_V2_1_1_9: &str =
    "../resources/demo-secrets/message-version-1/text-only-without-passwd-v2.1.1.9.PNG";

#[test]
fn ensure_text_only_without_password_from_v2_1_1_9_unveils() {
    let out_dir = TempDir::new().unwrap();

    unveil(
        DEMO_IMG_V1_TEXT_ONLY_WITHOUT_PASSWD_V2_1_1_9.as_ref(),
        out_dir.as_ref(),
        &CodecOptions::default(),
    )
    .unwrap();

    let contents =
        String::from_utf8(fs::read(out_dir.as_ref().join("secret-message.txt")).unwrap()).unwrap();

    assert_eq!(
        contents,
        "Welcome to a Text Only Secret Message".to_string()
    );
}
