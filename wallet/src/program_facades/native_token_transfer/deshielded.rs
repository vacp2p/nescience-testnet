use common::{error::ExecutionFailureKind, sequencer_client::json::SendTxResponse};
use nssa::AccountId;

use super::{NativeTokenTransfer, auth_transfer_preparation};
use crate::PrivacyPreservingAccount;

impl NativeTokenTransfer<'_> {
    pub async fn send_deshielded_transfer(
        &self,
        from: AccountId,
        to: AccountId,
        balance_to_move: u128,
    ) -> Result<(SendTxResponse, nssa_core::SharedSecretKey), ExecutionFailureKind> {
        let (instruction_data, program, tx_pre_check) = auth_transfer_preparation(balance_to_move);

        self.0
            .send_privacy_preserving_tx(
                vec![
                    PrivacyPreservingAccount::PrivateOwned(from),
                    PrivacyPreservingAccount::Public(to),
                ],
                &instruction_data,
                tx_pre_check,
                &program,
            )
            .await
            .map(|(resp, secrets)| {
                let first = secrets
                    .into_iter()
                    .next()
                    .expect("expected sender's secret");
                (resp, first)
            })
    }
}
