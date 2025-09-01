use actix_web::{HttpResponse, Responder, get, web::Data};
use ream_api_types_beacon::error::ApiError;
use ream_api_types_lean::head::Head;
use ream_chain_lean::lean_chain::LeanChainReader;

// GET /lean/v0/head
#[get("/head")]
pub async fn get_head(lean_chain: Data<LeanChainReader>) -> Result<impl Responder, ApiError> {
    Ok(HttpResponse::Ok().json(Head {
        head: lean_chain.read().await.head,
    }))
}
