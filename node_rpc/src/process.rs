use actix_web::Error as HttpError;
use serde_json::Value;

use rpc_primitives::{
    errors::RpcError,
    message::{Message, Request},
    parser::RpcRequest,
};

use crate::{
    rpc_error_responce_inverter,
    types::{
        err_rpc::cast_seq_client_error_into_rpc_error,
        rpc_structs::{RegisterAccountRequest, RegisterAccountResponse, SendTxRequest},
    },
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

    // async fn process_register_account(&self, request: Request) -> Result<Value, RpcErr> {
    //     let req = RegisterAccountRequest::parse(Some(request.params))?;

    //     {
    //         let guard = self.node_chain_store.lock().await;

    //         guard
    //             .sequencer_client
    //             .register_account(&guard.main_acc)
    //             .await
    //             .map_err(cast_seq_client_error_into_rpc_error)?;
    //     }

    //     let helperstruct = RegisterAccountResponse {
    //         status: "success".to_string(),
    //     };

    //     respond(helperstruct)
    // }

    async fn process_send_tx(&self, request: Request) -> Result<Value, RpcErr> {
        let req = SendTxRequest::parse(Some(request.params))?;

        {
            let guard = self.node_chain_store.lock().await;

            guard
                .sequencer_client
                .send_tx(req.transaction)
                .await
                .map_err(cast_seq_client_error_into_rpc_error)?;
        }

        let helperstruct = RegisterAccountResponse {
            status: "success".to_string(),
        };

        respond(helperstruct)
    }

    pub async fn process_request_internal(&self, request: Request) -> Result<Value, RpcErr> {
        match request.method.as_ref() {
            //Todo : Add handling of more JSON RPC methods
            //"register_account" => self.process_register_account(request).await,
            "send_tx" => self.process_send_tx(request).await,
            _ => Err(RpcErr(RpcError::method_not_found(request.method))),
        }
    }
}
