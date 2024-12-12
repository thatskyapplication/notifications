mod structures;
mod utility;
use anyhow::{Context, Result};
use chrono::{Datelike, Timelike, Utc, Weekday};
use core::panic;
use dotenvy::dotenv;
use futures::FutureExt;
use serenity::http::Http;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{env, time::Duration};
use structures::{
    notification::{prepare_notification_to_send, NotificationNotify, NotificationType},
    travelling_spirit::get_last_travelling_spirit,
};
use tokio::{sync::mpsc, time::sleep};
use utility::{
    constants::{
        INTERNATIONAL_SPACE_STATION_DATES, INTERNATIONAL_SPACE_STATION_PRIOR_DATES,
        MAXIMUM_CHANNEL_CAPACITY,
    },
    functions::last_day_of_month,
    wind_paths::shard_eruption,
};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let environment = env::var("RUST_ENV").unwrap_or("development".to_string());
    let discord_token = env::var("DISCORD_TOKEN").context("Error retrieving DISCORD_TOKEN.")?;
    let database_url = env::var("DATABASE_URL").context("Error retrieving DATABASE_URL.")?;

    let wind_paths_url = if environment == "production" {
        env::var("WIND_PATHS_URL").context("Error retrieving WIND_PATHS_URL.")?
    } else {
        env::var("DEVELOPMENT_WIND_PATHS_URL")
            .context("Error retrieving DEVELOPMENT_WIND_PATHS_URL.")?
    };

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await?;

    let travelling_spirit_pool = pool.clone();
    let client = Http::new(&discord_token);
    let (tx, mut rx) = mpsc::channel::<NotificationNotify>(MAXIMUM_CHANNEL_CAPACITY);

    tokio::spawn(async move {
        loop {
            let tx_clone = tx.clone();
            let travelling_spirit_pool_clone = travelling_spirit_pool.clone();
            let wind_paths_url_clone = wind_paths_url.clone();

            let result = panic::AssertUnwindSafe(async move {
                if let Err(error) =
                    notify(tx_clone, travelling_spirit_pool_clone, wind_paths_url_clone).await
                {
                    tracing::error!("Error in notifying: {error:?}");
                }
            })
            .catch_unwind()
            .await;

            if let Err(error) = result {
                tracing::error!("Panic in notify function: {error:?}");
            }
        }
    });

    tokio::spawn(async move {
        while let Some(notification_notify) = rx.recv().await {
            prepare_notification_to_send(&client, &pool, &notification_notify).await;
            let queued = rx.len();

            if queued == MAXIMUM_CHANNEL_CAPACITY {
                tracing::info!(
                    "There are {} notifications queued in the channel. This might be a bottleneck. Most recent notification type sent: {}",
                    queued,
                    notification_notify.r#type
                );
            }
        }
    });

    tokio::signal::ctrl_c().await?;
    Ok(())
}

