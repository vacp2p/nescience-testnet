use crate::{
    address::Address, error::NssaError, program::Program, public_transaction::PublicTransaction,
};
use nssa_core::{account::Account, program::ProgramId};
use std::collections::HashMap;

pub struct V01State {
    public_state: HashMap<Address, Account>,
    builtin_programs: HashMap<ProgramId, Program>,
}

impl V01State {
    pub fn new_with_genesis_accounts(initial_data: &[([u8; 32], u128)]) -> Self {
        let authenticated_transfer_program = Program::authenticated_transfer_program();
        let public_state = initial_data
            .iter()
            .copied()
            .map(|(address_value, balance)| {
                let account = Account {
                    balance,
                    program_owner: authenticated_transfer_program.id(),
                    ..Account::default()
                };
                let address = Address::new(address_value);
                (address, account)
            })
            .collect();

        let mut this = Self {
            public_state,
            builtin_programs: HashMap::new(),
        };

        this.insert_program(Program::authenticated_transfer_program());

        this
    }

    fn insert_program(&mut self, program: Program) {
        self.builtin_programs.insert(program.id(), program);
    }

    pub fn transition_from_public_transaction(
        &mut self,
        tx: &PublicTransaction,
    ) -> Result<(), NssaError> {
        let state_diff = tx.validate_and_compute_post_states(self)?;

        for (address, post) in state_diff.into_iter() {
            let current_account = self.get_account_by_address_mut(address);
            *current_account = post;
        }

        for address in tx.signer_addresses() {
            let current_account = self.get_account_by_address_mut(address);
            current_account.nonce += 1;
        }

        Ok(())
    }

    fn get_account_by_address_mut(&mut self, address: Address) -> &mut Account {
        self.public_state.entry(address).or_default()
    }

    pub fn get_account_by_address(&self, address: &Address) -> Account {
        self.public_state
            .get(address)
            .cloned()
            .unwrap_or(Account::default())
    }

    pub(crate) fn builtin_programs(&self) -> &HashMap<ProgramId, Program> {
        &self.builtin_programs
    }
}

// Test utils
#[cfg(test)]
impl V01State {
    /// Include test programs in the builtin programs map
    pub fn with_test_programs(mut self) -> Self {
        self.insert_program(Program::nonce_changer_program());
        self.insert_program(Program::extra_output_program());
        self.insert_program(Program::missing_output_program());
        self.insert_program(Program::program_owner_changer());
        self.insert_program(Program::simple_balance_transfer());
        self.insert_program(Program::data_changer());
        self.insert_program(Program::minter());
        self.insert_program(Program::burner());
        self
    }

    pub fn with_non_default_accounts_but_default_program_owners(mut self) -> Self {
        let account_with_default_values_except_balance = Account {
            balance: 100,
            ..Account::default()
        };
        let account_with_default_values_except_nonce = Account {
            nonce: 37,
            ..Account::default()
        };
        let account_with_default_values_except_data = Account {
            data: vec![0xca, 0xfe],
            ..Account::default()
        };
        self.public_state.insert(
            Address::new([255; 32]),
            account_with_default_values_except_balance,
        );
        self.public_state.insert(
            Address::new([254; 32]),
            account_with_default_values_except_nonce,
        );
        self.public_state.insert(
            Address::new([253; 32]),
            account_with_default_values_except_data,
        );
        self
    }

    pub fn with_account_owned_by_burner_program(mut self) -> Self {
        let account = Account {
            program_owner: Program::burner().id(),
            balance: 100,
            ..Default::default()
        };
        self.public_state.insert(Address::new([252; 32]), account);
        self
    }
}
