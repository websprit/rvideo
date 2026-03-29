use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::MySqlPool;
use std::collections::HashMap;

#[derive(Clone)]
pub struct AppConfig {
    pub auth_secret: String,
    pub subscription_sources: String,
    pub ad_keywords: Vec<String>,
    pub access_password: Option<String>,
    pub persist_password: bool,
    pub is_production: bool,
}

#[derive(Clone)]
pub struct AppState {
    pub db: MySqlPool,
    pub http: Client,
    pub config: AppConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoSource {
    pub id: String,
    pub name: String,
    #[serde(rename = "baseUrl")]
    pub base_url: String,
    #[serde(rename = "searchPath")]
    pub search_path: String,
    #[serde(rename = "detailPath")]
    pub detail_path: String,
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub priority: Option<i64>,
    #[serde(default)]
    pub group: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VideoItem {
    #[serde(default)]
    pub vod_id: JsonValue,
    #[serde(default)]
    pub vod_name: String,
    #[serde(default)]
    pub vod_pic: Option<String>,
    #[serde(default)]
    pub type_name: Option<String>,
    #[serde(default)]
    pub vod_remarks: Option<String>,
    #[serde(default)]
    pub vod_year: Option<String>,
    #[serde(default)]
    pub vod_area: Option<String>,
    #[serde(default)]
    pub vod_actor: Option<String>,
    #[serde(default)]
    pub vod_director: Option<String>,
    #[serde(default)]
    pub vod_content: Option<String>,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub latency: Option<u128>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub name: String,
    pub url: String,
    pub index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoDetail {
    pub vod_id: JsonValue,
    pub vod_name: String,
    pub vod_pic: String,
    #[serde(default)]
    pub vod_remarks: Option<String>,
    #[serde(default)]
    pub vod_year: Option<String>,
    #[serde(default)]
    pub vod_area: Option<String>,
    #[serde(default)]
    pub vod_actor: Option<String>,
    #[serde(default)]
    pub vod_director: Option<String>,
    #[serde(default)]
    pub vod_content: Option<String>,
    #[serde(default)]
    pub type_name: Option<String>,
    pub episodes: Vec<Episode>,
    pub source: String,
    pub source_code: String,
}

#[derive(Debug, Deserialize)]
pub struct ApiSearchResponse {
    pub code: i64,
    #[serde(default)]
    pub msg: Option<String>,
    #[serde(default)]
    pub list: Vec<VideoItem>,
}

#[derive(Debug, Deserialize)]
pub struct ApiDetailResponse {
    pub code: i64,
    #[serde(default)]
    pub msg: Option<String>,
    #[serde(default)]
    pub list: Vec<ApiDetailItem>,
}

#[derive(Debug, Deserialize)]
pub struct ApiDetailItem {
    pub vod_id: JsonValue,
    pub vod_name: String,
    pub vod_pic: String,
    #[serde(default)]
    pub vod_remarks: Option<String>,
    #[serde(default)]
    pub vod_year: Option<String>,
    #[serde(default)]
    pub vod_area: Option<String>,
    #[serde(default)]
    pub vod_actor: Option<String>,
    #[serde(default)]
    pub vod_director: Option<String>,
    #[serde(default)]
    pub vod_content: Option<String>,
    #[serde(default)]
    pub type_name: Option<String>,
    #[serde(default)]
    pub vod_play_from: Option<String>,
    #[serde(default)]
    pub vod_play_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamCategory {
    pub type_id: JsonValue,
    pub type_name: String,
}
