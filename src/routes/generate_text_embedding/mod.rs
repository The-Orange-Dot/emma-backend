use actix_web::{web, post, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
struct Payload {
  prompt: String
}

#[derive(Deserialize, Serialize, Debug)]
struct EmbedBody {
  model: String,
  prompt: String,
  stream: bool  
}

#[derive(Serialize, Deserialize, Debug)]
struct EmbeddingResponse {
  embedding: Vec<f64>
}

#[post("/shopify-embed-text")]
async fn generate_text_embedding (
  payload: web::Json<Payload>
) -> Result<HttpResponse, actix_web::Error> {
  let body = payload.into_inner();
  let client = reqwest::Client::new();
  let url = std::env::var("SERVER_URL").expect("Could not find server url");
  let model = "nomic-embed-text:latest";

  let req_data = EmbedBody {
    model: model.to_string(),
    prompt: body.prompt,
    stream: false
  };

  let response = client.post(format!("{}/api/embeddings", &url))
    .header("Content-Type", "application/json")
    .body(serde_json::to_string(&req_data).unwrap())
    .send()
    .await
    .map_err(|err| {
      eprint!("Error creating embeddings: {}", err);
      return actix_web::error::ErrorInternalServerError(format!("Error creating embeddings: {}", err))
    })?;  

  let parsed_response = response.text()
    .await
    .map_err(|err| {
      println!("Failed to parse response text: {}", err);
      actix_web::error::ErrorInternalServerError(format!("Error parsing response: {}", err))
    })?;    

  let json_response: EmbeddingResponse = serde_json::from_str(&parsed_response)
    .map_err(|err| {
      println!("Failed to converting response text to json: {}", err);
      actix_web::error::ErrorInternalServerError(format!("Error converting response text to json: {}", err))
    }).unwrap();

  Ok(HttpResponse::Ok().json(serde_json::json!({
    "status": 200,
    "message": "embedding sent successfully",
    "data": json_response.embedding
  })))
}