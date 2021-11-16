use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result};
use std::net::TcpStream;
use tungstenite::handshake::client::Response;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, WebSocket};
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Channels {
    instrument_name: String,
    group: String,
    depth: i32,
    interval: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Params {
    channels: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Subscription {
    jsonrpc: String,
    method: String,
    id: i32,
    params: Params,
}

impl Params {
    pub fn new(channels: Vec<String>) -> Self {
        Params { channels }
    }
}

impl Channels {
    pub fn new(instrument_name: String, group: String, depth: i32, interval: String) -> Self {
        Channels {
            instrument_name,
            group,
            depth,
            interval,
        }
    }

    pub fn connect(url: &str) -> (WebSocket<MaybeTlsStream<TcpStream>>, Response) {
        let url = Url::parse(url).unwrap();
        let (socket, response) = connect(url).expect("Could not connect to Deribit");
        println!("Connected to Deribit");
        (socket, response)
    }
}

impl Display for Channels {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "book.{}.{}.{}.{}",
            self.instrument_name, self.group, self.depth, self.interval
        )
    }
}

impl Subscription {
    pub fn new(jsonrpc: String, id: i32, method: String, params: Params) -> Self {
        Subscription {
            jsonrpc,
            method,
            id,
            params,
        }
    }
}
