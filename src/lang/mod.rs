use serde::{Deserialize, Serialize};

pub mod markdown;

#[derive(Serialize, Deserialize)]
pub struct Position {
    /// Column
    pub col: u32,
    /// Row
    pub row: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

pub trait Parser<L> {}
