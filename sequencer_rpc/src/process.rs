use actix_web::Error as HttpError;
use serde_json::Value;

use rpc_primitives::{
    errors::RpcError,
    message::{Message, Request},
    parser::RpcRequest,
};

use crate::{
    rpc_error_responce_inverter,
    types::rpc_structs::{HelloRequest, HelloResponse},
};

use super::{respond, types::err_rpc::RpcErr, JsonHandler};

impl JsonHandler {
    pub async fn process(&self, message: Message) -> Result<Message, HttpError> {
        let id = message.id();
        if let Message::Request(request) = message {
            let message_inner = self
                .process_request_internal(request)
                .await
                .map_err(|e| e.0)
                .map_err(rpc_error_responce_inverter);
            Ok(Message::response(id, message_inner))
        } else {
            Ok(Message::error(RpcError::parse_error(
                "JSON RPC Request format was expected".to_owned(),
            )))
        }
    }

    #[allow(clippy::unused_async)]
    ///Example of request processing
    async fn process_temp_hello(&self, request: Request) -> Result<Value, RpcErr> {
        let _hello_request = HelloRequest::parse(Some(request.params))?;

        let helperstruct = HelloResponse {
            greeting: "HELLO_FROM_SEQUENCER".to_string(),
        };

        respond(helperstruct)
    }

    pub async fn process_request_internal(&self, request: Request) -> Result<Value, RpcErr> {
        match request.method.as_ref() {
            //Todo : Add handling of more JSON RPC methods
            "hello" => self.process_temp_hello(request).await,
            _ => Err(RpcErr(RpcError::method_not_found(request.method))),
        }
    }
}
