#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Addr, attr, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdError, StdResult, to_json_binary};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{CHANNELS, Channels, ChannelState, Ratings, USER_PROFILES, UserProfile, ViewHistory};

const CONTRACT_NAME: &str = "crates.io:television";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attributes(vec![attr("action", "instantiate")]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateChannel { channel } => try_create_channel(deps, info, channel),
        ExecuteMsg::RemoveChannel { channel } => try_remove_channel(deps, info, channel),
        ExecuteMsg::UpdateBroadcast { channel, broadcast } => try_update_broadcast(deps, info, channel, broadcast),
        ExecuteMsg::TuneIn { channel } => try_tune_in(deps, env, info, channel),
        ExecuteMsg::RateBroadcast { channel, rating } => try_rate_broadcast(deps, info, channel, rating),
        ExecuteMsg::TransferChannelOwnership { channel, new_owner } => try_transfer_channel_ownership(deps, info, channel, new_owner),
    }
}

fn try_create_channel(
    deps: DepsMut,
    info: MessageInfo,
    channel: String,
) -> Result<Response, ContractError> {
    let state = ChannelState {
        owner: info.sender.clone(),
        broadcast: String::new(),
        ratings: vec![],
        viewer_count: 0,
    };

    CHANNELS.save(deps.storage, &channel, &state)?;

    Ok(Response::new().add_attributes(vec![
        attr("method", "create_channel"),
        attr("channel", channel)
    ]))
}

fn try_remove_channel(
    deps: DepsMut,
    info: MessageInfo,
    channel: String,
) -> Result<Response, ContractError> {
    let state = CHANNELS.load(deps.storage, &channel)?;

    if state.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    CHANNELS.remove(deps.storage, &channel);

    Ok(Response::new().add_attributes(vec![
        attr("method", "remove_channel"),
        attr("channel", channel)
    ]))
}

fn try_update_broadcast(
    deps: DepsMut,
    info: MessageInfo,
    channel: String,
    broadcast: String,
) -> Result<Response, ContractError> {
    let mut state = CHANNELS.load(deps.storage, &channel)?;

    if state.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    state.broadcast = broadcast;
    CHANNELS.save(deps.storage, &channel, &state)?;

    Ok(Response::new().add_attributes(vec![
        attr("method", "update_broadcast"),
        attr("channel", channel)
    ]))
}

fn try_tune_in(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    channel: String,
) -> Result<Response, ContractError> {
    let mut profile = USER_PROFILES.may_load(deps.storage, &info.sender)?
        .unwrap_or_else(|| UserProfile {
            current_channel: None,
            viewing_history: vec![],
        });

    if let Some(current_channel) = &profile.current_channel {
        if current_channel == &channel {
            return Err(ContractError::Std(StdError::generic_err("User is already tuned in to this channel")));
        }

        let mut current_state = CHANNELS.load(deps.storage, current_channel)?;
        current_state.viewer_count = current_state.viewer_count.saturating_sub(1);
        CHANNELS.save(deps.storage, current_channel, &current_state)?;
    }

    return match CHANNELS.may_load(deps.storage, &channel)? {
        Some(mut state) => {
            state.viewer_count += 1;
            CHANNELS.save(deps.storage, &channel, &state)?;

            profile.current_channel = Some(channel.clone());
            profile.viewing_history.push(ViewHistory {
                channel: channel.clone(),
                start_time: env.block.time.seconds(),
                start_height: env.block.height,
            });
            USER_PROFILES.save(deps.storage, &info.sender, &profile)?;

            Ok(Response::new().add_attribute("method", "tune_in").add_attribute("channel", channel))
        },
        None => Err(ContractError::Std(StdError::generic_err("Channel not found")))
    }
}

fn try_rate_broadcast(
    deps: DepsMut,
    info: MessageInfo,
    channel: String,
    rating: u8,
) -> Result<Response, ContractError> {
    let profile = USER_PROFILES.load(deps.storage, &info.sender)?;
    let current_channel = profile.current_channel.as_deref().ok_or_else(|| ContractError::Std(StdError::generic_err("User is not tuned in to any channel")))?;

    if current_channel != channel {
        return Err(ContractError::Std(StdError::generic_err("User is not tuned in to the specified channel")));
    }

    let mut state = CHANNELS.load(deps.storage, &channel)?;

    state.ratings.push(rating);
    CHANNELS.save(deps.storage, &channel, &state)?;

    Ok(Response::new().add_attribute("method", "rate_broadcast").add_attribute("channel", channel))
}

fn try_transfer_channel_ownership(
    deps: DepsMut,
    info: MessageInfo,
    channel: String,
    new_owner: Addr,
) -> Result<Response, ContractError> {
    let mut state = CHANNELS.load(deps.storage, &channel)?;

    if state.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    state.owner = new_owner;
    CHANNELS.save(deps.storage, &channel, &state)?;

    Ok(Response::new().add_attribute("method", "transfer_channel_ownership").add_attribute("channel", channel))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCurrentBroadcast { channel } => to_json_binary(&query_current_broadcast(deps, channel)?),
        QueryMsg::ListChannels {} => to_json_binary(&query_list_channels(deps)?),
        QueryMsg::GetUserProfile { user } => to_json_binary(&query_user_profile(deps, user)?),
        QueryMsg::GetChannelRatings { channel } => to_json_binary(&query_channel_ratings(deps, channel)?),
        QueryMsg::GetChannelViewers { channel } => to_json_binary(&query_channel_viewers(deps, channel)?),
    }
}

fn query_current_broadcast(
    deps: Deps,
    channel: String,
) -> StdResult<String> {
    let state = CHANNELS.load(deps.storage, &channel)?;
    Ok(state.broadcast)
}

fn query_list_channels(
    deps: Deps,
) -> StdResult<Channels> {
    let channels: Vec<String> = CHANNELS
        .keys(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?
        .iter()
        .map(|key| key.to_string())
        .collect();

    Ok(channels)
}

fn query_user_profile(
    deps: Deps,
    user: Addr,
) -> StdResult<UserProfile> {
    USER_PROFILES.load(deps.storage, &user)
}

fn query_channel_ratings(
    deps: Deps,
    channel: String,
) -> StdResult<Ratings> {
    let state = CHANNELS.load(deps.storage, &channel)?;
    Ok(state.ratings)
}

fn query_channel_viewers(
    deps: Deps,
    channel: String,
) -> StdResult<u64> {
    let state = CHANNELS.load(deps.storage, &channel)?;
    Ok(state.viewer_count)
}
