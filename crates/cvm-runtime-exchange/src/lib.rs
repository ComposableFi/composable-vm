pub mod error;
mod osmosis_std;

use cosmwasm_std::{
    ensure_eq, to_json_binary, Addr, Binary, Coin, CosmosMsg, DepsMut, Response, SubMsg, WasmMsg,
};
use cvm_route::exchange::ExchangeItem;
use cvm_runtime::{Amount, ExchangeId,  Funds};
use error::ContractError;
use osmosis_std::types::osmosis::poolmanager::v1beta1::MsgSwapExactAmountIn;

pub fn exchange(
    give: Funds,
    want: Funds,
    gateway_address: cvm_runtime::outpost::Gateway,
    deps: &mut DepsMut<'_>,
    sender: Addr,
    exchange_id: &ExchangeId,
    exchange: ExchangeItem,
    response_id: u64,
) -> Result<Response, ContractError> {
    use cvm_route::exchange::ExchangeType::*;
    use prost::Message;
    ensure_eq!(
        give.0.len(),
        1,
        ContractError::OnlySingleAssetExchangeIsSupportedByPool
    );
    ensure_eq!(
        want.0.len(),
        1,
        ContractError::OnlySingleAssetExchangeIsSupportedByPool
    );
    let give = give.0[0].clone();
    let want = want.0[0].clone();
    let give_asset = gateway_address
        .get_asset_by_id(deps.querier, give.0)
        .map_err(ContractError::AssetNotFound)?;
    let amount: Coin = deps.querier.query_balance(&sender, give_asset.denom())?;
    let amount = give.1.apply(amount.amount.u128())?;
    let give: ibc_apps_more::cosmos::Coin = ibc_apps_more::cosmos::Coin {
        denom: give_asset.denom(),
        amount: amount.to_string(),
    };
    let want_asset = gateway_address
        .get_asset_by_id(deps.querier, want.0)
        .map_err(ContractError::AssetNotFound)?;
    if want.1.is_absolute() && want.1.is_ratio() {
        return Err(ContractError::CannotDefineBothSlippageAndLimitAtSameTime);
    }
    let response = Response::default().add_attribute("exchange_id", exchange_id.to_string());
    let response = match exchange.exchange {
        OsmosisPoolManagerModuleV1Beta1 { pool_id, .. } => {
            let want = if want.1.is_absolute() {
                ibc_apps_more::cosmos::Coin {
                    denom: want_asset.denom(),
                    amount: want.1.intercept.to_string(),
                }
            } else {
                // use https://github.com/osmosis-labs/osmosis/blob/main/cosmwasm/contracts/swaprouter/src/msg.rs to allow slippage
                ibc_apps_more::cosmos::Coin {
                    denom: want_asset.denom(),
                    amount: "1".to_string(),
                }
            };

            use crate::osmosis_std::types::osmosis::poolmanager::v1beta1::*;
            use prost::Message;
            let msg = MsgSwapExactAmountIn {
                routes: vec![SwapAmountInRoute {
                    pool_id,
                    token_out_denom: want.denom,
                }],

                sender: sender.to_string(),
                token_in: Some(give),
                token_out_min_amount: want.amount,
            };

            deps.api
                .debug(&format!("cvm::executor::execute::exchange {:?}", &msg));
            let msg = CosmosMsg::Stargate {
                type_url: MsgSwapExactAmountIn::TYPE_URL.to_string(),
                value: Binary::from(msg.encode_to_vec()),
            };
            let msg = SubMsg::reply_always(msg, response_id);
            response.add_submessage(msg)
        }
        AstroportRouterContract {
            address,
            token_a,
            token_b,
        } => {
            use astroport::{asset::*, router::*};
            let (minimum_receive, max_spread) = if want.1.is_absolute() {
                (Some(want.1.intercept.into()), None)
            } else {
                (
                    None,
                    Some(cosmwasm_std::Decimal::from_ratio(
                        (Amount::MAX_PARTS - want.1.slope.0) as u128,
                        Amount::MAX_PARTS,
                    )),
                )
            };
            let msg = ExecuteMsg::ExecuteSwapOperations {
                operations: vec![SwapOperation::AstroSwap {
                    offer_asset_info: AssetInfo::NativeToken {
                        denom: give.denom.clone(),
                    },
                    ask_asset_info: AssetInfo::NativeToken {
                        denom: want_asset.denom(),
                    },
                }],
                to: None,
                minimum_receive,
                max_spread,
            };
            let msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: address.to_string(),
                msg: to_json_binary(&msg)?,
                funds: vec![give.try_into().expect("coin")],
            });
            let msg = SubMsg::reply_always(msg, response_id);
            response.add_submessage(msg)
        }
    };
    Ok(response)
}
