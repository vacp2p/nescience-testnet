use std::sync::atomic::Ordering;

use actix_web::Error as HttpError;
use serde_json::Value;

use rpc_primitives::{
    errors::RpcError,
    message::{Message, Request},
    parser::RpcRequest,
};
use storage::transaction::ActionData;

use crate::{
    rpc_error_responce_inverter,
    types::{
        err_rpc::cast_seq_client_error_into_rpc_error,
        rpc_structs::{
            ExecuteScenarioMultipleSendRequest, ExecuteScenarioMultipleSendResponse,
            ExecuteScenarioSplitRequest, ExecuteScenarioSplitResponse, ExecuteSubscenarioRequest,
            ExecuteSubscenarioResponse, GetBlockDataRequest, GetBlockDataResponse,
            GetLastBlockRequest, GetLastBlockResponse, RegisterAccountRequest,
            RegisterAccountResponse, SendTxRequest, ShowAccountPublicBalanceRequest,
            ShowAccountPublicBalanceResponse, ShowAccountUTXORequest, ShowAccountUTXOResponse,
            ShowTransactionRequest, ShowTransactionResponse,
        },
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

    async fn process_request_execute_subscenario(&self, request: Request) -> Result<Value, RpcErr> {
        let req = ExecuteSubscenarioRequest::parse(Some(request.params))?;

        {
            let mut store = self.node_chain_store.lock().await;

            match req.scenario_id {
                1 => store.subscenario_1().await,
                2 => store.subscenario_2().await,
                3 => store.subscenario_3().await,
                4 => store.subscenario_4().await,
                5 => store.subscenario_5().await,
                _ => return Err(RpcErr(RpcError::invalid_params("Scenario id not found"))),
            }
        }

        let helperstruct = ExecuteSubscenarioResponse {
            scenario_result: "success".to_string(),
        };

        respond(helperstruct)
    }

    async fn process_request_execute_scenario_split(
        &self,
        request: Request,
    ) -> Result<Value, RpcErr> {
        let req = ExecuteScenarioSplitRequest::parse(Some(request.params))?;

        {
            let mut store = self.node_chain_store.lock().await;

            store
                .scenario_1(req.visibility_list, req.publication_index)
                .await;
        }

        let helperstruct = ExecuteScenarioSplitResponse {
            scenario_result: "success".to_string(),
        };

        respond(helperstruct)
    }

    async fn process_request_execute_scenario_multiple_send(
        &self,
        request: Request,
    ) -> Result<Value, RpcErr> {
        let req = ExecuteScenarioMultipleSendRequest::parse(Some(request.params))?;

        {
            let mut store = self.node_chain_store.lock().await;

            store
                .scenario_2(req.number_of_assets, req.number_to_send)
                .await;
        }

        let helperstruct = ExecuteScenarioMultipleSendResponse {
            scenario_result: "success".to_string(),
        };

        respond(helperstruct)
    }

    async fn process_register_account(&self, request: Request) -> Result<Value, RpcErr> {
        let _req = RegisterAccountRequest::parse(Some(request.params))?;

        let acc_addr = {
            let mut guard = self.node_chain_store.lock().await;

            guard.create_new_account().await
        };

        let helperstruct = RegisterAccountResponse {
            status: hex::encode(acc_addr),
        };

        respond(helperstruct)
    }

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

    async fn process_get_block_data(&self, request: Request) -> Result<Value, RpcErr> {
        let req = GetBlockDataRequest::parse(Some(request.params))?;

        let block = {
            let guard = self.node_chain_store.lock().await;

            {
                let read_guard = guard.storage.read().await;

                read_guard.block_store.get_block_at_id(req.block_id)?
            }
        };

        let helperstruct = GetBlockDataResponse { block };

        respond(helperstruct)
    }

    async fn process_get_last_block(&self, request: Request) -> Result<Value, RpcErr> {
        let _req = GetLastBlockRequest::parse(Some(request.params))?;

        let last_block = {
            let guard = self.node_chain_store.lock().await;

            guard.curr_height.load(Ordering::Relaxed)
        };

        let helperstruct = GetLastBlockResponse { last_block };

        respond(helperstruct)
    }

    async fn process_show_account_public_balance(&self, request: Request) -> Result<Value, RpcErr> {
        let req = ShowAccountPublicBalanceRequest::parse(Some(request.params))?;

        let acc_addr_hex_dec = hex::decode(req.account_addr.clone()).map_err(|_| {
            RpcError::parse_error("Failed to decode account address from hex string".to_string())
        })?;

        let acc_addr: [u8; 32] = acc_addr_hex_dec.try_into().map_err(|_| {
            RpcError::parse_error("Failed to parse account address from bytes".to_string())
        })?;

        let balance = {
            let cover_guard = self.node_chain_store.lock().await;

            {
                let under_guard = cover_guard.storage.read().await;

                let acc = under_guard
                    .acc_map
                    .get(&acc_addr)
                    .ok_or(RpcError::new_internal_error(None, "Account not found"))?;

                acc.balance
            }
        };

        let helperstruct = ShowAccountPublicBalanceResponse {
            addr: req.account_addr,
            balance,
        };

        respond(helperstruct)
    }

    async fn process_show_account_utxo_request(&self, request: Request) -> Result<Value, RpcErr> {
        let req = ShowAccountUTXORequest::parse(Some(request.params))?;

        let acc_addr_hex_dec = hex::decode(req.account_addr.clone()).map_err(|_| {
            RpcError::parse_error("Failed to decode account address from hex string".to_string())
        })?;

        let acc_addr: [u8; 32] = acc_addr_hex_dec.try_into().map_err(|_| {
            RpcError::parse_error("Failed to parse account address from bytes".to_string())
        })?;

        let utxo_hash_hex_dec = hex::decode(req.utxo_hash.clone()).map_err(|_| {
            RpcError::parse_error("Failed to decode hash from hex string".to_string())
        })?;

        let utxo_hash: [u8; 32] = utxo_hash_hex_dec
            .try_into()
            .map_err(|_| RpcError::parse_error("Failed to parse hash from bytes".to_string()))?;

        let (asset, amount) = {
            let cover_guard = self.node_chain_store.lock().await;

            {
                let mut under_guard = cover_guard.storage.write().await;

                let acc = under_guard
                    .acc_map
                    .get_mut(&acc_addr)
                    .ok_or(RpcError::new_internal_error(None, "Account not found"))?;

                let utxo = acc
                    .utxo_tree
                    .get_item(utxo_hash)
                    .map_err(|err| {
                        RpcError::new_internal_error(None, &format!("DB fetch failure {err:?}"))
                    })?
                    .ok_or(RpcError::new_internal_error(
                        None,
                        "UTXO does not exist in tree",
                    ))?;

                (utxo.asset.clone(), utxo.amount)
            }
        };

        let helperstruct = ShowAccountUTXOResponse {
            hash: req.utxo_hash,
            asset,
            amount,
        };

        respond(helperstruct)
    }

    async fn process_show_transaction(&self, request: Request) -> Result<Value, RpcErr> {
        let req = ShowTransactionRequest::parse(Some(request.params))?;

        let tx_hash_hex_dec = hex::decode(req.tx_hash.clone()).map_err(|_| {
            RpcError::parse_error("Failed to decode hash from hex string".to_string())
        })?;

        let tx_hash: [u8; 32] = tx_hash_hex_dec
            .try_into()
            .map_err(|_| RpcError::parse_error("Failed to parse hash from bytes".to_string()))?;

        let helperstruct = {
            let cover_guard = self.node_chain_store.lock().await;

            {
                let under_guard = cover_guard.storage.read().await;

                let tx = under_guard
                    .pub_tx_store
                    .get_tx(tx_hash)
                    .ok_or(RpcError::new_internal_error(None, "Transactio not found"))?;

                ShowTransactionResponse {
                    hash: req.tx_hash,
                    tx_kind: tx.tx_kind,
                    public_input: if let Ok(action) =
                        serde_json::from_slice::<ActionData>(&tx.execution_input)
                    {
                        action.into_hexed_print()
                    } else {
                        "".to_string()
                    },
                    public_output: if let Ok(action) =
                        serde_json::from_slice::<ActionData>(&tx.execution_output)
                    {
                        action.into_hexed_print()
                    } else {
                        "".to_string()
                    },
                    utxo_commitments_created_hashes: tx
                        .utxo_commitments_created_hashes
                        .iter()
                        .map(|val| hex::encode(val.clone()))
                        .collect::<Vec<_>>(),
                    utxo_commitments_spent_hashes: tx
                        .utxo_commitments_spent_hashes
                        .iter()
                        .map(|val| hex::encode(val.clone()))
                        .collect::<Vec<_>>(),
                    utxo_nullifiers_created_hashes: tx
                        .nullifier_created_hashes
                        .iter()
                        .map(|val| hex::encode(val.clone()))
                        .collect::<Vec<_>>(),
                    encoded_data: tx
                        .encoded_data
                        .iter()
                        .map(|val| (hex::encode(val.0.clone()), hex::encode(val.1.clone())))
                        .collect::<Vec<_>>(),
                    ephemeral_pub_key: hex::encode(tx.ephemeral_pub_key.clone()),
                }
            }
        };

        respond(helperstruct)
    }

    pub async fn process_request_internal(&self, request: Request) -> Result<Value, RpcErr> {
        match request.method.as_ref() {
            //Todo : Add handling of more JSON RPC methods
            "register_account" => self.process_register_account(request).await,
            "execute_subscenario" => self.process_request_execute_subscenario(request).await,
            "send_tx" => self.process_send_tx(request).await,
            "get_block" => self.process_get_block_data(request).await,
            "get_last_block" => self.process_get_last_block(request).await,
            "execute_scenario_split" => self.process_request_execute_scenario_split(request).await,
            "execute_scenario_multiple_send" => {
                self.process_request_execute_scenario_multiple_send(request)
                    .await
            }
            "show_account_public_balance" => {
                self.process_show_account_public_balance(request).await
            }
            "show_account_utxo" => self.process_show_account_utxo_request(request).await,
            "show_trasnaction" => self.process_show_transaction(request).await,
            _ => Err(RpcErr(RpcError::method_not_found(request.method))),
        }
    }
}
