use serde_json::json;
use serde_json::Value;
use std::net::TcpStream;
use std::sync::mpsc::sync_channel;
use std::thread;
use std::time::Duration;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::Message::Text;
use tungstenite::{connect, WebSocket};

use crate::limit_order_book::limit_order_book::LimitOrderBook;
use crate::order::order::Data;

pub mod limit_order_book;
pub mod order;

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
    thread::spawn( move || {
        let mut order_book = LimitOrderBook::new();
        while order_book.event_listener(ws_connect(&api), sender.clone()) == false {
            println!("Reconnecting to Deribit.");
        }
    });

    loop {
        match receiver.try_recv() {
            Ok(data) => {
                println!("Best Bid: Price: ${}.{}, orderSize ${}", data[0].0, data[0].1, data[0].2);
                println!("Best Ask: Price: ${}.{}, orderSize ${}", data[1].0, data[1].1, data[1].2);
                thread::sleep(Duration::from_secs(1));
            }
            Err(_) => {
                println!("Error reading channel buffer."); }
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
