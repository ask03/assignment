#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE, SCORES};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:counter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    //validate inputted owner address
    let validatedOwner = deps.api.addr_validate(&msg.owner)?;
    let state = State {
        owner: validatedOwner,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", msg.owner)
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetScore{ address, score } => try_set_score(deps, info, address, score),
    }
}

pub fn try_set_score(deps: DepsMut, info: MessageInfo, address: String, score: i32) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    if info.sender != state.owner {
        return Err(ContractError::Unauthorized {});
    }
    let addr = deps.api.addr_validate(&address)?;
    // inline function to check score for address and update accordingly
    let score_entry = |num_entries: Option<i32>| -> StdResult<i32> {
        match num_entries {
            Some(number) => Ok(score),
            None => Ok(score),
        }
    };
    SCORES.update(deps.storage, &addr, score_entry)?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetOwner {} => {
            let state = STATE.load(deps.storage)?;
            to_binary(&state.owner)
        },
        QueryMsg::GetScore { address } => {
            let valid_addr = deps.api.addr_validate(&address)?;
            let raw_score = SCORES.load(deps.storage, &valid_addr)?;
            to_binary(&raw_score)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg { owner: "creator".to_string() };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

    }

    #[test]
    fn query_owner() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg { owner: "creator".to_string() };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOwner {}).unwrap();
        let value: String = from_binary(&res).unwrap();
        assert_eq!("creator".to_string(), value );
    }

    #[test]
    fn set_scores() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg { owner: "creator".to_string() };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // owner sets score for address_1
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = ExecuteMsg::SetScore { address: "address_1".to_string(), score: 30};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // someone other than owner tries to set score (only owner should be able to set score)
        let info = mock_info("address_1", &coins(2, "token"));
        let msg = ExecuteMsg::SetScore { address: "address_1".to_string(), score: 30};
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res{
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // owner sets score for address_2
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = ExecuteMsg::SetScore { address: "address_2".to_string(), score: 50};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // query score for address_1 (should be 30)
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetScore { address: "address_1".to_string()}).unwrap();
        let value: i32 = from_binary(&res).unwrap();
        assert_eq!(30, value);

        // query score for address_2 (should be 50)
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetScore { address: "address_2".to_string()}).unwrap();
        let value: i32 = from_binary(&res).unwrap();
        assert_eq!(50, value);
    }

}
