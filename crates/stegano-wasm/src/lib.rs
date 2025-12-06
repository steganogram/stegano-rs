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
    let mut img = image::load_from_memory(carrier_data)
        .map_err(|e| JsValue::from_str(&format!("Failed to load image: {}", e)))?
        .to_rgba8();

    // Auto-Resize Logic
    // Capacity in bytes = (width * height * 3) / 8
    // We compare against secret_data.len() + estimated overhead (e.g. 1KB for header)
    let overhead = 1024; 
    let payload_size = secret_data.len() + overhead;
    let capacity = (img.width() as usize * img.height() as usize * 3) / 8;

    if payload_size > capacity {
        // Calculate new dimensions
        // required_pixels = (payload_size * 8) / 3
        let required_pixels = (payload_size as f64 * 8.0) / 3.0;
        let current_pixels = (img.width() * img.height()) as f64;
        let scale_factor = (required_pixels / current_pixels).sqrt() * 1.1; // 1.1 safety margin

        let new_width = (img.width() as f64 * scale_factor).ceil() as u32;
        let new_height = (img.height() as f64 * scale_factor).ceil() as u32;

        img = image::imageops::resize(&img, new_width, new_height, image::imageops::FilterType::Lanczos3);
    }

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
