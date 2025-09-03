use actix_web::web::ServiceConfig;

use crate::handlers::{block::get_block, head::get_head};

/// Creates and returns all `/lean` routes.
pub fn register_lean_routes(cfg: &mut ServiceConfig) {
    cfg.service(get_head).service(get_block);
}
