use accounts::account_core::{Account, AccountAddress};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AccountPublicData {
    pub balance: u64,
    pub address: AccountAddress,
}

impl AccountPublicData {
    pub fn new(address: AccountAddress) -> Self {
        Self {
            balance: 0,
            address,
        }
    }

    fn new_with_balance(address: AccountAddress, balance: u64) -> Self {
        Self { balance, address }
    }
}

#[derive(Debug, Clone)]
pub struct SequencerAccountsStore {
    accounts: HashMap<AccountAddress, AccountPublicData>,
}

impl SequencerAccountsStore {
    pub fn new(initial_accounts: &[Account]) -> Self {
        let mut accounts = HashMap::new();

        for account in initial_accounts {
            accounts.insert(
                account.address,
                AccountPublicData::new_with_balance(account.address, account.balance),
            );
        }

        Self { accounts }
    }

    ///Register new account in accounts store
    ///
    ///Starts with zero public balance
    pub fn register_account(&mut self, account_addr: AccountAddress) {
        self.accounts
            .insert(account_addr, AccountPublicData::new(account_addr));
    }

    ///Check, if `account_addr` present in account store
    pub fn contains_account(&self, account_addr: &AccountAddress) -> bool {
        self.accounts.contains_key(account_addr)
    }

    ///Check `account_addr` balance,
    ///
    ///returns 0, if account address not found
    pub fn get_account_balance(&self, account_addr: &AccountAddress) -> u64 {
        self.accounts
            .get(account_addr)
            .map(|acc| acc.balance)
            .unwrap_or(0)
    }

    ///Update `account_addr` balance,
    ///
    /// returns 0, if account address not found, othervise returns previous balance
    pub fn set_account_balance(&mut self, account_addr: &AccountAddress, new_balance: u64) -> u64 {
        let acc_data = self.accounts.get_mut(account_addr);

        acc_data
            .map(|data| {
                let old_bal = data.balance;

                data.balance = new_balance;

                old_bal
            })
            .unwrap_or(0)
    }

    ///Remove account from storage
    ///
    /// Fails, if `balance` is != 0
    ///
    /// Returns `Option<AccountAddress>` which is `None` if `account_addr` vere not present in store
    pub fn unregister_account(
        &mut self,
        account_addr: AccountAddress,
    ) -> Result<Option<AccountAddress>> {
        if self.get_account_balance(&account_addr) == 0 {
            Ok(self.accounts.remove(&account_addr).map(|data| data.address))
        } else {
            anyhow::bail!("Chain consistency violation: It is forbidden to remove account with nonzero balance");
        }
    }

    ///Number of accounts present in store
    pub fn len(&self) -> usize {
        self.accounts.len()
    }
}

impl Default for SequencerAccountsStore {
    fn default() -> Self {
        Self::new(&[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_balance_account_data_creation() {
        let new_acc = AccountPublicData::new([1; 32]);

        assert_eq!(new_acc.balance, 0);
        assert_eq!(new_acc.address, [1; 32]);
    }

    #[test]
    fn test_non_zero_balance_account_data_creation() {
        let new_acc = AccountPublicData::new_with_balance([1; 32], 10);

        assert_eq!(new_acc.balance, 10);
        assert_eq!(new_acc.address, [1; 32]);
    }

    #[test]
    fn default_account_sequencer_store() {
        let seq_acc_store = SequencerAccountsStore::default();

        assert!(seq_acc_store.accounts.is_empty());
    }

    #[test]
    fn account_sequencer_store_register_acc() {
        let mut seq_acc_store = SequencerAccountsStore::default();

        seq_acc_store.register_account([1; 32]);

        assert!(seq_acc_store.contains_account(&[1; 32]));

        let acc_balance = seq_acc_store.get_account_balance(&[1; 32]);

        assert_eq!(acc_balance, 0);
    }

    #[test]
    fn account_sequencer_store_unregister_acc_not_present() {
        let mut seq_acc_store = SequencerAccountsStore::default();

        seq_acc_store.register_account([1; 32]);

        let rem_res = seq_acc_store.unregister_account([2; 32]).unwrap();

        assert!(rem_res.is_none());
    }

    #[test]
    fn account_sequencer_store_unregister_acc_not_zero_balance() {
        let acc1 = Account::new_with_balance(12);
        let acc2 = Account::new_with_balance(100);

        let acc1_addr = acc1.address.clone();

        let mut seq_acc_store = SequencerAccountsStore::new(&[acc1, acc2]);

        let rem_res = seq_acc_store.unregister_account(acc1_addr);

        assert!(rem_res.is_err());
    }

    #[test]
    fn account_sequencer_store_unregister_acc() {
        let mut seq_acc_store = SequencerAccountsStore::default();

        seq_acc_store.register_account([1; 32]);

        assert!(seq_acc_store.contains_account(&[1; 32]));

        seq_acc_store.unregister_account([1; 32]).unwrap().unwrap();

        assert!(!seq_acc_store.contains_account(&[1; 32]));
    }

    #[test]
    fn account_sequencer_store_with_preset_accounts_1() {
        let acc1 = Account::new_with_balance(12);
        let acc2 = Account::new_with_balance(100);

        let acc1_addr = acc1.address.clone();
        let acc2_addr = acc2.address.clone();

        let seq_acc_store = SequencerAccountsStore::new(&[acc1, acc2]);

        assert!(seq_acc_store.contains_account(&acc1_addr));
        assert!(seq_acc_store.contains_account(&acc2_addr));

        let acc_balance = seq_acc_store.get_account_balance(&acc1_addr);

        assert_eq!(acc_balance, 12);

        let acc_balance = seq_acc_store.get_account_balance(&acc2_addr);

        assert_eq!(acc_balance, 100);
    }

    #[test]
    fn account_sequencer_store_with_preset_accounts_2() {
        let acc1 = Account::new_with_balance(120);
        let acc2 = Account::new_with_balance(15);
        let acc3 = Account::new_with_balance(10);

        let acc1_addr = acc1.address.clone();
        let acc2_addr = acc2.address.clone();
        let acc3_addr = acc3.address.clone();

        let seq_acc_store = SequencerAccountsStore::new(&[acc1, acc2, acc3]);

        assert!(seq_acc_store.contains_account(&acc1_addr));
        assert!(seq_acc_store.contains_account(&acc2_addr));
        assert!(seq_acc_store.contains_account(&acc3_addr));

        let acc_balance = seq_acc_store.get_account_balance(&[6; 32]);

        assert_eq!(acc_balance, 120);

        let acc_balance = seq_acc_store.get_account_balance(&[7; 32]);

        assert_eq!(acc_balance, 15);

        let acc_balance = seq_acc_store.get_account_balance(&[8; 32]);

        assert_eq!(acc_balance, 10);
    }

    #[test]
    fn account_sequencer_store_fetch_unknown_account() {
        let acc1 = Account::new_with_balance(120);
        let acc2 = Account::new_with_balance(15);
        let acc3 = Account::new_with_balance(10);

        let seq_acc_store = SequencerAccountsStore::new(&[acc1, acc2, acc3]);

        let acc_balance = seq_acc_store.get_account_balance(&[9; 32]);

        assert_eq!(acc_balance, 0);
    }
}
