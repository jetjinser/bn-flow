use blocknative_flows::event_received;
use slack_flows::send_message_to_channel;

#[no_mangle]
pub fn run() {
    if let Some(bnm) = event_received("0xC8a8f0C656D21bd619FB06904626255af19663ff") {
        send_message_to_channel("ham-5b68442", "general", bnm.to);
    }
}
