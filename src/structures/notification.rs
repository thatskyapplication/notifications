use super::shard_eruption::ShardEruptionResponse;
use anyhow::{anyhow, Result};
use futures::{future::join_all, FutureExt};
use serde::{Deserialize, Serialize};
use serenity::{
    all::{CreateAllowedMentions, CreateMessage, MessageFlags, Nonce},
    http::Http,
    model::id::{ChannelId, GuildId, RoleId},
};
use sqlx::{prelude::FromRow, Pool, Postgres};
use std::{fmt, str::FromStr, sync::Arc};

#[derive(Clone, Deserialize, FromRow, Serialize)]
pub struct NotificationPacket {
    guild_id: String,
    polluted_geyser_channel_id: Option<String>,
    polluted_geyser_role_id: Option<String>,
    polluted_geyser_sendable: bool,
    polluted_geyser_offset: i16,
    grandma_channel_id: Option<String>,
    grandma_role_id: Option<String>,
    grandma_sendable: bool,
    grandma_offset: i16,
    turtle_channel_id: Option<String>,
    turtle_role_id: Option<String>,
    turtle_sendable: bool,
    turtle_offset: i16,
    eye_of_eden_channel_id: Option<String>,
    eye_of_eden_role_id: Option<String>,
    eye_of_eden_sendable: bool,
    daily_reset_channel_id: Option<String>,
    daily_reset_role_id: Option<String>,
    daily_reset_sendable: bool,
    passage_channel_id: Option<String>,
    passage_role_id: Option<String>,
    passage_sendable: bool,
    passage_offset: i16,
    aurora_channel_id: Option<String>,
    aurora_role_id: Option<String>,
    aurora_sendable: bool,
    aurora_offset: i16,
    regular_shard_eruption_channel_id: Option<String>,
    regular_shard_eruption_role_id: Option<String>,
    regular_shard_eruption_sendable: bool,
    regular_shard_eruption_offset: i16,
    strong_shard_eruption_channel_id: Option<String>,
    strong_shard_eruption_role_id: Option<String>,
    strong_shard_eruption_sendable: bool,
    strong_shard_eruption_offset: i16,
    iss_channel_id: Option<String>,
    iss_role_id: Option<String>,
    iss_sendable: bool,
    aviarys_firework_festival_channel_id: Option<String>,
    aviarys_firework_festival_role_id: Option<String>,
    aviarys_firework_festival_sendable: bool,
    dragon_channel_id: Option<String>,
    dragon_role_id: Option<String>,
    dragon_sendable: bool,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum NotificationEvent {
    PollutedGeyser,
    Grandma,
    Turtle,
    DailyReset,
    EyeOfEden,
    InternationalSpaceStation,
    ShardEruptionRegular,
    ShardEruptionStrong,
    Aurora,
    Passage,
    AviarysFireworkFestival,
    Dragon,
}

impl fmt::Display for NotificationEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NotificationEvent::PollutedGeyser => write!(f, "0"),
            NotificationEvent::Grandma => write!(f, "1"),
            NotificationEvent::Turtle => write!(f, "2"),
            NotificationEvent::DailyReset => write!(f, "3"),
            NotificationEvent::EyeOfEden => write!(f, "4"),
            NotificationEvent::InternationalSpaceStation => write!(f, "5"),
            NotificationEvent::ShardEruptionRegular => write!(f, "6"),
            NotificationEvent::ShardEruptionStrong => write!(f, "7"),
            NotificationEvent::Aurora => write!(f, "8"),
            NotificationEvent::Passage => write!(f, "9"),
            NotificationEvent::AviarysFireworkFestival => write!(f, "10"),
            NotificationEvent::Dragon => write!(f, "11"),
        }
    }
}

pub struct NotificationNotify {
    pub r#type: NotificationEvent,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub time_until_start: Option<u32>,
    pub shard_eruption: Option<ShardEruptionResponse>,
}

