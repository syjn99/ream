use actix_web::{
    HttpResponse, Responder, get,
    web::{Data, Path},
};
use ream_api_types_beacon::error::ApiError;
use ream_api_types_common::id::ID;
use ream_chain_lean::lean_chain::LeanChainReader;

// GET /lean/v0/blocks/{block_id}
#[get("/blocks/{block_id}")]
pub async fn get_block(
    block_id: Path<ID>,
    lean_chain: Data<LeanChainReader>,
) -> Result<impl Responder, ApiError> {
    // Obtain read guard first from the reader.
    let lean_chain = lean_chain.read().await;

    Ok(HttpResponse::Ok().json(
        match block_id.into_inner() {
            ID::Finalized => {
                lean_chain.get_block_by_root(lean_chain.latest_finalized_hash().ok_or(
                    ApiError::InternalError("Failed to get latest finalized hash".to_string()),
                )?)
            }
            ID::Genesis => lean_chain.get_block_by_root(lean_chain.genesis_hash),
            ID::Head => lean_chain.get_block_by_root(lean_chain.head),
            ID::Justified => {
                lean_chain.get_block_by_root(lean_chain.latest_justified_hash().ok_or(
                    ApiError::InternalError("Failed to get latest justified hash".to_string()),
                )?)
            }
            ID::Slot(slot) => lean_chain.get_block_by_slot(slot),
            ID::Root(root) => lean_chain.get_block_by_root(root),
        }
        .ok_or_else(|| ApiError::NotFound("Block not found".to_string()))?,
    ))
}
