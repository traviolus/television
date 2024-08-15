use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Map;

#[cw_serde]
pub struct ChannelState {
    pub owner: Addr,
    pub broadcast: String,
    pub ratings: Vec<u8>,
    pub viewer_count: u64,
}

#[cw_serde]
pub struct ViewHistory {
    pub channel: String,
    pub start_time: u64,
    pub start_height: u64,
}

#[cw_serde]
pub struct UserProfile {
    pub current_channel: Option<String>,
    pub viewing_history: Vec<ViewHistory>,
}

pub type Channels = Vec<String>;
pub type Ratings = Vec<u8>;

pub const CHANNELS: Map<&str, ChannelState> = Map::new("channels");
pub const USER_PROFILES: Map<&Addr, UserProfile> = Map::new("user_profiles");
