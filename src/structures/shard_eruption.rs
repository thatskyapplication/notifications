use crate::utility::constants::{SkyMap, CDN_URL};
use chrono::{DateTime, Datelike, Duration, Utc};
use chrono_tz::Tz;

fn shard_eruption_map_url(sky_map: SkyMap) -> String {
    format!(
        "{}/daily_guides/shard_eruptions/{}.webp",
        CDN_URL,
        sky_map.to_string().to_lowercase().replace(' ', "_")
    )
}

struct ShardEruptionMapData {
    sky_map: SkyMap,
    url: String,
    reward: f32,
}

struct ShardEruptionData {
    no_shard_weekday: [u32; 2],
    interval: u32,
    offset: u32,
    area: Vec<ShardEruptionMapData>,
}

#[derive(Debug, Clone)]
pub struct ShardEruptionResponse {
    pub realm: String,
    pub sky_map: SkyMap,
    pub strong: bool,
    #[allow(dead_code)]
    reward: f32,
    pub timestamps: Vec<(DateTime<Tz>, DateTime<Tz>)>,
    pub url: String,
}

const SHARD_ERUPTION_REALM_NAMES: [&str; 5] = [
    "Daylight Prairie",
    "Hidden Forest",
    "Valley of Triumph",
    "Golden Wasteland",
    "Vault of Knowledge",
];

pub struct ShardEruption {
    data: [ShardEruptionData; 5],
}

impl ShardEruption {
    pub fn shard(&self) -> Option<ShardEruptionResponse> {
        let now = Utc::now()
            .with_timezone(&chrono_tz::America::Los_Angeles)
            .date_naive()
            .and_hms_opt(0, 0, 0)?;

        let day = now.day();
        let weekday = now.weekday().number_from_monday();
        let strong = day % 2 == 1;

        let info_index = if strong {
            ((day - 1) / 2) % 3 + 2
        } else {
            (day / 2) % 2
        };

        let shard_eruption = &self.data[info_index as usize];
        let no_shard_day = shard_eruption.no_shard_weekday.contains(&weekday);

        if no_shard_day {
            return None;
        }

        let realm_index = (day - 1) % 5;
        let area = &shard_eruption.area[realm_index as usize];

        // https://x.com/whirthun/status/1758229071216152597
        // if area.sky_map == SkyMap::JellyfishCove {
        //     return;
        // }

        let mut timestamps = Vec::new();
        let mut start_time = now + Duration::milliseconds(shard_eruption.offset.into());

        while timestamps.len() < 3 {
            let start = start_time + Duration::seconds(520);
            let end = start_time + Duration::hours(4);

            timestamps.push((
                start
                    .and_local_timezone(chrono_tz::America::Los_Angeles)
                    .unwrap(),
                end.and_local_timezone(chrono_tz::America::Los_Angeles)
                    .unwrap(),
            ));

            start_time += Duration::milliseconds((shard_eruption.interval * 3_600_000).into());
        }

        let realm = match SHARD_ERUPTION_REALM_NAMES.get(realm_index as usize) {
            Some(realm) => realm.to_string(),
            None => return None,
        };

        Some(ShardEruptionResponse {
            realm,
            sky_map: area.sky_map.clone(),
            strong,
            reward: area.reward,
            timestamps,
            url: area.url.clone(),
        })
    }
}

pub fn initialise_shard_eruption() -> ShardEruption {
    ShardEruption {
        data: [
            ShardEruptionData {
                no_shard_weekday: [6, 7],
                interval: 8,
                offset: 6600000,
                area: [
                    SkyMap::ButterflyFields,
                    SkyMap::ForestBrook,
                    SkyMap::IceRink,
                    SkyMap::BrokenTemple,
                    SkyMap::StarlightDesert,
                ]
                .into_iter()
                .map(|sky_map| {
                    let cloned_sky_map = sky_map.clone();

                    ShardEruptionMapData {
                        sky_map,
                        url: shard_eruption_map_url(cloned_sky_map),
                        reward: 200.0,
                    }
                })
                .collect(),
            },
            ShardEruptionData {
                no_shard_weekday: [7, 1],
                interval: 8,
                offset: 7800000,
                area: [
                    SkyMap::KoiPond,
                    SkyMap::Boneyard,
                    SkyMap::IceRink,
                    SkyMap::Battlefield,
                    SkyMap::StarlightDesert,
                ]
                .into_iter()
                .map(|sky_map| {
                    let cloned_sky_map = sky_map.clone();

                    ShardEruptionMapData {
                        sky_map,
                        url: shard_eruption_map_url(cloned_sky_map),
                        reward: 200.0,
                    }
                })
                .collect(),
            },
            ShardEruptionData {
                no_shard_weekday: [1, 2],
                interval: 6,
                offset: 27600000,
                area: [
                    ShardEruptionMapData {
                        sky_map: SkyMap::Cave,
                        url: shard_eruption_map_url(SkyMap::Cave),
                        reward: 2.0,
                    },
                    ShardEruptionMapData {
                        sky_map: SkyMap::ForestEnd,
                        url: shard_eruption_map_url(SkyMap::ForestEnd),
                        reward: 2.5,
                    },
                    ShardEruptionMapData {
                        sky_map: SkyMap::VillageOfDreams,
                        url: shard_eruption_map_url(SkyMap::VillageOfDreams),
                        reward: 2.5,
                    },
                    ShardEruptionMapData {
                        sky_map: SkyMap::Graveyard,
                        url: shard_eruption_map_url(SkyMap::Graveyard),
                        reward: 2.0,
                    },
                    ShardEruptionMapData {
                        sky_map: SkyMap::JellyfishCove,
                        url: shard_eruption_map_url(SkyMap::JellyfishCove),
                        reward: 3.5,
                    },
                ]
                .into(),
            },
            ShardEruptionData {
                no_shard_weekday: [2, 3],
                interval: 6,
                offset: 8400000,
                area: [
                    ShardEruptionMapData {
                        sky_map: SkyMap::BirdNest,
                        url: shard_eruption_map_url(SkyMap::BirdNest),
                        reward: 2.5,
                    },
                    ShardEruptionMapData {
                        sky_map: SkyMap::Treehouse,
                        url: shard_eruption_map_url(SkyMap::Treehouse),
                        reward: 3.5,
                    },
                    ShardEruptionMapData {
                        sky_map: SkyMap::VillageOfDreams,
                        url: shard_eruption_map_url(SkyMap::VillageOfDreams),
                        reward: 2.5,
                    },
                    ShardEruptionMapData {
                        sky_map: SkyMap::CrabFields,
                        url: shard_eruption_map_url(SkyMap::CrabFields),
                        reward: 2.5,
                    },
                    ShardEruptionMapData {
                        sky_map: SkyMap::JellyfishCove,
                        url: shard_eruption_map_url(SkyMap::JellyfishCove),
                        reward: 3.5,
                    },
                ]
                .into(),
            },
            ShardEruptionData {
                no_shard_weekday: [3, 4],
                interval: 6,
                offset: 12600000,
                area: [
                    SkyMap::SanctuaryIslands,
                    SkyMap::ElevatedClearing,
                    SkyMap::HermitValley,
                    SkyMap::ForgottenArk,
                    SkyMap::JellyfishCove,
                ]
                .into_iter()
                .map(|sky_map| {
                    let cloned_sky_map = sky_map.clone();

                    ShardEruptionMapData {
                        sky_map,
                        url: shard_eruption_map_url(cloned_sky_map),
                        reward: 3.5,
                    }
                })
                .collect(),
            },
        ],
    }
}
