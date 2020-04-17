// Copyright 2019-2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

use std::sync::Arc;
use sp_blockchain::HeaderBackend;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_runtime::{
    generic::BlockId,
    traits::{Block as BlockT},
};
use sp_api::ProvideRuntimeApi;
use codec::Codec;
use did_offchain_trade_rpc_runtime_api::DIDOffchainTradeApi as TradeRuntimeApi;

#[rpc]
pub trait DIDOffchainTradeApi<BlockHash, AccountId, AppStatus>  
    where
        AccountId: Codec,
        AppStatus: Codec,

{
    #[rpc(name = "didTrade_getNonce")]
    fn get_nonce(
        &self,
        condition_address: AccountId,
        at: Option<BlockHash>
    ) -> Result<Option<u32>>;

    #[rpc(name = "didTrade_getStatus")]
    fn get_status(
        &self,
        condition_address: AccountId,
        at: Option<BlockHash>
    ) -> Result<Option<u32>>;

    #[rpc(name = "didTrade_getOwner")]
    fn get_owner(
        &self,
        condition_address: AccountId,
        at: Option<BlockHash>
    ) -> Result<Option<AccountId>>;

    #[rpc(name = "didTrade_getGrantee")]
    fn get_grantee(
        &self,
        condition_address: AccountId,
        at: Option<BlockHash>
    ) -> Result<Option<AccountId>>;
}

pub struct DIDOffchainTrade<C, M> {
    client: Arc<C>,
    _market: std::marker::PhantomData<M>,
}

impl<C, Block> DIDOffchainTradeApi<<Block as BlockT>::Hash, AccountId, AppStatus>
    for DIDOffchainTrade<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: TradeRuntimeApi<Block>,
    AccountId: Codec,
    AppStatus: Codec,
{
    fn get_nonce(
        &self,
        condition_address: AccountId,
        at: Option<<Block as BlockT>::Hash>
    ) -> Result<i32> {

        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| 
            self.client.info().best_hash
        ));

        let runtime_api_resuult = api.get_nonce(condition_address, &at);
        runtime_api_resuult.map_err(|e| RpcError {
            code: ErrorCode::ServerError(9876),
            message: "Error".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }

    fn get_status(
        &self,
        condition_address: AccountId,
        at: Option<<Block as BlockT>::Hash>
    ) -> Result<i32> {

        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| 
            self.client.info().best_hash
        ));

        let runtime_api_resuult = api.get_status(condition_address, &at);
        runtime_api_resuult.map_err(|e| RpcError {
            code: ErrorCode::ServerError(9876),
            message: "Error".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }

    fn get_owner(
        &self,
        condition_address: AccountId,
        at: Option<<Block as BlockT>::Hash>
    ) -> Result<u64> {

        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| 
            self.client.info().best_hash
        ));

        let runtime_api_resuult = api.get_owner(condition_address, &at);
        runtime_api_resuult.map_err(|e| RpcError {
            code: ErrorCode::ServerError(9876),
            message: "Error".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }

    fn get_grantee(
        &self,
        condition_address:  AccountId,
        at: Option<<Block as BlockT>::Hash>
    ) -> Result<u64> {

        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| 
            self.client.info().best_hash
        ));

        let runtime_api_resuult = api.get_grantee(condition_address, &at);
        runtime_api_resuult.map_err(|e| RpcError {
            code: ErrorCode::ServerError(9876),
            message: "Error".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }
}