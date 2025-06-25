use actix_web::{post, web, HttpRequest, HttpResponse};
use serde_json;
use crate::models::pools_models::AdminPool;
use crate::helpers::target_pool::target_admin_pool;
use serde::{Serialize, Deserialize};
use reqwest;

#[derive(Serialize, Deserialize, Debug)]
struct EmbedRequestPayload {
data: Vec<EmbedData>
}

#[derive(Serialize, Deserialize, Debug)]
struct EmbedData {
  product_id: String,
  prompt: String,
  product_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct EmbedResponse {
  product_id: String,
  embedding: String,
  product_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct EmbedBody {
  prompt: String,
  model: String,
  stream: bool
}

#[derive(Serialize, Deserialize, Debug)]
struct EmbeddingResponse {
  embedding: Vec<f64>
}

#[post("/shopify-generate-embeddings")]
pub async fn shopify_generate_embeddings(
  req: HttpRequest,
  payload: web::Json<EmbedRequestPayload>,
  admin_pool: web::Data<AdminPool>
) -> Result<HttpResponse, actix_web::Error> {
    dotenv::dotenv().ok();
    let _admin_conn = target_admin_pool(admin_pool);
    let body = payload.into_inner();

    let client = reqwest::Client::new();
    let url = std::env::var("SERVER_URL").expect("Could not find server url");
    let model = "nomic-embed-text:latest";
    let data_vec = body.data;

    let mut response_vec : Vec<EmbedResponse> = Vec::new();

    for data in data_vec {
      let body = EmbedBody {
        model: model.to_string(),
        prompt: data.prompt,
        stream: false
      };

      let response = client.post(format!("{}/api/embeddings", &url))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&body).unwrap())
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
      let prefix_to_remove = "{\"embedding\":";
      let suffix_to_remove = "}";

      let suffuxed_removed = parsed_response.strip_prefix(prefix_to_remove).unwrap();
      let string_embedding = suffuxed_removed.strip_suffix(suffix_to_remove).unwrap();

      // let json_response: EmbeddingResponse = serde_json::from_str(&parsed_response)
      //   .map_err(|err| {
      //     println!("Failed to converting response text to json: {}", err);
      //     actix_web::error::ErrorInternalServerError(format!("Error converting response text to json: {}", err))
      //   }).unwrap();

        println!("RESPONSE: {:?}", string_embedding);

        let product_embedding = EmbedResponse { product_id: data.product_id, product_type: data.product_type, embedding: string_embedding.to_string() };

        response_vec.push(product_embedding);

    };

    Ok(HttpResponse::Ok().json(serde_json::json!({
      "status": 200,
      "message": "Successfully generated embeddings",
      "data": response_vec
    })))
}