async fn notify(
    tx: mpsc::Sender<NotificationNotify>,
    pool: Pool<Postgres>,
    wind_paths_url: String,
) -> Result<()> {
    let mut shard_data = shard_eruption(&wind_paths_url).await;
    let mut travelling_spirit = get_last_travelling_spirit(&pool).await;
    let mut travelling_spirit_start = travelling_spirit.start;

    let mut travelling_spirit_earliest_notification_time =
        travelling_spirit_start - Duration::from_secs(900);

    loop {
        sleep(Duration::from_millis(
            60000 - (Utc::now().timestamp_millis() % 60000) as u64,
        ))
        .await;

        let now = Utc::now()
            .with_timezone(&chrono_tz::America::Los_Angeles)
            .with_nanosecond(0)
            .unwrap();

        let (day, hour, minute) = (now.day(), now.hour(), now.minute());
        let last_day_of_month = last_day_of_month(now);
        let mut notification_notifies = vec![];

        if hour == 0 && minute == 0 {
            // Update the shard eruption.
            shard_data = shard_eruption(&wind_paths_url).await;

            // Update the travelling spirit.
            // It may seem unusual to do this every day, but it is not future-proof to check every 2 weeks only.
            // For example, Saluting Protector at 09/12/2024 was out of the usual 2-week rotation.
            travelling_spirit = get_last_travelling_spirit(&pool).await;
            travelling_spirit_start = travelling_spirit.start;

            travelling_spirit_earliest_notification_time =
                travelling_spirit_start - Duration::from_secs(900);
        }

        if let Some(ref shard) = shard_data {
            // Find a start timestamp that is 10 minutes before the shard eruption.
            let timestamps = shard.timestamps.iter().find(|dates| {
                let time = dates.start.signed_duration_since(now);
                (0..=10).contains(&time.num_minutes())
            });

            if let Some(dates) = timestamps {
                let r#type = if shard.strong {
                    NotificationType::ShardEruptionStrong
                } else {
                    NotificationType::ShardEruptionRegular
                };

                notification_notifies.push(NotificationNotify {
                    r#type,
                    start_time: dates.start.timestamp(),
                    end_time: Some(dates.end.timestamp()),
                    time_until_start: dates
                        .start
                        .signed_duration_since(now)
                        .num_minutes()
                        .try_into()
                        .expect("Failed to create time_until_start for a shard eruption."),
                    shard_eruption: Some(shard.clone()),
                    travelling_spirit_name: None,
                });
            }
        }

        if (hour == 23 && (45..=59).contains(&minute)) || (hour == 0 && minute == 0) {
            let time_until_start = (60 - minute) % 60;
            let date = now + Duration::from_secs((time_until_start * 60).into());

            notification_notifies.push(NotificationNotify {
                r#type: NotificationType::DailyReset,
                start_time: date.timestamp(),
                end_time: None,
                time_until_start,
                shard_eruption: None,
                travelling_spirit_name: None,
            });
        }

        if (now.weekday() == Weekday::Sat && hour == 23 && (36..=59).contains(&minute))
            || (now.weekday() == Weekday::Sun && hour == 0 && minute == 0)
        {
            let time_until_start = (60 - minute) % 60;
            let date = now + Duration::from_secs((time_until_start * 60).into());

            notification_notifies.push(NotificationNotify {
                r#type: NotificationType::EyeOfEden,
                start_time: date.timestamp(),
                end_time: None,
                time_until_start,
                shard_eruption: None,
                travelling_spirit_name: None,
            });
        }

        if (INTERNATIONAL_SPACE_STATION_PRIOR_DATES.contains(&day)
            && hour == 23
            && (45..=59).contains(&minute))
            || (INTERNATIONAL_SPACE_STATION_DATES.contains(&day) && hour == 0 && minute == 0)
        {
            let time_until_start = (60 - minute) % 60;
            let date = now + Duration::from_secs((time_until_start * 60).into());

            notification_notifies.push(NotificationNotify {
                r#type: NotificationType::InternationalSpaceStation,
                start_time: date.timestamp(),
                end_time: None,
                time_until_start,
                shard_eruption: None,
                travelling_spirit_name: None,
            });
        }

        if now >= travelling_spirit_earliest_notification_time && now <= travelling_spirit_start {
            let time_until_start = (travelling_spirit_start - now).num_minutes();

            notification_notifies.push(NotificationNotify {
                r#type: NotificationType::TravellingSpirit,
                start_time: travelling_spirit_start.timestamp(),
                end_time: None,
                time_until_start: time_until_start
                    .try_into()
                    .expect("Failed to create time_until_start for a travelling spirit."),
                shard_eruption: None,
                travelling_spirit_name: Some(travelling_spirit.entity.clone()),
            });
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
                r#type: NotificationType::Passage,
                start_time: date.timestamp(),
                end_time: None,
                time_until_start,
                shard_eruption: None,
                travelling_spirit_name: None,
            });
        }

        if (((hour % 2) == 1) && (45..=59).contains(&minute)) || (((hour % 2) == 0) && minute == 0)
        {
            let time_until_start = (60 - minute) % 60;
            let date = now + Duration::from_secs((time_until_start * 60).into());

            notification_notifies.push(NotificationNotify {
                r#type: NotificationType::Aurora,
                start_time: date.timestamp(),
                end_time: None,
                time_until_start,
                shard_eruption: None,
                travelling_spirit_name: None,
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
                r#type: NotificationType::PollutedGeyser,
                start_time: date.timestamp(),
                end_time: None,
                time_until_start,
                shard_eruption: None,
                travelling_spirit_name: None,
            });
        }

        if ((hour % 2) == 0) && ((25..=35).contains(&minute)) {
            let time_until_start = 35 - minute;
            let date = now + Duration::from_secs((time_until_start * 60).into());

            notification_notifies.push(NotificationNotify {
                r#type: NotificationType::Grandma,
                start_time: date.timestamp(),
                end_time: None,
                time_until_start,
                shard_eruption: None,
                travelling_spirit_name: None,
            });
        }

        if ((hour % 2) == 0) && ((40..=50).contains(&minute)) {
            let time_until_start = 50 - minute;
            let date = now + Duration::from_secs((time_until_start * 60).into());

            notification_notifies.push(NotificationNotify {
                r#type: NotificationType::Turtle,
                start_time: date.timestamp(),
                end_time: None,
                time_until_start,
                shard_eruption: None,
                travelling_spirit_name: None,
            });
        }

        if (day == 1
            && ((((hour % 4) == 0) && minute == 0)
                || ((hour % 4) == 3) && (45..=59).contains(&minute)))
            || (day == last_day_of_month && hour == 23 && (45..=59).contains(&minute))
        {
            let time_until_start = (60 - minute) % 60;
            let date = now + Duration::from_secs((time_until_start * 60).into());

            notification_notifies.push(NotificationNotify {
                r#type: NotificationType::AviarysFireworkFestival,
                start_time: date.timestamp(),
                end_time: None,
                time_until_start,
                shard_eruption: None,
                travelling_spirit_name: None,
            });
        }

        // if minute == 0 || (50..=59).contains(&minute) {
        //     let time_until_start = (60 - minute) % 60;
        //     let date = now + Duration::from_secs((time_until_start * 60).into());

        //     notification_notifies.push(NotificationNotify {
        //         r#type: NotificationType::Dragon,
        //         start_time: date.timestamp(),
        //         end_time: None,
        //         time_until_start,
        //         shard_eruption: None,
        //         travelling_spirit_name: None,
        //     });
        // }

        for notification_notify in notification_notifies {
            tracing::info!(
                r#type = ?notification_notify.r#type,
                until = notification_notify.time_until_start,
                "Notifications Queuing"
            );

            let send = tx.send(notification_notify).await;

            if let Err(error) = send {
                tracing::error!("Failed to queue notification: {error:?}");
            }
        }
    }
}
