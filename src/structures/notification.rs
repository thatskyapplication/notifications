use crate::utility::wind_paths::ShardEruptionResponse;
use anyhow::{anyhow, Result};
use futures::{future::join_all, FutureExt};
use serde::{Deserialize, Serialize};
use serenity::{
    all::{CreateAllowedMentions, CreateMessage, MessageFlags, Nonce},
    http::Http,
    model::id::{ChannelId, GuildId, RoleId},
};
use sqlx::{prelude::FromRow, Pool, Postgres};
use std::{fmt, str::FromStr};

#[derive(Clone, Deserialize, FromRow, Serialize)]
pub struct NotificationPacket {
    guild_id: String,
    r#type: i16,
    channel_id: String,
    role_id: String,
    offset: i16,
    sendable: bool,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NotificationType {
    DailyReset,
    EyeOfEden,
    InternationalSpaceStation,
    Dragon,
    PollutedGeyser,
    Grandma,
    Turtle,
    ShardEruptionRegular,
    ShardEruptionStrong,
    Aurora,
    Passage,
    AviarysFireworkFestival,
    TravellingSpirit,
}

impl fmt::Display for NotificationType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NotificationType::DailyReset => write!(f, "0"),
            NotificationType::EyeOfEden => write!(f, "1"),
            NotificationType::InternationalSpaceStation => write!(f, "2"),
            NotificationType::Dragon => write!(f, "3"),
            NotificationType::PollutedGeyser => write!(f, "4"),
            NotificationType::Grandma => write!(f, "5"),
            NotificationType::Turtle => write!(f, "6"),
            NotificationType::ShardEruptionRegular => write!(f, "7"),
            NotificationType::ShardEruptionStrong => write!(f, "8"),
            NotificationType::Aurora => write!(f, "9"),
            NotificationType::Passage => write!(f, "10"),
            NotificationType::AviarysFireworkFestival => write!(f, "11"),
            NotificationType::TravellingSpirit => write!(f, "12"),
        }
    }
}

pub struct NotificationNotify {
    pub r#type: NotificationType,
    pub start_time: i64,
    pub end_time: Option<i64>,
    pub time_until_start: u32,
    pub shard_eruption: Option<ShardEruptionResponse>,
    pub travelling_spirit_name: Option<String>,
}

#[derive(Debug)]
pub struct Notification {
    guild_id: GuildId,
    r#type: i16,
    pub channel_id: ChannelId,
    pub role_id: RoleId,
    offset: i16,
    sendable: bool,
}

impl From<NotificationPacket> for Notification {
    fn from(packet: NotificationPacket) -> Self {
        Self {
            guild_id: GuildId::from_str(&packet.guild_id).expect("Invalid guild id."),
            r#type: packet.r#type,
            channel_id: ChannelId::from_str(&packet.channel_id).expect("Invalid channel id."),
            role_id: RoleId::from_str(&packet.role_id).expect("Invalid role id."),
            offset: packet.offset,
            sendable: packet.sendable,
        }
    }
}

