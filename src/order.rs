use serde::{Deserialize, Serialize};

pub mod order {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub struct Data {
        #[serde(rename = "type")]
        pub order_type: String,
        pub timestamp: i64,
        pub prev_change_id: Option<i64>,
        pub instrument_name: String,
        pub change_id: i64,
        pub bids: Vec<(String, f64, f64)>,
        pub asks: Vec<(String, f64, f64)>,
    }

    impl Data {
        pub fn get_prev_change_id(&self) -> Option<i64> {
            self.prev_change_id
        }
        pub fn get_change_id(&self) -> i64 {
            self.change_id
        }

        pub fn get_bids(&self) -> &Vec<(String, f64, f64)> {
            &self.bids
        }
        pub fn get_asks(&self) -> &Vec<(String, f64, f64)> {
            &self.asks
        }
    }
}
