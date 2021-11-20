use serde_json::Value;
use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::mpsc::SyncSender;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::Message::Text;
use tungstenite::WebSocket;

use crate::Data;

pub mod limit_order_book {
    use super::*;

    /**
     * Limit order book, f64 = orderSize / quantity
     */
    pub struct LimitOrderBook {
        bids: HashMap<PriceLevel, f64>,
        asks: HashMap<PriceLevel, f64>,
    }

    /**
     * PriceLevel represents the integral and fractional parts of a price.
     */
    #[derive(Debug, Hash, Eq, PartialEq)]
    struct PriceLevel {
        integral: u64,
        decimal: u64,
    }

    impl PriceLevel {
        pub fn new(value: f64) -> PriceLevel {
            PriceLevel {
                integral: value.trunc() as u64,
                decimal: (value.fract() * 100.0) as u64,
            }
        }
    }

    impl LimitOrderBook {
        pub fn new() -> LimitOrderBook {
            LimitOrderBook {
                bids: HashMap::new(),
                asks: HashMap::new(),
            }
        }

        /**
         * Add new orders to the limit order book.
         */
        pub fn add_orders(&mut self, order: Data) {
            order.get_bids().iter().for_each(|bid| {
                if bid.0 == "delete" {
                    self.bids.remove(&PriceLevel::new(bid.1));
                } else {
                    self.bids.insert(PriceLevel::new(bid.1), bid.2);
                }
            });

            order.get_asks().iter().for_each(|ask| {
                if ask.0 == "delete" {
                    self.bids.remove(&PriceLevel::new(ask.1));
                } else {
                    self.asks.insert(PriceLevel::new(ask.1), ask.2);
                }
            });
        }

        pub fn get_best_bid(&self) -> (u64, u64, f64) {
            let mut best_bid = (0, 0, 0.0);
            for (price_level, quantity) in self.bids.iter() {
                if best_bid.0 < price_level.integral
                    || best_bid.0 <= price_level.integral && best_bid.1 < price_level.decimal
                {
                    best_bid = (price_level.integral, price_level.decimal, *quantity);
                }
            }
            best_bid
        }

        pub fn get_best_ask(&self) -> (u64, u64, f64) {
            let mut best_ask = (0, 0, 0.0);
            for (price_level, quantity) in self.asks.iter() {
                if best_ask.0 > price_level.integral
                    || best_ask.0 == 0
                    || best_ask.0 >= price_level.integral && best_ask.1 > price_level.decimal
                {
                    best_ask = (price_level.integral, price_level.decimal, *quantity);
                }
            }
            best_ask
        }

        pub fn event_listener(
            &mut self,
            mut socket: WebSocket<MaybeTlsStream<TcpStream>>,
            sender: SyncSender<Vec<(u64, u64, f64)>>,
        ) -> bool {
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
                        self.add_orders(orders);
                        let best = vec![self.get_best_bid(), self.get_best_ask()];
                        match sender.try_send(best).is_err() {
                            _ => (),
                        }
                    }
                    _error => {
                        panic!("Error getting text");
                    }
                };
            }
        }
    }
}
