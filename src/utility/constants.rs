use serde::{Deserialize, Serialize};
use std::fmt;

pub const MAXIMUM_CHANNEL_CAPACITY: usize = 10;
pub const INTERNATIONAL_SPACE_STATION_DATES: [u32; 4] = [6, 14, 22, 30];
pub const INTERNATIONAL_SPACE_STATION_PRIOR_DATES: [u32; 4] = [5, 13, 21, 29];

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum SkyMap {
    // Daylight Prairie.
    #[serde(rename = "Bird Nest")]
    BirdNest,
    #[serde(rename = "Butterfly Fields")]
    ButterflyFields,
    Cave,
    #[serde(rename = "Koi Pond")]
    KoiPond,
    #[serde(rename = "Sanctuary Islands")]
    SanctuaryIslands,

    // Hidden Forest.
    Boneyard,
    #[serde(rename = "Elevated Clearing")]
    ElevatedClearing,
    #[serde(rename = "Forest Brook")]
    ForestBrook,
    #[serde(rename = "Forest End")]
    ForestEnd,
    Treehouse,

    // Valley of Triumph.
    #[serde(rename = "Ice Rink")]
    IceRink,
    #[serde(rename = "Hermit Valley")]
    HermitValley,
    #[serde(rename = "Village of Dreams")]
    VillageOfDreams,

    // Golden Wasteland.
    Battlefield,
    #[serde(rename = "Broken Temple")]
    BrokenTemple,
    #[serde(rename = "Crab Fields")]
    CrabFields,
    #[serde(rename = "Forgotten Ark")]
    ForgottenArk,
    Graveyard,

    // Vault of Knowledge.
    #[serde(rename = "Jellyfish Cove")]
    JellyfishCove,
    #[serde(rename = "Starlight Desert")]
    StarlightDesert,
}

impl fmt::Display for SkyMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).unwrap().trim_matches('"')
        )
    }
}
