mod structures;
mod utility;
use anyhow::{Context, Result};
use chrono::{Datelike, Timelike, Utc, Weekday};
use dashmap::DashMap;
use dotenvy::dotenv;
use serenity::{all::GuildId, http::Http};
use sqlx::postgres::{PgListener, PgPoolOptions};
use std::{env, sync::Arc, time::Duration};
use structures::{
    notification::{
        prepare_notification_to_send, Notification, NotificationEvent, NotificationListenerPacket,
        NotificationNotify, NotificationPacket,
    },
    shard_eruption,
};
use tokio::time::{interval, sleep};
use utility::constants::ISS_DATES_ACCESSIBLE;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    dotenv().ok();
    let discord_token = env::var("DISCORD_TOKEN").context("Error retrieving DISCORD_TOKEN")?;
    let database_url = env::var("DATABASE_URL").context("Error retrieving DATABASE_URL")?;
    let notification_cache: Arc<DashMap<GuildId, Notification>> = Arc::new(DashMap::new());

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await?;

    let notification_packets: Vec<NotificationPacket> =
        sqlx::query_as("select * from notifications;")
            .fetch_all(&pool)
            .await?;

    for notification_packet in notification_packets {
        let notification = Notification::from(notification_packet);
        notification_cache.insert(notification.guild_id, notification);
    }

    let mut listener = PgListener::connect_with(&pool).await?;
    listener.listen("notifications").await?;
    let notification_cache_clone = Arc::clone(&notification_cache);

    tokio::spawn(async move {
        if let Err(error) = listen(listener, notification_cache_clone).await {
            eprintln!("Error listening: {:?}", error);
        }
    });

    let client = Http::new(&discord_token);
    let notification_cache_clone = Arc::clone(&notification_cache);

    tokio::spawn(async move {
        if let Err(error) = notify(client, notification_cache_clone).await {
            eprintln!("Error in notifying: {:?}", error);
        }
    });

    tokio::signal::ctrl_c().await?;
    Ok(())
}

