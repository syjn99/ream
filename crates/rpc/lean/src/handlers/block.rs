use actix_web::{
    HttpResponse, Responder, get,
    web::{Data, Path},
};
use ream_api_types_beacon::error::ApiError;
use ream_api_types_lean::block_id::BlockID;
use ream_chain_lean::lean_chain::LeanChainReader;

// GET /lean/v0/blocks/{block_id}
#[get("/blocks/{block_id}")]
pub async fn get_block(
    block_id: Path<BlockID>,
    lean_chain: Data<LeanChainReader>,
) -> Result<impl Responder, ApiError> {
    // Obtain read guard first from the reader.
    let lean_chain = lean_chain.read().await;

    Ok(HttpResponse::Ok().json(
        match block_id.into_inner() {
            BlockID::Finalized => {
                lean_chain.get_block_by_root(lean_chain.latest_finalized_hash().ok_or(
                    ApiError::InternalError(format!("Failed to get latest finalized hash")),
                )?)
            }
            BlockID::Genesis => lean_chain.get_block_by_root(lean_chain.genesis_hash),
            BlockID::Head => lean_chain.get_block_by_root(lean_chain.head),
            BlockID::Justified => {
                lean_chain.get_block_by_root(lean_chain.latest_justified_hash().ok_or(
                    ApiError::InternalError(format!("Failed to get latest justified hash")),
                )?)
            }
            // TODO: Implement fetching block by slot
            BlockID::Slot(_slot) => None,
            BlockID::Root(root) => lean_chain.get_block_by_root(root),
        }
        .ok_or_else(|| ApiError::NotFound(format!("Block not found")))?,
    ))
}
