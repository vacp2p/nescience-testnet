use rpc_primitives::errors::RpcParseError;
use rpc_primitives::parse_request;
use rpc_primitives::parser::parse_params;
use rpc_primitives::parser::RpcRequest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use storage::block::Block;
use storage::transaction::Transaction;
use storage::transaction::TxKind;

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterAccountRequest {}

#[derive(Serialize, Deserialize, Debug)]
pub struct SendTxRequest {
    pub transaction: Transaction,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetBlockDataRequest {
    pub block_id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExecuteSubscenarioRequest {
    pub scenario_id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExecuteScenarioSplitRequest {
    pub visibility_list: [bool; 3],
    pub publication_index: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExecuteScenarioMultipleSendRequest {
    pub number_of_assets: usize,
    pub number_to_send: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetGenesisIdRequest {}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetLastBlockRequest {}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShowAccountPublicBalanceRequest {
    pub account_addr: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShowAccountUTXORequest {
    pub account_addr: String,
    pub utxo_hash: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShowTransactionRequest {
    pub tx_hash: String,
}

parse_request!(RegisterAccountRequest);
parse_request!(SendTxRequest);
parse_request!(GetBlockDataRequest);
parse_request!(GetGenesisIdRequest);
parse_request!(ExecuteSubscenarioRequest);
parse_request!(ExecuteScenarioSplitRequest);
parse_request!(ExecuteScenarioMultipleSendRequest);
parse_request!(GetLastBlockRequest);
parse_request!(ShowAccountPublicBalanceRequest);
parse_request!(ShowAccountUTXORequest);
parse_request!(ShowTransactionRequest);

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
pub struct ExecuteSubscenarioResponse {
    pub scenario_result: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExecuteScenarioSplitResponse {
    pub scenario_result: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExecuteScenarioMultipleSendResponse {
    pub scenario_result: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetGenesisIdResponse {
    pub genesis_id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetLastBlockResponse {
    pub last_block: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShowAccountPublicBalanceResponse {
    pub addr: String,
    pub balance: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShowAccountUTXOResponse {
    pub hash: String,
    pub asset: Vec<u8>,
    pub amount: u128,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShowTransactionResponse {
    pub hash: String,
    pub tx_kind: TxKind,
    pub public_input: String,
    pub public_output: String,
    pub utxo_commitments_created_hashes: Vec<String>,
    pub utxo_commitments_spent_hashes: Vec<String>,
    pub utxo_nullifiers_created_hashes: Vec<String>,
    pub encoded_data: Vec<(String, String)>,
    pub ephemeral_pub_key: String,
}
