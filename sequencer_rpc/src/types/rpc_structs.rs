use rpc_primitives::errors::RpcParseError;
use rpc_primitives::parse_request;
use rpc_primitives::parser::parse_params;
use rpc_primitives::parser::RpcRequest;
use sequencer_core::transaction_mempool::TransactionMempool;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use storage::block::Block;

#[derive(Serialize, Deserialize, Debug)]
pub struct HelloRequest {}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterAccountRequest {
    pub nullifier_public_key: Vec<u8>,
    pub viewing_public_key: Vec<u8>,
    pub address: [u8; 32],
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SendTxRequest {
    pub transaction: TransactionMempool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetBlockDataRequest {
    pub block_id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetGenesisIdRequest {}

parse_request!(HelloRequest);
parse_request!(RegisterAccountRequest);
parse_request!(SendTxRequest);
parse_request!(GetBlockDataRequest);
parse_request!(GetGenesisIdRequest);

#[derive(Serialize, Deserialize, Debug)]
pub struct HelloResponse {
    pub greeting: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterAccountResponse {
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SendTxResponse {
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetBlockDataResponse {
    pub block: Block,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetGenesisIdResponse {
    pub genesis_id: u64,
}
