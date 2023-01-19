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
    flows_user: String,
}

async fn listen(
    Path((flows_user, flow_id)): Path<(String, String)>,
    Query(FlowQuery { address }): Query<FlowQuery>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    let sql = "INSERT INTO bn_trigger (flows_user, flow_id, address) VALUES ($1, $2, $3)";
    let result = sqlx::query(sql)
        .bind(flows_user)
        .bind(flow_id)
        .bind(address)
        .execute(&pool)
        .await;

    match result {
        Ok(_) => (
            StatusCode::OK,
            serde_json::json!({
                "status": "pending",
                "monitorId": "ec2_ue1_c_prod_bn_monitor_eth_goerli_1",
                "monitorVersion": "0.117.1",
                "pendingTimeStamp": "2023-01-19T05:04:38.782Z",
                "pendingBlockNumber": 8336598,
                "hash": "0xe39c510d76e5811d526355ef7a85bbb3ba79f0c735a048d61f9bba8b8c59f1cd",
                "from": "0x1291351b8Aa33FdC64Ac77C8302Db523d5B43AeF",
                "to": "0xC8a8f0C656D21bd619FB06904626255af19663ff",
                "value": "0",
                "gas": 21000,
                "nonce": 40,
                "blockHash": null,
                "blockNumber": null,
                "v": "0x1",
                "r": "0x39fcab9a97034327f643c9ab684c5cf961a4432faa135f427cfd48ed5aadfa7",
                "s": "0x4c7908431a4059d46b6bbb31d4c4fd2713b399181d6677314bd26381a6b1dd7d",
                "input": "0x",
                "type": 2,
                "maxFeePerGas": "1500000017",
                "maxFeePerGasGwei": 1.5,
                "maxPriorityFeePerGas": "1500000000",
                "maxPriorityFeePerGasGwei": 1.5,
                "transactionIndex": null,
                "asset": "ETH",
                "watchedAddress": "0xc8a8f0c656d21bd619fb06904626255af19663ff",
                "direction": "incoming",
                "counterparty": "0x1291351b8Aa33FdC64Ac77C8302Db523d5B43AeF",
                "serverVersion": "0.158.1",
                "eventCode": "txPool",
                "timeStamp": "2023-01-19T05:04:38.782Z",
                "dispatchTimestamp": "2023-01-19T05:04:38.825Z",
                "system": "ethereum",
                "network": "goerli"
            })
            .to_string(),
        ),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

async fn revoke(
    Path((flows_user, flow_id)): Path<(String, String)>,
    State(pool): State<PgPool>,
) -> StatusCode {
    let sql = "DELETE FROM bn_trigger WHERE flows_user = $1 AND flow_id = $2";
    let result = sqlx::query(sql)
        .bind(flows_user)
        .bind(flow_id)
        .execute(&pool)
        .await;

    match result {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn event(Path(address): Path<String>, State(pool): State<PgPool>) -> impl IntoResponse {
    let sql = "SELECT flow_id, flows_user FROM bn_trigger WHERE address = $1";
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
        .with_state(pool);
    let sync_wrapper = SyncWrapper::new(router);

    Ok(sync_wrapper)
}
