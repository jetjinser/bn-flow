use axum::{extract::Path, routing::get, Router};
use sync_wrapper::SyncWrapper;

async fn listen(Path((flows_user, flow_id)): Path<(String, String)>) -> String {
    format!("listening: {} / {}", flows_user, flow_id)
}

async fn revoke(Path((flows_user, flow_id)): Path<(String, String)>) -> String {
    format!("revoking: {} / {}", flows_user, flow_id)
}

#[shuttle_service::main]
async fn axum() -> shuttle_service::ShuttleAxum {
    let router = Router::new()
        .route("/api/:flows_user/:flow_id/listen", get(listen))
        .route("/api/:flows_user/:flow_id/revoke", get(revoke));
    let sync_wrapper = SyncWrapper::new(router);

    Ok(sync_wrapper)
}

