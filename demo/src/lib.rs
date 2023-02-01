use blocknative_flows::listen_to_address;
use slack_flows::send_message_to_channel;

#[no_mangle]
pub fn run() {
    let address = "0xC8a8f0C656D21bd619FB06904626255af19663ff";

    listen_to_address(address, |bnm| {
        send_message_to_channel("ham-5b68442", "general", bnm.hash);
    });
}