impl Notification {
    pub async fn send(
        &self,
        client: &Http,
        notification_notify: &NotificationNotify,
    ) -> Result<()> {
        let r#type = &notification_notify.r#type;

        let suffix = match r#type {
            NotificationType::DailyReset => {
                if notification_notify.time_until_start == 0 {
                    "It's a new day. Time to forge candles again!".to_string()
                } else {
                    format!(
                        "A new day will begin in <t:{}:R>!",
                        notification_notify.start_time
                    )
                }
            }
            NotificationType::EyeOfEden => {
                if notification_notify.time_until_start == 0 {
                    "Sky kids may save statues in the Eye of Eden again!".to_string()
                } else {
                    format!(
                        "Statues in the Eye of Eden will reset <t:{}:R>!",
                        notification_notify.start_time
                    )
                }
            }
            NotificationType::InternationalSpaceStation => {
                if notification_notify.time_until_start == 0 {
                    "The International Space Station is accessible!".to_string()
                } else {
                    format!(
                        "The International Space Station will be accessible <t:{}:R>!",
                        notification_notify.start_time
                    )
                }
            }
            NotificationType::Dragon => {
                if notification_notify.time_until_start == 0 {
                    "The dragon is appearing now!".to_string()
                } else {
                    format!(
                        "The dragon will appear <t:{}:R>!",
                        notification_notify.start_time
                    )
                }
            }
            NotificationType::PollutedGeyser => {
                if notification_notify.time_until_start == 0 {
                    "The Polluted Geyser is starting to erupt!".to_string()
                } else {
                    format!(
                        "The Polluted Geyser will erupt <t:{}:R>!",
                        notification_notify.start_time
                    )
                }
            }
            NotificationType::Grandma => {
                if notification_notify.time_until_start == 0 {
                    "Grandma has begun sharing her light!".to_string()
                } else {
                    format!(
                        "Grandma will share her light <t:{}:R>!",
                        notification_notify.start_time
                    )
                }
            }
            NotificationType::Turtle => {
                if notification_notify.time_until_start == 0 {
                    "The turtle needs cleansing of darkness now!".to_string()
                } else {
                    format!(
                        "The turtle will need cleansing of darkness <t:{}:R>!",
                        notification_notify.start_time
                    )
                }
            }
            NotificationType::ShardEruptionRegular => {
                let shard_eruption = notification_notify
                    .shard_eruption
                    .as_ref()
                    .expect("A shard eruption must have data.");

                let end_time = notification_notify
                    .end_time
                    .expect("A shard eruption must have an end time.");

                if notification_notify.time_until_start == 0 {
                    format!(
                        "A regular shard eruption is landing in the [{} ({})]({}) and clears up <t:{}:R>!",
                        shard_eruption.realm,
                        shard_eruption.sky_map,
                        shard_eruption.url,
                        end_time
                    )
                } else {
                    format!(
                        "A regular shard eruption lands in the [{} ({})]({}) <t:{}:R> and clears up <t:{}:R>!",
                        shard_eruption.realm,
                        shard_eruption.sky_map,
                        shard_eruption.url,
                        notification_notify.start_time,
                        end_time
                    )
                }
            }
            NotificationType::ShardEruptionStrong => {
                let shard_eruption = notification_notify
                    .shard_eruption
                    .as_ref()
                    .expect("A shard eruption must have data.");

                let end_time = notification_notify
                    .end_time
                    .expect("A shard eruption must have an end time.");

                if notification_notify.time_until_start == 0 {
                    format!(
                        "A strong shard eruption is landing in the [{} ({})]({}) and clears up <t:{}:R>!",
                        shard_eruption.realm,
                        shard_eruption.sky_map,
                        shard_eruption.url,
                        end_time
                    )
                } else {
                    format!(
						"A strong shard eruption lands in the [{} ({})]({}) <t:{}:R> and clears up <t:{}:R>!",
						shard_eruption.realm,
						shard_eruption.sky_map,
						shard_eruption.url,
						notification_notify.start_time,
						end_time
					)
                }
            }
            NotificationType::Aurora => {
                if notification_notify.time_until_start == 0 {
                    "The AURORA concert is starting! Take your friends!".to_string()
                } else {
                    format!(
                        "The AURORA concert will start <t:{}:R>! Take your friends!",
                        notification_notify.start_time
                    )
                }
            }
            NotificationType::Passage => {
                if notification_notify.time_until_start == 0 {
                    "The Season of Passage quests are starting!".to_string()
                } else {
                    format!(
                        "The Season of Passage quests will start <t:{}:R>!",
                        notification_notify.start_time
                    )
                }
            }
            NotificationType::AviarysFireworkFestival => {
                if notification_notify.time_until_start == 0 {
                    "Aviary's Firework Festival is beginning!".to_string()
                } else {
                    format!(
                        "Aviary's Firework Festival will begin <t:{}:R>!",
                        notification_notify.start_time
                    )
                }
            }
            NotificationType::TravellingSpirit => {
                if notification_notify.time_until_start == 0 {
                    format!(
                        "{} has arrived!",
                        notification_notify
                            .travelling_spirit_name
                            .as_ref()
                            .expect("A travelling spirit must have a name.")
                    )
                } else {
                    format!(
                        "{} will arrive <t:{}:R>!",
                        notification_notify
                            .travelling_spirit_name
                            .as_ref()
                            .expect("A travelling spirit must have a name."),
                        notification_notify.start_time
                    )
                }
            }
        };

        let channel_id = self.channel_id;
        let role_id = self.role_id;

        client
            .send_message(
                channel_id,
                vec![],
                &CreateMessage::new()
                    .allowed_mentions(CreateAllowedMentions::new().roles(vec![role_id]))
                    .content(format!("<@&{}> {}", role_id, suffix))
                    .enforce_nonce(true)
                    .flags(MessageFlags::SUPPRESS_EMBEDS)
                    .nonce(Nonce::String(format!("{}-{}", r#type, channel_id,))),
            )
            .await
            .map_err(|error| anyhow!(error))?;

        Ok(())
    }
}

pub async fn prepare_notification_to_send(
    client: &Http,
    pool: &Pool<Postgres>,
    notification_notify: &NotificationNotify,
) {
    let results: Vec<NotificationPacket> = sqlx::query_as(
        r#"select * from notifications where type = $1 and "offset" = $2 and sendable is true;"#,
    )
    .bind(notification_notify.r#type as i16)
    .bind(notification_notify.time_until_start as i16)
    .fetch_all(pool)
    .await
    .expect("Failed to retrieve notification packets.");

    let futures = results
        .iter()
        .map(|notification_packet| {
            let notification = Notification::from(notification_packet.clone());
            { async move { notification.send(client, notification_notify).await } }.boxed()
        })
        .collect::<Vec<_>>();

    let results = join_all(futures).await;

    for result in results {
        if let Err(error) = result {
            tracing::error!("Failed to send notification: {error:?}");
        }
    }
}
