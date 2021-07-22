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
    pub node: Anime,
    pub list_status: ListStatus,
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
pub struct Anime {
    pub id: u32,
    pub title: String,
    pub main_picture: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnimeDetails {
    #[serde(flatten)]
    pub show: Anime,
    pub alternative_titles: Option<AlternativeTitles>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub synopsis: Option<String>,
    pub mean: Option<f32>,
    pub rank: Option<u32>,
    pub num_list_users: Option<u32>,
    pub num_scoring_users: Option<u32>,
    pub nsfw: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub media_type: Option<String>,
    pub status: Option<String>,
    pub genres: Option<Vec<HashMap<String, Value>>>,
    pub my_list_status: Option<ListStatus>,
    pub num_episodes: Option<u32>,
    pub start_season: Option<HashMap<String, Value>>,
    pub broadcast: Option<HashMap<String, String>>,
    pub source: Option<String>,
    pub average_episode_duration: Option<u32>,
    pub rating: Option<String>,
    pub pictures: Option<Vec<HashMap<String, String>>>,
    pub background: Option<String>,
    pub related_anime: Option<Vec<Related>>,
    pub related_manga: Option<Vec<HashMap<String, Value>>>,
    pub recommendations: Option<Vec<Recommnendation>>,
    pub studios: Option<Vec<HashMap<String, Value>>>,
    pub statistics: Option<Stats>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Stats {
    pub status: HashMap<String, String>,
    pub num_list_users: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AlternativeTitles {
    pub synonyms: Vec<String>,
    #[serde(flatten)]
    pub languages: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Related {
    pub node: Anime,
    pub relation_type: String,
    pub relation_type_formatted: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Recommnendation {
    pub node: Anime,
    pub num_recommendations: u32,
}
