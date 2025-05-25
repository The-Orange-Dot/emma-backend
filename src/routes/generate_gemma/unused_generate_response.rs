use crate::models::generation_models::{GemmaRequest, ImageData};
use actix_web::{Result, Error};

// DONT EVENT USE THIS ANYMORE

fn extract_base64(data_url: &str) -> Result<String, String> {
    data_url
        .split(',')
        .nth(1)
        .ok_or_else(|| "Invalid data URL format".to_string())
        .map(|s| s.to_string())
}

pub async fn _generate_response(images: Vec<ImageData>, 
  prompt: String) -> Result<String, Error> {
   // Process images uploaded by the user to extract just the base64 portion
  let images = images
      .into_iter()
      .map(|img| extract_base64(&img.data_url))
      .collect::<Result<Vec<String>, String>>()
      .map_err(|e| {
          eprintln!("Error processing images: {}", e);
          actix_web::error::ErrorBadRequest("Invalid image data")
      })?;
  dotenv::dotenv().ok();
  let model = std::env::var("LMM_MODEL").expect("No model for generating first response has been set.");

    // Create typed request body
  let body = GemmaRequest {
      model: model,
      prompt: prompt,
      stream: false,
      images: images,
  };

  let client = reqwest::Client::new();
  let url = std::env::var("SERVER_URL").expect("Could not find server url");

  // Sends a request with prompt to the AI server
  let response = client.post(url)
  .header("Content-Type", "application/json")
  .body(serde_json::to_string(&body).unwrap())
  .send()
  .await
  .map_err(|err| {
    eprint!("Failed to connect to Gemma3: {}", err);
    actix_web::error::ErrorInternalServerError("Failed to connect to Gemma3")
  })?
  // Reads the json data to prepare to parse
  .text()
  .await
  .map_err(|err| {
    eprint!("Failed to read Gemma3 response: {}", err);
    actix_web::error::ErrorInternalServerError("Failed to parse Gemma3 Response")
  })?;

  // Parses the response to extract the AI text
  let api_response : serde_json::Value = serde_json::from_str(&response)
  .map_err(|err| {
    eprint!("Failed to parse Gemma3 response: {}", err);
    actix_web::error::ErrorInternalServerError("Failed to parse Gemma3 response")
  })?;

  let response_text = &api_response["response"];

  Ok(response_text.to_string())
}