#[derive(Debug)]
pub struct Notification {
    pub guild_id: GuildId,
    pub polluted_geyser_channel_id: Option<ChannelId>,
    pub polluted_geyser_role_id: Option<RoleId>,
    pub polluted_geyser_sendable: bool,
    pub polluted_geyser_offset: u32,
    pub grandma_channel_id: Option<ChannelId>,
    pub grandma_role_id: Option<RoleId>,
    pub grandma_sendable: bool,
    pub grandma_offset: u32,
    pub turtle_channel_id: Option<ChannelId>,
    pub turtle_role_id: Option<RoleId>,
    pub turtle_sendable: bool,
    pub turtle_offset: u32,
    pub eye_of_eden_channel_id: Option<ChannelId>,
    pub eye_of_eden_role_id: Option<RoleId>,
    pub eye_of_eden_sendable: bool,
    pub daily_reset_channel_id: Option<ChannelId>,
    pub daily_reset_role_id: Option<RoleId>,
    pub daily_reset_sendable: bool,
    pub passage_channel_id: Option<ChannelId>,
    pub passage_role_id: Option<RoleId>,
    pub passage_sendable: bool,
    pub passage_offset: u32,
    pub aurora_channel_id: Option<ChannelId>,
    pub aurora_role_id: Option<RoleId>,
    pub aurora_sendable: bool,
    pub aurora_offset: u32,
    pub regular_shard_eruption_channel_id: Option<ChannelId>,
    pub regular_shard_eruption_role_id: Option<RoleId>,
    pub regular_shard_eruption_sendable: bool,
    pub regular_shard_eruption_offset: u32,
    pub strong_shard_eruption_channel_id: Option<ChannelId>,
    pub strong_shard_eruption_role_id: Option<RoleId>,
    pub strong_shard_eruption_sendable: bool,
    pub strong_shard_eruption_offset: u32,
    pub iss_channel_id: Option<ChannelId>,
    pub iss_role_id: Option<RoleId>,
    pub iss_sendable: bool,
    pub aviarys_firework_festival_channel_id: Option<ChannelId>,
    pub aviarys_firework_festival_role_id: Option<RoleId>,
    pub aviarys_firework_festival_sendable: bool,
    pub dragon_channel_id: Option<ChannelId>,
    pub dragon_role_id: Option<RoleId>,
    pub dragon_sendable: bool,
}

fn convert_string_to_id<T: FromStr>(id: Option<String>) -> Option<T> {
    id.and_then(|id| T::from_str(&id).ok())
}

fn convert_i16_to_u32(i: i16) -> u32 {
    i.try_into().expect("Conversion from i16 to u32 failed.")
}

impl From<NotificationPacket> for Notification {
    fn from(packet: NotificationPacket) -> Self {
        Self {
            guild_id: GuildId::from_str(&packet.guild_id).expect("Invalid guild id."),
            polluted_geyser_channel_id: convert_string_to_id(packet.polluted_geyser_channel_id),
            polluted_geyser_role_id: convert_string_to_id(packet.polluted_geyser_role_id),
            polluted_geyser_sendable: packet.polluted_geyser_sendable,
            polluted_geyser_offset: convert_i16_to_u32(packet.polluted_geyser_offset),
            grandma_channel_id: convert_string_to_id(packet.grandma_channel_id),
            grandma_role_id: convert_string_to_id(packet.grandma_role_id),
            grandma_sendable: packet.grandma_sendable,
            grandma_offset: convert_i16_to_u32(packet.grandma_offset),
            turtle_channel_id: convert_string_to_id(packet.turtle_channel_id),
            turtle_role_id: convert_string_to_id(packet.turtle_role_id),
            turtle_sendable: packet.turtle_sendable,
            turtle_offset: convert_i16_to_u32(packet.turtle_offset),
            eye_of_eden_channel_id: convert_string_to_id(packet.eye_of_eden_channel_id),
            eye_of_eden_role_id: convert_string_to_id(packet.eye_of_eden_role_id),
            eye_of_eden_sendable: packet.eye_of_eden_sendable,
            daily_reset_channel_id: convert_string_to_id(packet.daily_reset_channel_id),
            daily_reset_role_id: convert_string_to_id(packet.daily_reset_role_id),
            daily_reset_sendable: packet.daily_reset_sendable,
            passage_channel_id: convert_string_to_id(packet.passage_channel_id),
            passage_role_id: convert_string_to_id(packet.passage_role_id),
            passage_sendable: packet.passage_sendable,
            passage_offset: convert_i16_to_u32(packet.passage_offset),
            aurora_channel_id: convert_string_to_id(packet.aurora_channel_id),
            aurora_role_id: convert_string_to_id(packet.aurora_role_id),
            aurora_sendable: packet.aurora_sendable,
            aurora_offset: convert_i16_to_u32(packet.aurora_offset),
            regular_shard_eruption_channel_id: convert_string_to_id(
                packet.regular_shard_eruption_channel_id,
            ),
            regular_shard_eruption_role_id: convert_string_to_id(
                packet.regular_shard_eruption_role_id,
            ),
            regular_shard_eruption_sendable: packet.regular_shard_eruption_sendable,
            regular_shard_eruption_offset: convert_i16_to_u32(packet.regular_shard_eruption_offset),
            strong_shard_eruption_channel_id: convert_string_to_id(
                packet.strong_shard_eruption_channel_id,
            ),
            strong_shard_eruption_role_id: convert_string_to_id(
                packet.strong_shard_eruption_role_id,
            ),
            strong_shard_eruption_sendable: packet.strong_shard_eruption_sendable,
            strong_shard_eruption_offset: convert_i16_to_u32(packet.strong_shard_eruption_offset),
            iss_channel_id: convert_string_to_id(packet.iss_channel_id),
            iss_role_id: convert_string_to_id(packet.iss_role_id),
            iss_sendable: packet.iss_sendable,
            aviarys_firework_festival_channel_id: convert_string_to_id(
                packet.aviarys_firework_festival_channel_id,
            ),
            aviarys_firework_festival_role_id: convert_string_to_id(
                packet.aviarys_firework_festival_role_id,
            ),
            aviarys_firework_festival_sendable: packet.aviarys_firework_festival_sendable,
            dragon_channel_id: convert_string_to_id(packet.dragon_channel_id),
            dragon_role_id: convert_string_to_id(packet.dragon_role_id),
            dragon_sendable: packet.dragon_sendable,
        }
    }
}

