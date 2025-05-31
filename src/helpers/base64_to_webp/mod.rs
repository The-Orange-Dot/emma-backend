use image::ImageOutputFormat;
use base64::{Engine as _, engine::general_purpose};

fn convert_base64_to_webp(base64_str: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Remove data URL prefix
    let base64_data = base64_str.split(',').last().unwrap_or(base64_str);
    let bytes = general_purpose::STANDARD.decode(base64_data)?;
    
    let img = image::load_from_memory(&bytes)?;
    let mut webp_bytes = Vec::new();
    img.write_to(&mut webp_bytes, ImageOutputFormat::WebP)?;
    
    Ok(webp_bytes)
}