async fn listen(
    mut listener: PgListener,
    notification_cache: Arc<DashMap<GuildId, Notification>>,
) -> Result<()> {
    loop {
        match listener.recv().await {
            Ok(pg_notification) => {
                println!("Received: {:?}", pg_notification);
                match serde_json::from_str::<NotificationListenerPacket>(pg_notification.payload())
                {
                    Ok(notification_listener_packet) => {
                        let notification_listener = notification_listener_packet;

                        match notification_listener.operation.as_str() {
                            "INSERT" | "UPDATE" => {
                                let notification = Notification::from(notification_listener.data);
                                notification_cache.insert(notification.guild_id, notification);
                            }
                            "DELETE" => {
                                let notification = Notification::from(notification_listener.data);
                                notification_cache.remove(&notification.guild_id);
                            }
                            _ => {
                                eprintln!(
                                    "Unknown operation: {:?}",
                                    notification_listener.operation
                                );
                            }
                        }
                    }
                    Err(error) => {
                        eprintln!(
                            "Error deserialising notification listener packet: {:?}",
                            error
                        );
                    }
                }
            }
            Err(err) => {
                eprintln!("Error receiving notification: {:?}", err);
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

async fn notify(
    client: Http,
    notification_cache: Arc<DashMap<GuildId, Notification>>,
) -> Result<()> {
    let mut interval = interval(Duration::from_secs(1));
    let initialised_shard_eruption = shard_eruption::initialise_shard_eruption();
    let mut shard_eruption = initialised_shard_eruption.shard();

    loop {
        interval.tick().await;
        let now = Utc::now().with_timezone(&chrono_tz::America::Los_Angeles);
        let (day, hour, minute, second) = (now.day(), now.hour(), now.minute(), now.second());

        if second != 0 {
            continue;
        }

        let mut notification_notifies = vec![];

        if let Some(ref shard) = shard_eruption {
            // Find a start timestamp that is 5 minutes before the shard eruption.
            let timestamps = shard.timestamps.iter().find(|(start, _)| {
                let time = start.signed_duration_since(now);
                println!("Minutes: {:?} | {:?}", time, time.num_minutes());
                (0..=10).contains(&time.num_minutes())
            });

            if let Some((start_time, end_time)) = timestamps {
                let r#type = if shard.strong {
                    NotificationEvent::ShardEruptionStrong
                } else {
                    NotificationEvent::ShardEruptionRegular
                };

                notification_notifies.push(NotificationNotify {
                    r#type,
                    start_time: Some(start_time.timestamp()),
                    end_time: Some(end_time.timestamp()),
                    time_until_start: Some(
                        start_time
                            .signed_duration_since(now)
                            .num_minutes()
                            .try_into()
                            .expect("Failed to create time_until_start for a shard eruption."),
                    ),
                    shard_eruption: Some(shard.clone()),
                });
            }
        }

        if hour == 0 && minute == 0 {
            // Update the shard eruption.
            shard_eruption = initialised_shard_eruption.shard();

            notification_notifies.push(NotificationNotify {
                r#type: NotificationEvent::DailyReset,
                start_time: None,
                end_time: None,
                time_until_start: None,
                shard_eruption: None,
            });

            if now.weekday() == Weekday::Sun {
                notification_notifies.push(NotificationNotify {
                    r#type: NotificationEvent::EyeOfEden,
                    start_time: None,
                    end_time: None,
                    time_until_start: None,
                    shard_eruption: None,
                });
            }

            if ISS_DATES_ACCESSIBLE.contains(&day) {
                notification_notifies.push(NotificationNotify {
                    r#type: NotificationEvent::InternationalSpaceStation,
                    start_time: None,
                    end_time: None,
                    time_until_start: None,
                    shard_eruption: None,
                });
            }
        }

        if minute == 0
            || (10..=15).contains(&minute)
            || (25..=30).contains(&minute)
            || (40..=45).contains(&minute)
            || (55..=59).contains(&minute)
        {
            let time_until_start = match 15 - (minute % 15) {
                15 => 0,
                minute => minute,
            };

            let date = now + Duration::from_secs((time_until_start * 60).into());

            notification_notifies.push(NotificationNotify {
                r#type: NotificationEvent::Passage,
                start_time: Some(date.timestamp()),
                end_time: None,
                time_until_start: Some(time_until_start),
                shard_eruption: None,
            });
        }

        if ((((hour + 3) % 4) == 0) && ((45..=59).contains(&minute)))
            || ((((hour + 2) % 4) == 0) && (minute == 0))
        {
            let time_until_start = match 60 - minute {
                60 => 0,
                minute => minute,
            };

            let date = now + Duration::from_secs((time_until_start * 60).into());

            notification_notifies.push(NotificationNotify {
                r#type: NotificationEvent::Aurora,
                start_time: Some(date.timestamp()),
                end_time: None,
                time_until_start: Some(time_until_start),
                shard_eruption: None,
            });
        }

        if ((0..=5).contains(&minute) && (hour % 2) == 0)
            || ((55..=59).contains(&minute) && (hour % 2) == 1)
        {
            let time_until_start = match hour % 2 {
                0 => 5 - minute,
                1 => 65 - minute,
                _ => unreachable!(),
            };

            let date = now + Duration::from_secs((time_until_start * 60).into());

            notification_notifies.push(NotificationNotify {
                r#type: NotificationEvent::PollutedGeyser,
                start_time: Some(date.timestamp()),
                end_time: None,
                time_until_start: Some(time_until_start),
                shard_eruption: None,
            });
        }

        if ((hour % 2) == 0) && ((25..=35).contains(&minute)) {
            let time_until_start = 35 - minute;
            let date = now + Duration::from_secs((time_until_start * 60).into());

            notification_notifies.push(NotificationNotify {
                r#type: NotificationEvent::Grandma,
                start_time: Some(date.timestamp()),
                end_time: None,
                time_until_start: Some(time_until_start),
                shard_eruption: None,
            });
        }

        if ((hour % 2) == 0) && ((40..=50).contains(&minute)) {
            let time_until_start = 50 - minute;
            let date = now + Duration::from_secs((time_until_start * 60).into());

            notification_notifies.push(NotificationNotify {
                r#type: NotificationEvent::Turtle,
                start_time: Some(date.timestamp()),
                end_time: None,
                time_until_start: Some(time_until_start),
                shard_eruption: None,
            });
        }

        if day == 1 && (hour % 4) == 0 {
            notification_notifies.push(NotificationNotify {
                r#type: NotificationEvent::AviarysFireworkFestival,
                start_time: None,
                end_time: None,
                time_until_start: None,
                shard_eruption: None,
            });
        }

        // if minute == 55 {
        //     notification_notifies.push(NotificationNotify {
        //         r#type: NotificationEvent::Dragon,
        //         start_time: None,
        //         end_time: None,
        //         time_until_start: None,
        //         shard_eruption: None,
        //     });
        // }

        for notification_notify in notification_notifies {
            println!(
                "{}:{}:{} | {}",
                hour, minute, second, notification_notify.r#type
            );

            prepare_notification_to_send(
                &client,
                Arc::clone(&notification_cache),
                notification_notify,
            )
            .await;
        }
    }
}