impl Notification {
    pub async fn send(
        &self,
        client: &Http,
        notification_notify: Arc<NotificationNotify>,
    ) -> Result<()> {
        let r#type = &notification_notify.r#type;

        let (channel_id, role_id, suffix) = match r#type {
            NotificationEvent::PollutedGeyser => (
                self.polluted_geyser_channel_id,
                self.polluted_geyser_role_id,
                match notification_notify.time_until_start {
                    Some(0) => "The Polluted Geyser is starting to erupt!".to_string(),
                    None => panic!("Polluted Geyser notifications should have a time until start."),
                    _ => format!(
                        "The Polluted Geyser will erupt <t:{}:R>!",
                        notification_notify.start_time.expect(
                            "A start time for the polluted geyser notification should be set."
                        )
                    ),
                },
            ),
            NotificationEvent::Grandma => (
                self.grandma_channel_id,
                self.grandma_role_id,
                match notification_notify.time_until_start {
                    Some(0) => "Grandma has begun sharing her light!".to_string(),
                    None => panic!("Grandma notifications should have a time until start."),
                    _ => format!(
                        "Grandma will share her light <t:{}:R>!",
                        notification_notify
                            .start_time
                            .expect("A start time for the grandma notification should be set.")
                    ),
                },
            ),
            NotificationEvent::Turtle => (
                self.turtle_channel_id,
                self.turtle_role_id,
                match notification_notify.time_until_start {
                    Some(0) => "The turtle needs cleansing of darkness now!".to_string(),
                    None => panic!("Turtle notifications should have a time until start."),
                    _ => format!(
                        "The turtle will need cleansing of darkness <t:{}:R>!",
                        notification_notify
                            .start_time
                            .expect("A start time for the turtle notification should be set.")
                    ),
                },
            ),
            NotificationEvent::DailyReset => (
                self.daily_reset_channel_id,
                self.daily_reset_role_id,
                "It's a new day. Time to forge candles again!".to_string(),
            ),
            NotificationEvent::EyeOfEden => (
                self.eye_of_eden_channel_id,
                self.eye_of_eden_role_id,
                "Skykids may save statues in the Eye of Eden again!".to_string(),
            ),
            NotificationEvent::InternationalSpaceStation => (
                self.iss_channel_id,
                self.iss_role_id,
                "The International Space Station is accessible!".to_string(),
            ),
            NotificationEvent::ShardEruptionRegular => {
                let shard_eruption = notification_notify
                    .shard_eruption
                    .clone()
                    .expect("A shard eruption must have data.");

                let end_time = notification_notify
                    .end_time
                    .expect("A shard eruption must have an end time.");

                (
                    self.regular_shard_eruption_channel_id,
                    self.regular_shard_eruption_role_id,
                    match notification_notify.time_until_start {
                        Some(0) => format!(
                            "A regular shard eruption is landing in the [{} ({})]({}) and clears up <t:{}:R>!",
                            shard_eruption.realm,
                            shard_eruption.sky_map,
                            shard_eruption.url,
                            end_time
                        ),
                        None => panic!("Shard eruption notifications should have a time until start."),
                        _ => format!(
                            "A regular shard eruption lands in the [{} ({})]({}) <t:{}:R> and clears up <t:{}:R>!",
                            shard_eruption.realm,
                            shard_eruption.sky_map,
                            shard_eruption.url,
                            notification_notify.start_time.expect("A start time for the shard eruption notification should be set."),
                            end_time
                        ),
                    }
                )
            }
            NotificationEvent::ShardEruptionStrong => {
                let shard_eruption = notification_notify
                    .shard_eruption
                    .clone()
                    .expect("A shard eruption must have data.");

                let end_time = notification_notify
                    .end_time
                    .expect("A shard eruption must have an end time.");

                (
                    self.strong_shard_eruption_channel_id,
                    self.strong_shard_eruption_role_id,
                    match notification_notify.time_until_start {
                        Some(0) => format!(
                            "A strong shard eruption is landing in the [{} ({})]({}) and clears up <t:{}:R>!",
                            shard_eruption.realm,
                            shard_eruption.sky_map,
                            shard_eruption.url,
                            end_time
                        ),
                        None => panic!("Shard eruption notifications should have a time until start."),
                         _ => format!(
                            "A strong shard eruption lands in the [{} ({})]({}) <t:{}:R> and clears up <t:{}:R>!",
                            shard_eruption.realm,
                            shard_eruption.sky_map,
                            shard_eruption.url,
                            notification_notify.start_time.expect("A start time for the shard eruption notification should be set."),
                            end_time
                        )
                    }
                )
            }
            NotificationEvent::Aurora => (
                self.aurora_channel_id,
                self.aurora_role_id,
                match notification_notify.time_until_start {
                    Some(0) => "The AURORA concert is starting! Take your friends!".to_string(),
                    None => panic!("AURORA notifications should have a time until start."),
                    _ => format!(
                        "The AURORA concert will start <t:{}:R>! Take your friends!",
                        notification_notify
                            .start_time
                            .expect("A start time for the AURORA notification should be set.")
                    ),
                },
            ),
            NotificationEvent::Passage => (
                self.passage_channel_id,
                self.passage_role_id,
                match notification_notify.time_until_start {
                    Some(0) => "The Season of Passage quests are starting!".to_string(),
                    None => {
                        panic!("Season of Passage notifications should have a time until start.")
                    }
                    _ => format!(
                        "The Season of Passage quests will start <t:{}:R>!",
                        notification_notify.start_time.expect(
                            "A start time for the Season of Passage notification should be set."
                        )
                    ),
                },
            ),
            NotificationEvent::AviarysFireworkFestival => (
                self.aviarys_firework_festival_channel_id,
                self.aviarys_firework_festival_role_id,
                "Aviary's Firework Festival is beginning!".to_string(),
            ),
            NotificationEvent::Dragon => (
                self.dragon_channel_id,
                self.dragon_role_id,
                format!(
                    "The dragon will appear <t:{}:R>!",
                    notification_notify
                        .start_time
                        .expect("A start time for the dragon notification should be set.")
                ),
            ),
        };

