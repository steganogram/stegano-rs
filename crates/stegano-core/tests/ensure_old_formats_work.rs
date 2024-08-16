use std::fs;
use std::fs::File;
use tempfile::TempDir;

use stegano_core::commands::unveil;
use stegano_core::*;

pub const DEMO_IMG_V1_TEXT_ONLY_WITHOUT_PASSWD_V2_1_1_9: &str =
    "tests/demo-secrets/message-version-1/text-only-without-passwd-v2.1.1.9.PNG";

pub const DEMO_IMG_V2_TEXT_AND_DOCUMENT_WITHOUT_PASSWD_V2_1_1_9: &str =
    "tests//demo-secrets/message-version-2/text-and-one-document-without-passwd-v2.1.1.9.PNG";

pub const DEMO_IMG_TEXT_AND_DOCUMENT_WITHOUT_PASSWD_V2_2_5: &str =
    "tests/demo-secrets/message-version-text-and-documents-with-length-header/text-and-one-document-without-passwd-v2.2.5.PNG";

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

#[test]
fn ensure_text_and_documents_without_password_from_v2_1_1_9_unveils() {
    let out_dir = TempDir::new().unwrap();

    unveil(
        DEMO_IMG_V2_TEXT_AND_DOCUMENT_WITHOUT_PASSWD_V2_1_1_9.as_ref(),
        out_dir.as_ref(),
        &CodecOptions::default(),
    )
    .unwrap();

    let contents =
        String::from_utf8(fs::read(out_dir.as_ref().join("secret-message.txt")).unwrap()).unwrap();

    assert_eq!(
        contents,
        "Welcome to a secret message with also one document, containing maybe another message."
            .to_string()
    );

    let contained_file = File::open(out_dir.as_ref().join("image-with-hello-world.png")).unwrap();
    assert_eq!(contained_file.metadata().unwrap().len(), 188292);
}
