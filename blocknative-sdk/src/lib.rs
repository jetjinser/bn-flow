use http_req::request;
use serde::{Deserialize, Serialize};

const BN_API_PREFIX: &str = "https://blocknative-flows.shuttleapp.rs/api";

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
    pub from: String,
    pub to: String,
    // nonce: u128,
    // gas: u128,
    // gas_price: String,
    // gas_price_gwei: u128,
    // gas_used: Option<u128>, // present only when the tx is on-chain,
    // base_fee_per_gas: String,
    // base_fee_per_gas_gwei: u128,
    // max_priority_fee_per_gas: String,
    // max_priority_fee_per_gas_gwei: u128,
    // max_fee_per_gas: String,
    // max_fee_per_gas_gwei: u128,
    // #[serde(rename = "type")]
    // bn_type: u128,
    // value: String,
    // hash: String,
    // input: String,
    // v: String,
    // r: String,
    // s: String,
    // block_hash: Option<String>, // or null when status is 'pending',
    // blocku128: Option<String>,  // or null when status is 'pending',
    // estimated_blocks_until_confirmed: Option<u128>, // or null for estimates of 1-5
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
