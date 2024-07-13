use std::fmt;

pub const CDN_URL: &str = "https://cdn.thatskyapplication.com";
pub const ISS_DATES_ACCESSIBLE: [u32; 4] = [6, 14, 22, 30];

// Community-organised AURORA concerts.
pub const COMMUNITY_ORGANISED_AURORA_CONCERT_START_DATE_1: i32 = 1721480400;
pub const COMMUNITY_ORGANISED_AURORA_CONCERT_START_DATE_2: i32 = 1721523600;

// SkyFest.
pub const SKY_FEST_AVIARYS_FIREWORK_FESTIVAL_END_TIMESTAMP: i64 = 1721970000;

#[derive(Clone, Debug, PartialEq)]
pub enum SkyMap {
    // Daylight Prairie.
    BirdNest,
    ButterflyFields,
    Cave,
    KoiPond,
    SanctuaryIslands,

    // Hidden Forest.
    Boneyard,
    ElevatedClearing,
    ForestBrook,
    ForestEnd,
    Treehouse,

    // Valley of Triumph.
    IceRink,
    HermitValley,
    VillageOfDreams,

    // Golden Wasteland.
    Battlefield,
    BrokenTemple,
    CrabFields,
    ForgottenArk,
    Graveyard,

    // Vault of Knowledge.
    JellyfishCove,
    StarlightDesert,
}

impl fmt::Display for SkyMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            SkyMap::BirdNest => "Bird Nest",
            SkyMap::ButterflyFields => "Butterfly Fields",
            SkyMap::Cave => "Cave",
            SkyMap::KoiPond => "Koi Pond",
            SkyMap::SanctuaryIslands => "Sanctuary Islands",
            SkyMap::Boneyard => "Boneyard",
            SkyMap::ElevatedClearing => "Elevated Clearing",
            SkyMap::ForestBrook => "Forest Brook",
            SkyMap::ForestEnd => "Forest End",
            SkyMap::Treehouse => "Treehouse",
            SkyMap::IceRink => "Ice Rink",
            SkyMap::HermitValley => "Hermit Valley",
            SkyMap::VillageOfDreams => "Village Of Dreams",
            SkyMap::Battlefield => "Battlefield",
            SkyMap::BrokenTemple => "Broken Temple",
            SkyMap::CrabFields => "Crab Fields",
            SkyMap::ForgottenArk => "Forgotten Ark",
            SkyMap::Graveyard => "Graveyard",
            SkyMap::JellyfishCove => "Jellyfish Cove",
            SkyMap::StarlightDesert => "Starlight Desert",
        };
        write!(f, "{}", name)
    }
}
