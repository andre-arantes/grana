use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cycle {
    Monthly,
    Yearly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriceChange {
    pub cents: i64,
    pub date: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subscription {
    pub name: String,
    pub cycle: Cycle,
    /// Oldest first; the last entry is the current price. Never empty.
    pub prices: Vec<PriceChange>,
    pub archived: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    TheyOweMe,
    IOweThem,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Debt {
    pub id: u32,
    pub person: String,
    pub cents: i64,
    pub description: String,
    pub date: NaiveDate,
    pub direction: Direction,
    pub paid_on: Option<NaiveDate>,
}

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Data {
    pub subscriptions: Vec<Subscription>,
    pub debts: Vec<Debt>,
    pub next_id: u32,
}
