use serde_json::json;
use serde_json::Value;
use std::net::TcpStream;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::Message::Text;
use tungstenite::{connect, WebSocket};

use crate::limit_order_book::limit_order_book::LimitOrderBook;
use crate::order::order::Data;

pub mod limit_order_book;
pub mod order;

static DERIBIT_WS: &str = "wss://test.deribit.com/ws/api/v2";

fn main() {
    let api: Value = json!({
      "method": "public/subscribe",
      "params": {
        "channels": [
          "book.BTC-PERPETUAL.raw"
        ]
      },
      "jsonrpc": "2.0",
      "id": 8
    });

    while order_book_events(ws_connect(&api)) == false {
        println!("Reconnecting to Deribit.");
    }
}

/**
 * Set up a stream to the Deribit websocket.
 * @param api
 * @return WebSocket
 */
fn ws_connect(api: &Value) -> WebSocket<MaybeTlsStream<TcpStream>> {
    let (mut socket, _) = connect(DERIBIT_WS).expect("Could not connect to Deribit");
    println!("Connected to Deribit.");
    let message = serde_json::to_string(&api).unwrap();
    let _init = socket.write_message(Text(message));
    socket
}

/**
 * Listen for order book events.
 * @param socket
 * @return bool false = reconnect
 */
fn order_book_events(mut socket: WebSocket<MaybeTlsStream<TcpStream>>) -> bool {
    let mut order_book: LimitOrderBook = LimitOrderBook::new();
    let mut prev_id: i64 = 0;
    loop {
        let response = socket.read_message().expect("Error reading message");
        match response {
            Text(s) => {
                if !s.contains("change_id") {
                    continue;
                }
                let msg: Value = serde_json::from_str(&s).expect("Error parsing message");
                let orders: Data = serde_json::from_str(&msg["params"]["data"].to_string())
                    .expect("Error parsing object");
                if orders.prev_change_id.is_some() {
                    if prev_id != orders.prev_change_id.unwrap() {
                        return false;
                    }
                }
                prev_id = orders.change_id;
                order_book.add_orders(orders);

                let bid = order_book.get_best_bid();
                let ask = order_book.get_best_ask();

                std::thread::sleep(std::time::Duration::from_secs(1));
                println!("Best Bid: Price: {}.{} quantity: {}", bid.0, bid.1, bid.2);
                println!("Best Ask: Price: {}.{} quantity: {}", ask.0, ask.1, ask.2);
            }
            _error => {
                panic!("Error getting text");
            }
        };
    }
}
