use http_req::request;
use serde::{Deserialize, Serialize};

// const BN_API_PREFIX: &str = "https://blocknative-flows.shuttleapp.rs/api";
const BN_API_PREFIX: &str = "https://bn-flows-js.vercel.app/api";

extern "C" {
    // Flag if current running is for listening(1) or message receving(0)
    fn is_listening() -> i32;

    // Return the user id of the flows platform
    fn get_flows_user(p: *mut u8) -> i32;

    // Return the flow id
    fn get_flow_id(p: *mut u8) -> i32;

    fn get_event_body_length() -> i32;
    fn get_event_body(p: *mut u8) -> i32;
    fn set_error_log(p: *const u8, len: i32);
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub status: String,
    pub monitor_id: String,
    pub monitor_version: String,
    pub time_pending: Option<String>,
    pub blocks_pending: Option<i64>,
    pub pending_time_stamp: String,
    pub pending_block_number: i64,
    pub hash: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub gas: i64,
    pub nonce: i64,
    pub block_hash: Option<String>,
    pub block_number: Option<i64>,
    pub v: String,
    pub r: String,
    pub s: String,
    pub input: String,
    pub gas_used: Option<i64>,
    #[serde(rename = "type")]
    pub type_field: i64,
    pub max_fee_per_gas: String,
    pub max_fee_per_gas_gwei: f64,
    pub max_priority_fee_per_gas: String,
    pub max_priority_fee_per_gas_gwei: f64,
    pub base_fee_per_gas: Option<String>,
    pub base_fee_per_gas_gwei: Option<f64>,
    pub transaction_index: Option<i64>,
    pub asset: String,
    pub block_time_stamp: Option<String>,
    pub watched_address: String,
    pub direction: String,
    pub counterparty: String,
    pub server_version: String,
    pub event_code: String,
    pub time_stamp: String,
    pub dispatch_timestamp: String,
    pub system: String,
    pub network: String,
}

pub fn revoke_listeners() {
    unsafe {
        let mut flows_user = Vec::<u8>::with_capacity(100);
        let c = get_flows_user(flows_user.as_mut_ptr());
        flows_user.set_len(c as usize);
        let flows_user = String::from_utf8(flows_user).unwrap();

        let mut flow_id = Vec::<u8>::with_capacity(100);
        let c = get_flow_id(flow_id.as_mut_ptr());
        if c == 0 {
            panic!("Failed to get flow id");
        }
        flow_id.set_len(c as usize);
        let flow_id = String::from_utf8(flow_id).unwrap();

        let mut writer = Vec::new();
        let res = request::get(
            format!("{}/{}/{}/revoke", BN_API_PREFIX, flows_user, flow_id),
            &mut writer,
        )
        .unwrap();

        match res.status_code().is_success() {
            true => (),
            false => {
                set_error_log(writer.as_ptr(), writer.len() as i32);
            }
        }
    }
}

pub fn event_received(address: &str) -> Option<Event> {
    unsafe {
        match is_listening() {
            // Calling register
            1 => {
                let mut flows_user = Vec::<u8>::with_capacity(100);
                let c = get_flows_user(flows_user.as_mut_ptr());
                flows_user.set_len(c as usize);
                let flows_user = String::from_utf8(flows_user).unwrap();

                let mut flow_id = Vec::<u8>::with_capacity(100);
                let c = get_flow_id(flow_id.as_mut_ptr());
                if c == 0 {
                    panic!("Failed to get flow id");
                }
                flow_id.set_len(c as usize);
                let flow_id = String::from_utf8(flow_id).unwrap();

                let mut writer = Vec::new();
                let res = request::get(
                    format!(
                        "{}/{}/{}/listen?address={}",
                        BN_API_PREFIX, flows_user, flow_id, address
                    ),
                    &mut writer,
                )
                .unwrap();

                match res.status_code().is_success() {
                    true => serde_json::from_slice::<Event>(&writer).ok(),
                    false => {
                        set_error_log(writer.as_ptr(), writer.len() as i32);
                        None
                    }
                }
            }
            _ => event_from_subcription(),
        }
    }
}

fn event_from_subcription() -> Option<Event> {
    unsafe {
        let l = get_event_body_length();
        let mut event_body = Vec::<u8>::with_capacity(l as usize);
        let c = get_event_body(event_body.as_mut_ptr());
        assert!(c == l);
        event_body.set_len(c as usize);
        match serde_json::from_slice::<Event>(&event_body) {
            Ok(e) => Some(e),
            Err(_) => None,
        }
    }
}
