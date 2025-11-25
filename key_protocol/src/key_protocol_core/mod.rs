use std::collections::HashMap;

use anyhow::Result;
use k256::AffinePoint;
use serde::{Deserialize, Serialize};

use crate::key_management::KeyChain;

pub type PublicKey = AffinePoint;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NSSAUserData {
    ///Map for all user public accounts
    pub pub_account_signing_keys: HashMap<nssa::AccountId, nssa::PrivateKey>,
    ///Map for all user private accounts
    pub user_private_accounts: HashMap<nssa::AccountId, (KeyChain, nssa_core::account::Account)>,
}

impl NSSAUserData {
    fn valid_public_key_transaction_pairing_check(
        accounts_keys_map: &HashMap<nssa::AccountId, nssa::PrivateKey>,
    ) -> bool {
        let mut check_res = true;
        for (account_id, key) in accounts_keys_map {
            let expected_account_id =
                nssa::AccountId::from(&nssa::PublicKey::new_from_private_key(key));
            if &expected_account_id != account_id {
                println!("{}, {}", expected_account_id, account_id);
                check_res = false;
            }
        }
        check_res
    }

    fn valid_private_key_transaction_pairing_check(
        accounts_keys_map: &HashMap<nssa::AccountId, (KeyChain, nssa_core::account::Account)>,
    ) -> bool {
        let mut check_res = true;
        for (account_id, (key, _)) in accounts_keys_map {
            let expected_account_id = nssa::AccountId::from(&key.nullifer_public_key);
            if expected_account_id != *account_id {
                println!("{}, {}", expected_account_id, account_id);
                check_res = false;
            }
        }
        check_res
    }

    pub fn new_with_accounts(
        accounts_keys: HashMap<nssa::AccountId, nssa::PrivateKey>,
        accounts_key_chains: HashMap<nssa::AccountId, (KeyChain, nssa_core::account::Account)>,
    ) -> Result<Self> {
        if !Self::valid_public_key_transaction_pairing_check(&accounts_keys) {
            anyhow::bail!(
                "Key transaction pairing check not satisfied, there is account_ids, which is not derived from keys"
            );
        }

        if !Self::valid_private_key_transaction_pairing_check(&accounts_key_chains) {
            anyhow::bail!(
                "Key transaction pairing check not satisfied, there is account_ids, which is not derived from keys"
            );
        }

        Ok(Self {
            pub_account_signing_keys: accounts_keys,
            user_private_accounts: accounts_key_chains,
        })
    }

    /// Generated new private key for public transaction signatures
    ///
    /// Returns the account_id of new account
    pub fn generate_new_public_transaction_private_key(&mut self) -> nssa::AccountId {
        let private_key = nssa::PrivateKey::new_os_random();
        let account_id =
            nssa::AccountId::from(&nssa::PublicKey::new_from_private_key(&private_key));

        self.pub_account_signing_keys
            .insert(account_id, private_key);

        account_id
    }

    /// Returns the signing key for public transaction signatures
    pub fn get_pub_account_signing_key(
        &self,
        account_id: &nssa::AccountId,
    ) -> Option<&nssa::PrivateKey> {
        self.pub_account_signing_keys.get(account_id)
    }

    /// Generated new private key for privacy preserving transactions
    ///
    /// Returns the account_id of new account
    pub fn generate_new_privacy_preserving_transaction_key_chain(&mut self) -> nssa::AccountId {
        let key_chain = KeyChain::new_os_random();
        let account_id = nssa::AccountId::from(&key_chain.nullifer_public_key);

        self.user_private_accounts.insert(
            account_id,
            (key_chain, nssa_core::account::Account::default()),
        );

        account_id
    }

    /// Returns the signing key for public transaction signatures
    pub fn get_private_account(
        &self,
        account_id: &nssa::AccountId,
    ) -> Option<&(KeyChain, nssa_core::account::Account)> {
        self.user_private_accounts.get(account_id)
    }

    /// Returns the signing key for public transaction signatures
    pub fn get_private_account_mut(
        &mut self,
        account_id: &nssa::AccountId,
    ) -> Option<&mut (KeyChain, nssa_core::account::Account)> {
        self.user_private_accounts.get_mut(account_id)
    }
}

impl Default for NSSAUserData {
    fn default() -> Self {
        //Safe unwrap as maps are empty
        Self::new_with_accounts(HashMap::default(), HashMap::default()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_account() {
        let mut user_data = NSSAUserData::default();

        let addr_pub = user_data.generate_new_public_transaction_private_key();
        let addr_private = user_data.generate_new_privacy_preserving_transaction_key_chain();

        let is_private_key_generated = user_data.get_pub_account_signing_key(&addr_pub).is_some();

        assert!(is_private_key_generated);

        let is_key_chain_generated = user_data.get_private_account(&addr_private).is_some();

        assert!(is_key_chain_generated);

        let addr_private_str = addr_private.to_string();
        println!("{addr_private_str:#?}");
        let key_chain = &user_data.get_private_account(&addr_private).unwrap().0;
        println!("{key_chain:#?}");
    }
}
