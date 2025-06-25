use actix_web::{post, web, HttpRequest, HttpResponse};
use serde_json;
use crate::models::pools_models::AdminPool;
use crate::helpers::target_pool::target_admin_pool;
use serde::{Serialize, Deserialize};
use reqwest;

#[derive(Serialize, Deserialize, Debug)]
struct EmbedRequestPayload {
  prompt: Vec<String>
}

#[derive(Serialize, Deserialize, Debug)]
struct EmbedBody {
  prompt: String,
  model: String,
  stream: bool
}

#[post("/shopify-generate-embeddings")]
pub async fn shopify_generate_embeddings(
  req: HttpRequest,
  payload: web::Json<EmbedRequestPayload>,
  admin_pool: web::Data<AdminPool>
) -> Result<HttpResponse, actix_web::Error> {
    let admin_conn = target_admin_pool(admin_pool);
    let body = payload.into_inner();

    println!("{:?}", body);

    let client = reqwest::Client::new();
    let url = std::env::var("SERVER_URL").expect("Could not find server url");
    let model = "nomic-embed-text:latest";
    let prompts = body.prompt;

    for prompt in prompts {
      let body = EmbedBody {
        model: model.to_string(),
        prompt: prompt,
        stream: false
      };

      let response = client.post(&url)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&body).unwrap())
        .send()
        .await
        .map_err(|err| {
          eprint!("Error creating embeddings: {}", err);
          return actix_web::error::ErrorInternalServerError(format!("Error creating embeddings: {}", err))
        })?;

        println!("RESPONSE: {:?}", response);
    }


    Ok(HttpResponse::Ok().json(serde_json::json!({
      "status": 200,
      "message": "Successfully generated embeddings",
      "data": {}
    })))
}