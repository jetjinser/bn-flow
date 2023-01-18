use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use shuttle_service::error::CustomError;
use sqlx::{Executor, FromRow, PgPool};
use sync_wrapper::SyncWrapper;

#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
struct FlowQuery {
    address: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
struct FlowTrigger {
    flow_id: String,
    flow_user: String,
}

async fn listen(
    Path((flow_user, flow_id)): Path<(String, String)>,
    Query(FlowQuery { address }): Query<FlowQuery>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    let sql = "INSERT INTO bn_trigger (flow_user, flow_id, address) VALUES (?, ?, ?)";
    let result = sqlx::query(sql)
        .bind(flow_id)
        .bind(flow_user)
        .bind(address)
        .execute(&pool)
        .await;

    match result {
        Ok(_) => (
            StatusCode::OK,
            serde_json::json!({
                "from": "test from",
                "to": "test to",
            })
            .to_string(),
        ),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

async fn revoke(
    Path((flow_user, flow_id)): Path<(String, String)>,
    State(pool): State<PgPool>,
) -> StatusCode {
    let sql = "DELETE FROM bn_trigger WHERE flow_user = ? AND flow_id = ?";
    let result = sqlx::query(sql)
        .bind(flow_id)
        .bind(flow_user)
        .execute(&pool)
        .await;

    match result {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn event(Path(address): Path<String>, State(pool): State<PgPool>) -> impl IntoResponse {
    let sql = "SELECT flow_id, flow_user FROM bn_trigger WHERE address = ?";
    let all_flows = sqlx::query_as::<_, FlowTrigger>(sql)
        .bind(address)
        .fetch_all(&pool)
        .await;

    if let Ok(afs) = all_flows {
        (StatusCode::OK, serde_json::to_string(&afs).unwrap())
    } else {
        (
            StatusCode::NOT_FOUND,
            String::from("No flow binding with the channel"),
        )
    }
}

#[shuttle_service::main]
async fn axum(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_service::ShuttleAxum {
    pool.execute(include_str!("../schema.sql"))
        .await
        .map_err(CustomError::new)?;

    let router = Router::new()
        .route("/api/:flows_user/:flow_id/listen", get(listen))
        .route("/api/:flows_user/:flow_id/revoke", get(revoke))
        .route("/api/event/:address", get(event))
        .route("/echo", get(|s: String| async { s }))
        .with_state(pool);
    let sync_wrapper = SyncWrapper::new(router);

    Ok(sync_wrapper)
}
