use super::constants::SkyMap;
use chrono::{DateTime, Utc};
use chrono_tz::{America::Los_Angeles, Tz};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ShardEruptionRawDates {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct ShardEruptionRawResponse {
    pub realm: String,
    pub sky_map: SkyMap,
    pub strong: bool,
    pub reward: f32,
    pub timestamps: Vec<ShardEruptionRawDates>,
    pub url: String,
}

#[derive(Clone, Debug)]
pub struct ShardEruptionDates {
    pub start: DateTime<Tz>,
    pub end: DateTime<Tz>,
}

#[derive(Clone, Debug)]
pub struct ShardEruptionResponse {
    pub realm: String,
    pub sky_map: SkyMap,
    pub strong: bool,
    pub reward: f32,
    pub timestamps: Vec<ShardEruptionDates>,
    pub url: String,
}

pub async fn shard_eruption(url: &String) -> Option<ShardEruptionResponse> {
    let data = reqwest::get(format!("{url}/shard-eruption"))
        .await
        .expect("Failed to fetch the shard eruption.")
        .json::<Option<ShardEruptionRawResponse>>()
        .await
        .expect("Failed to parse the shard eruption.");

    if let Some(raw_data) = data {
        Some(ShardEruptionResponse {
            realm: raw_data.realm,
            sky_map: raw_data.sky_map,
            strong: raw_data.strong,
            reward: raw_data.reward,
            timestamps: raw_data
                .timestamps
                .iter()
                .map(|timestamp| ShardEruptionDates {
                    start: timestamp.start.with_timezone(&Los_Angeles),
                    end: timestamp.end.with_timezone(&Los_Angeles),
                })
                .collect(),
            url: raw_data.url,
        })
    } else {
        None
    }
}
