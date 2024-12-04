use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Transaction {
    pub r#type: String,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<f64>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ClientAccount {
    pub client: u16,
    pub available_amount: f64,
    pub held_amount: f64,
    pub total_amount: f64,
    pub locked: bool,
}