use std::vec;

use common::{error::ExecutionFailureKind, sequencer_client::json::SendTxResponse};
use nssa::AccountId;
use nssa_core::{NullifierPublicKey, SharedSecretKey, encryption::IncomingViewingPublicKey};

use super::{NativeTokenTransfer, auth_transfer_preparation};
use crate::PrivacyPreservingAccount;

impl NativeTokenTransfer<'_> {
    pub async fn send_private_transfer_to_outer_account(
        &self,
        from: AccountId,
        to_npk: NullifierPublicKey,
        to_ipk: IncomingViewingPublicKey,
        balance_to_move: u128,
    ) -> Result<(SendTxResponse, [SharedSecretKey; 2]), ExecutionFailureKind> {
        let (instruction_data, program, tx_pre_check) = auth_transfer_preparation(balance_to_move);

        self.0
            .send_privacy_preserving_tx(
                vec![
                    PrivacyPreservingAccount::PrivateOwned(from),
                    PrivacyPreservingAccount::PrivateForeign {
                        npk: to_npk,
                        ipk: to_ipk,
                    },
                ],
                &instruction_data,
                tx_pre_check,
                &program,
            )
            .await
            .map(|(resp, secrets)| {
                let mut secrets_iter = secrets.into_iter();
                let first = secrets_iter.next().expect("expected sender's secret");
                let second = secrets_iter.next().expect("expected receiver's secret");
                (resp, [first, second])
            })
    }

    pub async fn send_private_transfer_to_owned_account(
        &self,
        from: AccountId,
        to: AccountId,
        balance_to_move: u128,
    ) -> Result<(SendTxResponse, [SharedSecretKey; 2]), ExecutionFailureKind> {
        let (instruction_data, program, tx_pre_check) = auth_transfer_preparation(balance_to_move);

        self.0
            .send_privacy_preserving_tx(
                vec![
                    PrivacyPreservingAccount::PrivateOwned(from),
                    PrivacyPreservingAccount::PrivateOwned(to),
                ],
                &instruction_data,
                tx_pre_check,
                &program,
            )
            .await
            .map(|(resp, secrets)| {
                let mut secrets_iter = secrets.into_iter();
                let first = secrets_iter.next().expect("expected sender's secret");
                let second = secrets_iter.next().expect("expected receiver's secret");
                (resp, [first, second])
            })
    }
}
