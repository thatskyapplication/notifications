use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(FromRow)]
pub struct TravellingSpiritPacket {
    entity: String,
    pub start: DateTime<Utc>,
}

pub struct TravellingSpirit {
    pub entity: String,
    pub start: DateTime<chrono_tz::Tz>,
}

pub async fn get_last_travelling_spirit(pool: &sqlx::PgPool) -> TravellingSpirit {
    let row: TravellingSpiritPacket = sqlx::query_as(
        r#"select "entity", "start" from travelling_spirits order by visit desc limit 1;"#,
    )
    .fetch_one(pool)
    .await
    .expect("Failed to fetch the travelling spirit.");

    TravellingSpirit {
        entity: row.entity,
        start: row.start.with_timezone(&chrono_tz::America::Los_Angeles),
    }
}
