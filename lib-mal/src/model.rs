use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct AnimeList {
    pub data: Vec<ListNode>,
    #[serde(flatten)]
    paging: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListNode {
    pub node: Show,
    pub list_status: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListStatus {
    pub status: String,
    pub num_episodes_watched: u32,
    pub score: u8,
    pub updated_at: String,
    pub is_rewatching: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Show {
    pub id: i32,
    pub title: String,
    pub main_picture: HashMap<String, Value>,
}
