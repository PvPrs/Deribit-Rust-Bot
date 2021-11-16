mod channel_handler;

use crate::channel_handler::{Channels, Params, Subscription};

static DERIBIT_WS: &str = "wss://test.deribit.com/ws/api/v2";

fn main() {
    let (mut socket, _) = Channels::connect(DERIBIT_WS);

    let subscription: Subscription = Subscription::new(
        "2.0".to_string(),
        42,
        "public/subscribe".to_string(),
        Params::new(vec![Channels::new(
            "ETH-PERPETUAL".to_string(),
            "100".to_string(),
            1,
            "100ms".to_string(),
        )
        .to_string()]),
    );

    let msg = serde_json::to_string(&subscription).unwrap();
    let _init = socket.write_message(tungstenite::Message::Text(msg));

    loop {
        let res = socket.read_message().expect("Error reading message");
        let _res = match res {
            tungstenite::Message::Text(s) => println!("Res: {}", s.to_string()),
            _ => {
                panic!("Error getting text");
            }
        };
    }
}
