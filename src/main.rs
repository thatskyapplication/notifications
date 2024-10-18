mod structures;
mod utility;
use anyhow::{Context, Result};
use chrono::{Datelike, Timelike, Utc, Weekday};
use dotenvy::dotenv;
use serenity::http::Http;
use sqlx::postgres::PgPoolOptions;
use std::{env, time::Duration};
use structures::{
    notification::{prepare_notification_to_send, NotificationNotify, NotificationType},
    shard_eruption,
};
use tokio::{sync::mpsc, time::sleep};
use utility::{
    constants::{
        INTERNATIONAL_SPACE_STATION_DATES, INTERNATIONAL_SPACE_STATION_PRIOR_DATES,
        MAXIMUM_CHANNEL_CAPACITY,
    },
    functions::last_day_of_month,
};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    dotenv().ok();
    let discord_token = env::var("DISCORD_TOKEN").context("Error retrieving DISCORD_TOKEN")?;
    let database_url = env::var("DATABASE_URL").context("Error retrieving DATABASE_URL")?;

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await?;

    let client = Http::new(&discord_token);
    let (tx, mut rx) = mpsc::channel::<NotificationNotify>(MAXIMUM_CHANNEL_CAPACITY);

    tokio::spawn(async move {
        if let Err(error) = notify(tx).await {
            tracing::error!("Error in notifying: {error:?}");
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

async fn notify(tx: mpsc::Sender<NotificationNotify>) -> Result<()> {
    let initialised_shard_eruption = shard_eruption::initialise_shard_eruption();
    let mut shard_eruption = initialised_shard_eruption.shard();

    loop {
        sleep(Duration::from_millis(
            60000 - (Utc::now().timestamp_millis() % 60000) as u64,
        ))
        .await;

        let now = Utc::now().with_timezone(&chrono_tz::America::Los_Angeles);
        let (day, hour, minute) = (now.day(), now.hour(), now.minute());
        let last_day_of_month = last_day_of_month(now);
        let mut notification_notifies = vec![];

        if hour == 0 && minute == 0 {
            // Update the shard eruption.
            shard_eruption = initialised_shard_eruption.shard();
        }

        if let Some(ref shard) = shard_eruption {
            // Find a start timestamp that is 5 minutes before the shard eruption.
            let timestamps = shard.timestamps.iter().find(|(start, _)| {
                let time = start.signed_duration_since(now);
                (0..=10).contains(&time.num_minutes())
            });

            if let Some((start_time, end_time)) = timestamps {
                let r#type = if shard.strong {
                    NotificationType::ShardEruptionStrong
                } else {
                    NotificationType::ShardEruptionRegular
                };

                notification_notifies.push(NotificationNotify {
                    r#type,
                    start_time: start_time.timestamp(),
                    end_time: Some(end_time.timestamp()),
                    time_until_start: start_time
                        .signed_duration_since(now)
                        .num_minutes()
                        .try_into()
                        .expect("Failed to create time_until_start for a shard eruption."),
                    shard_eruption: Some(shard.clone()),
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
            });
        }

        if ((((hour + 3) % 4) == 0) && (45..=59).contains(&minute))
            || ((((hour + 2) % 4) == 0) && minute == 0)
        {
            let time_until_start = (60 - minute) % 60;
            let date = now + Duration::from_secs((time_until_start * 60).into());

            notification_notifies.push(NotificationNotify {
                r#type: NotificationType::Aurora,
                start_time: date.timestamp(),
                end_time: None,
                time_until_start,
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
                r#type: NotificationType::PollutedGeyser,
                start_time: date.timestamp(),
                end_time: None,
                time_until_start,
                shard_eruption: None,
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
        //     });
        // }

        for notification_notify in notification_notifies {
            let r#type = &notification_notify.r#type;
            tracing::info!("{}", r#type);

            if tx.send(notification_notify).await.is_err() {
                tracing::error!("Failed to queue notification.");
            }
        }
    }
}
