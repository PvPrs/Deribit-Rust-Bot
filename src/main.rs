use serde_json::json;
use serde_json::Value;
use std::net::TcpStream;
use std::sync::mpsc::{sync_channel, SyncSender};
use std::thread;
use std::time::Duration;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::Message::Text;
use tungstenite::{connect, WebSocket};

use crate::limit_order_book::limit_order_book::LimitOrderBook;
use crate::order::order::Data;

pub mod limit_order_book;
pub mod order;

/**
   A product grade version would simply require an iteration through
   serde_json::from_str(&msg["params"]["data"]["bids"].to_string()) as f64

   and a conditional statement to check whether a value is a higher or lower
   than the current best value

   no storage of any data should be required.

   # Edit: fn product() { is a product grade version
*/

static DERIBIT_WS: &str = "wss://www.deribit.com/ws/api/v2";

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

    let (sender, receiver) = sync_channel(1);
    thread::spawn(move || {
        let mut order_book = LimitOrderBook::new();
        //while order_book.event_listener(ws_connect(&api), sender.clone()) == false {
        while product(ws_connect(&api), sender.clone()) == false {
            println!("Reconnecting to Deribit.");
        }
    });

    loop {
        match receiver.try_recv() {
            Ok(data) => {
                println!("Best Bid: Price: ${}", data[0]);
                println!("Best Ask: Price: ${}", data[1]);
                // println!(
                //     "Best Bid: Price: ${}.{}, orderSize ${}",
                //     data[0].0, data[0].1, data[0].2
                // );
                // println!(
                //     "Best Ask: Price: ${}.{}, orderSize ${}",
                //     data[1].0, data[1].1, data[1].2
                // );
                thread::sleep(Duration::from_secs(1));
            }
            Err(_) => {
                println!("Error reading empty channel buffer.");
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
}

/**
    product grade version
*/
fn product(mut socket: WebSocket<MaybeTlsStream<TcpStream>>, sender: SyncSender<Vec<f64>>) -> bool {
    let mut prev_id: Option<f64> = None;
    let mut best_bid: f64 = 0.0;
    let mut best_ask: f64 = 0.0;

    loop {
        let response = socket.read_message().expect("Error reading message");
        match response {
            Text(s) => {
                if !s.contains("change_id") {
                    continue;
                }
                let msg: Value = serde_json::from_str(&s).expect("Error parsing message");
                if msg["params"]["data"]["prev_change_id"].is_f64() {
                    if prev_id != msg["params"]["data"]["prev_change_id"].as_f64() {
                        return false;
                    }
                }
                prev_id = msg["params"]["data"]["change_id"].as_f64();
                msg["params"]["data"]["bids"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .for_each(|bid| {
                        if bid[0] != "delete" && best_bid < bid[1].as_f64().unwrap() {
                            best_bid = bid[1].as_f64().unwrap();
                        }
                    });
                msg["params"]["data"]["asks"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .for_each(|ask| {
                        if (ask[0] != "delete")
                            && (best_ask > ask[1].as_f64().unwrap() || best_ask == 0.0)
                        {
                            best_ask = ask[1].as_f64().unwrap();
                        }
                    });
                let best = vec![best_bid, best_ask];
                match sender.try_send(best).is_err() {
                    _ => (),
                }
            }
            _error => {
                return false;
            }
        }
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