        let (channel_id, role_id) = match (channel_id, role_id) {
            (Some(channel_id), Some(role_id)) => (channel_id, role_id),
            _ => return Ok(()),
        };

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

fn is_sendable(
    notification: &Notification,
    event: &NotificationEvent,
    time_until_start: Option<u32>,
) -> bool {
    match event {
        NotificationEvent::PollutedGeyser => {
            notification.polluted_geyser_offset
                == time_until_start
                    .expect("Polluted Geyser notifications should have a time until start.")
        }
        NotificationEvent::Grandma => {
            notification.grandma_offset
                == time_until_start.expect("Grandma notifications should have a time until start.")
        }
        NotificationEvent::Turtle => {
            notification.turtle_offset
                == time_until_start.expect("Turtle notifications should have a time until start.")
        }
        NotificationEvent::DailyReset => true,
        NotificationEvent::EyeOfEden => true,
        NotificationEvent::InternationalSpaceStation => true,
        NotificationEvent::ShardEruptionRegular => {
            notification.regular_shard_eruption_offset
                == time_until_start
                    .expect("Shard eruption notifications should have a time until start.")
        }
        NotificationEvent::ShardEruptionStrong => {
            notification.strong_shard_eruption_offset
                == time_until_start
                    .expect("Shard eruption notifications should have a time until start.")
        }
        NotificationEvent::Aurora => {
            notification.aurora_offset
                == time_until_start.expect("Aurora notifications should have a time until start.")
        }
        NotificationEvent::Passage => {
            notification.passage_offset
                == time_until_start
                    .expect("Season of Passage notifications should have a time until start.")
        }
        NotificationEvent::AviarysFireworkFestival => true,
        NotificationEvent::Dragon => true,
    }
}

pub async fn prepare_notification_to_send(
    client: &Http,
    pool: &Pool<Postgres>,
    notification_notify: NotificationNotify,
) {
    let notification_notify = Arc::new(notification_notify);

    let query_string = match notification_notify.r#type {
        NotificationEvent::PollutedGeyser => "select * from notifications where polluted_geyser_channel_id is not null and polluted_geyser_role_id is not null and polluted_geyser_sendable is true;",
        NotificationEvent::Grandma => "select * from notifications where grandma_channel_id is not null and grandma_role_id is not null and grandma_sendable is true;",
        NotificationEvent::Turtle => "select * from notifications where turtle_channel_id is not null and turtle_role_id is not null and turtle_sendable is true;",
        NotificationEvent::DailyReset => "select * from notifications where daily_reset_channel_id is not null and daily_reset_role_id is not null and daily_reset_sendable is true;",
        NotificationEvent::EyeOfEden => "select * from notifications where eye_of_eden_channel_id is not null and eye_of_eden_role_id is not null and eye_of_eden_sendable is true;",
        NotificationEvent::InternationalSpaceStation => "select * from notifications where iss_channel_id is not null and iss_role_id is not null and iss_sendable is true;",
        NotificationEvent::ShardEruptionRegular => "select * from notifications where regular_shard_eruption_channel_id is not null and regular_shard_eruption_role_id is not null and regular_shard_eruption_sendable is true;",
        NotificationEvent::ShardEruptionStrong => "select * from notifications where strong_shard_eruption_channel_id is not null and strong_shard_eruption_role_id is not null and strong_shard_eruption_sendable is true;",
        NotificationEvent::Aurora => "select * from notifications where aurora_channel_id is not null and aurora_role_id is not null and aurora_sendable is true;",
        NotificationEvent::Passage => "select * from notifications where passage_channel_id is not null and passage_role_id is not null and passage_sendable is true;",
        NotificationEvent::AviarysFireworkFestival => "select * from notifications where aviarys_firework_festival_channel_id is not null and aviarys_firework_festival_role_id is not null and aviarys_firework_festival_sendable is true;",
        NotificationEvent::Dragon => "select * from notifications where dragon_channel_id is not null and dragon_role_id is not null and dragon_sendable is true;",
    };

    let results: Vec<NotificationPacket> = sqlx::query_as(query_string)
        .fetch_all(pool)
        .await
        .expect("what");

    let futures = results
        .iter()
        .filter_map(|notification_packet| {
            let notification = Notification::from(notification_packet.clone());

            if is_sendable(
                &notification,
                &notification_notify.r#type,
                notification_notify.time_until_start,
            ) {
                Some(
                    {
                        let notification_notify_clone = notification_notify.clone();
                        async move {
                            notification
                                .send(client, Arc::clone(&notification_notify_clone))
                                .await
                        }
                    }
                    .boxed(),
                )
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let results = join_all(futures).await;

    for result in results {
        if let Err(error) = result {
            eprintln!("Failed to send notification: {:?}", error);
        }
    }
}
