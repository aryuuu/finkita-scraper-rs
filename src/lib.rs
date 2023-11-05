use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
pub struct Mutation {
    pub email: String,
    pub date: DateTime<Utc>,
    pub description: String,
    pub m_type: String,
    pub amount: i32,
    pub balance: i32,
    pub currency: String,
}

impl Mutation {
    pub fn new() -> Mutation {
        Mutation {
            email: String::from("malgahfattahillahi@gmail.com"),
            amount: 0,
            balance: 0,
            date: Utc::now(),
            description: String::from(""),
            m_type: String::from(""),
            currency: String::from("IDR"),
        }
    }
}

impl fmt::Display for Mutation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "email: {}, date: {}, description: {}, m_type: {}, amount: {}, balance: {}",
            self.email, self.date, self.description, self.m_type, self.amount, self.balance
        )
    }
}
