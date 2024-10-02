use rpc_primitives::errors::RpcParseError;
use rpc_primitives::parse_request;
use rpc_primitives::parser::parse_params;
use rpc_primitives::parser::RpcRequest;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct HelloRequest {}

parse_request!(HelloRequest);

#[derive(Serialize, Deserialize, Debug)]
pub struct HelloResponse {
    pub greeting: String,
}
