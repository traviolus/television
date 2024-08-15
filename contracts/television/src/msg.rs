use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use crate::state::{Channels, Ratings, UserProfile};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    CreateChannel {
        channel: String,
    },
    RemoveChannel {
        channel: String,
    },
    UpdateBroadcast {
        channel: String,
        broadcast: String,
    },
    TuneIn {
        channel: String,
    },
    RateBroadcast {
        channel: String,
        rating: u8,
    },
    TransferChannelOwnership {
        channel: String,
        new_owner: Addr,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(String)]
    GetCurrentBroadcast {
        channel: String,
    },
    #[returns(Channels)]
    ListChannels {},
    #[returns(UserProfile)]
    GetUserProfile {
        user: Addr,
    },
    #[returns(Ratings)]
    GetChannelRatings {
        channel: String,
    },
    #[returns(u64)]
    GetChannelViewers {
        channel: String,
    },
}
