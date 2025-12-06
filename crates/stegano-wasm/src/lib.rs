use wasm_bindgen::prelude::*;
use stegano_core::media::Media;
use stegano_core::SteganoEncoder;
use stegano_core::api::unveil;
use std::io::Cursor;
use image::ImageFormat;

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn hide_data(carrier_data: &[u8], secret_name: &str, secret_data: &[u8], password: Option<String>) -> Result<Vec<u8>, JsValue> {
    let img = image::load_from_memory(carrier_data)
        .map_err(|e| JsValue::from_str(&format!("Failed to load image: {}", e)))?
        .to_rgba8();

    let media = Media::from_image(img);
    
    let mut encoder = SteganoEncoder::default();
    if let Some(pwd) = password {
        encoder.with_encryption(pwd);
    }
    encoder.use_media_from_media(media);
    encoder.add_file_from_memory(secret_name, secret_data).map_err(|e| JsValue::from_str(&format!("Failed to add memory file: {}", e)))?;
    
    let result = encoder.hide_to_vec().map_err(|e| JsValue::from_str(&format!("Failed to hide data: {}", e)))?;
    
    Ok(result)
}

#[wasm_bindgen]
pub struct UnveiledFile {
    name: String,
    data: Vec<u8>,
}

#[wasm_bindgen]
impl UnveiledFile {
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn data(&self) -> Vec<u8> {
        self.data.clone()
    }
}

#[wasm_bindgen]
pub fn unveil_data(carrier_data: &[u8], password: Option<String>) -> Result<Vec<UnveiledFile>, JsValue> {
    let img = image::load_from_memory(carrier_data)
        .map_err(|e| JsValue::from_str(&format!("Failed to load image: {}", e)))?
        .to_rgba8();
        
    let media = Media::from_image(img);
    
    let mut unveil = unveil::prepare();
    if let Some(pwd) = password {
        unveil = unveil.using_password(Some(pwd));
    }

    let results = unveil
        .from_media(media)
        .execute_to_memory()
        .map_err(|e| JsValue::from_str(&format!("Failed to unveil: {}", e)))?;
        
    Ok(results.into_iter().map(|(name, data)| UnveiledFile { name, data }).collect())
}
