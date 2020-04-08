use std::sync::Arc;
use sp_blockchain::HeaderBackend;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_runtime::{
    generic::BlockId,
    traits::{Block as BlockT},
};
use sp_api::ProvideRuntimeApi;
use did_offchain_trade_rpc_runtime_api::DIDOffchainTradeApi as TradeRuntimeApi;

#[rpc]
pub trait DIDOffchainTradeApi<BlockHash> {
    #[rpc(name = "didTrade_getNonce")]
    fn get_nonce(
        &self,
        at: Option<BlockHash>
    ) -> Result<i32>;

    #[rpc(name = "didTrade_getStatus")]
    fn get_status(
        &self,
        at: Option<BlockHash>
    ) -> Result<i32>;

    #[rpc(name = "didTrade_getOwner")]
    fn get_owner(
        &self,
        at: Option<BlockHash>
    ) -> Result<u64>;

    #[rpc(name = "didTrade_getGrantee")]
    fn get_grantee(
        &self,
        at: Option<BlockHash>
    ) -> Result<u64>;

    #[rpc(name = "didTrade_getDIDKey")]
    fn get_did_key(
        &self,
        at: Option<BlockHash> 
    ) -> Result<i32>;

    #[rpc(name = "didTrade_accessConditionAddressKey")]
    fn access_condition_address_key(
        &self,
        at: Option<BlockHash>
    ) -> Result<i32>;
}

pub struct DIDOffchainTrade<C, M> {
    client: Arc<C>,
    _market: std::marker::PhantomData<M>,
}

impl<C, Block> DIDOffchainTradeApi<<Block as BlockT>::Hash>
    for DIDOffchainTrade<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: TradeRuntimeApi<Block>,
{
    fn get_nonce(
        &self,
        at: Option<<Block as BlockT>::Hash>
    ) -> Result<i32> {

        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| 
            self.client.info().best_hash
        ));

        let runtime_api_resuult = api.get_nonce(&at);
        runtime_api_resuult.map_err(|e| RpcError {
            code: ErrorCode::ServerError(9876),
            message: "Error".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }

    fn get_status(
        &self,
        at: Option<<Block as BlockT>::Hash>
    ) -> Result<i32> {

        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| 
            self.client.info().best_hash
        ));

        let runtime_api_resuult = api.get_status(&at);
        runtime_api_resuult.map_err(|e| RpcError {
            code: ErrorCode::ServerError(9876),
            message: "Error".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }

    fn get_owner(
        &self,
        at: Option<<Block as BlockT>::Hash>
    ) -> Result<u64> {

        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| 
            self.client.info().best_hash
        ));

        let runtime_api_resuult = api.get_owner(&at);
        runtime_api_resuult.map_err(|e| RpcError {
            code: ErrorCode::ServerError(9876),
            message: "Error".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }

    fn get_grantee(
        &self,
        at: Option<<Block as BlockT>::Hash>
    ) -> Result<u64> {

        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| 
            self.client.info().best_hash
        ));

        let runtime_api_resuult = api.get_grantee(&at);
        runtime_api_resuult.map_err(|e| RpcError {
            code: ErrorCode::ServerError(9876),
            message: "Error".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }

    fn get_did_key(
        &self,
        at: Option<<Block as BlockT>::Hash>
    ) -> Result<i32> {

        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| 
            self.client.info().best_hash
        ));

        let runtime_api_resuult = api.get_did_key(&at);
        runtime_api_resuult.map_err(|e| RpcError {
            code: ErrorCode::ServerError(9876),
            message: "Error".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }

    fn access_condition_address_key(
        &self,
        at: Option<<Block as BlockT>::Hash>
    ) -> Result<i32> {

        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| 
            self.client.info().best_hash
        ));

        let runtime_api_resuult = api.access_condition_address_key(&at);
        runtime_api_resuult.map_err(|e| RpcError {
            code: ErrorCode::ServerError(9876),
            message: "Error".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }
}