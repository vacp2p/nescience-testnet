use nssa_core::{
    account::{Account, AccountId, AccountWithMetadata, Data},
    program::{ProgramId, ProgramInput, ChainedCall, read_nssa_inputs, write_nssa_outputs, write_nssa_outputs_with_chained_call},
};

use bytemuck;

// The token program has two functions:
// 1. New token definition.
//    Arguments to this function are:
//      * Two **default** accounts: [definition_account, holding_account].
//        The first default account will be initialized with the token definition account values. The second account will
//        be initialized to a token holding account for the new token, holding the entire total supply.
//      * An instruction data of 23-bytes, indicating the total supply and the token name, with
//        the following layout:
//        [0x00 || total_supply (little-endian 16 bytes) || name (6 bytes)]
//        The name cannot be equal to [0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
// 2. Token transfer
//    Arguments to this function are:
//      * Two accounts: [sender_account, recipient_account].
//      * An instruction data byte string of length 23, indicating the total supply with the following layout
//        [0x01 || amount (little-endian 16 bytes) || 0x00 || 0x00 || 0x00 || 0x00 || 0x00 || 0x00].

const POOL_DEFINITION_DATA_SIZE: usize = 176;

struct PoolDefinition{
    definition_token_a_id: AccountId,
    definition_token_b_id: AccountId,
    vault_a_addr: AccountId,
    vault_b_addr: AccountId,
    liquidity_pool_id: AccountId,
    liquidity_pool_cap: u128,
    reserve_a: u128,
    reserve_b: u128,
    token_program_id: ProgramId,
}



impl PoolDefinition {
    fn into_data(self) -> Vec<u8> {
        let u8_token_program_id : [u8;32] = bytemuck::cast(self.token_program_id);

        let mut bytes = [0; POOL_DEFINITION_DATA_SIZE];
        bytes[0..32].copy_from_slice(&self.definition_token_a_id.to_bytes());
        bytes[32..64].copy_from_slice(&self.definition_token_b_id.to_bytes());
        bytes[64..96].copy_from_slice(&self.vault_a_addr.to_bytes());
        bytes[96..128].copy_from_slice(&self.vault_b_addr.to_bytes());
        bytes[128..160].copy_from_slice(&self.liquidity_pool_id.to_bytes());
        bytes[160..176].copy_from_slice(&self.liquidity_pool_cap.to_le_bytes());
        bytes[176..192].copy_from_slice(&self.reserve_a.to_le_bytes());
        bytes[192..208].copy_from_slice(&self.reserve_b.to_le_bytes());
        bytes[208..].copy_from_slice(&u8_token_program_id);
        bytes.into()
    }

    fn parse(data: &[u8]) -> Option<Self> {
        if data.len() != POOL_DEFINITION_DATA_SIZE {
            None
        } else {
            let definition_token_a_id = AccountId::new(data[0..32].try_into().unwrap());
            let definition_token_b_id = AccountId::new(data[32..64].try_into().unwrap());
            let vault_a_addr = AccountId::new(data[64..96].try_into().unwrap());
            let vault_b_addr = AccountId::new(data[96..128].try_into().unwrap());
            let liquidity_pool_id = AccountId::new(data[128..160].try_into().unwrap());
            let liquidity_pool_cap = u128::from_le_bytes(data[160..176].try_into().unwrap());
            let reserve_a = u128::from_le_bytes(data[176..].try_into().unwrap());
            let reserve_b = u128::from_le_bytes(data[192..208].try_into().unwrap());


            let token_program_id : &[u32] = bytemuck::cast_slice(&data[208..]);
            let token_program_id : ProgramId = token_program_id[0..8].try_into().unwrap();
            Some(Self {
                definition_token_a_id,
                definition_token_b_id,
                vault_a_addr,
                vault_b_addr,
                liquidity_pool_id,
                liquidity_pool_cap,
                reserve_a,
                reserve_b,
                token_program_id,
            })
        }
    }
}


//TODO: remove repeated code for Token_Definition and TokenHoldling
const TOKEN_DEFINITION_TYPE: u8 = 0;
const TOKEN_DEFINITION_DATA_SIZE: usize = 23;
const TOKEN_HOLDING_TYPE: u8 = 1;
const TOKEN_HOLDING_DATA_SIZE: usize = 49;

struct TokenHolding {
    account_type: u8,
    definition_id: AccountId,
    balance: u128,
}

impl TokenHolding {
    fn new(definition_id: &AccountId) -> Self {
        Self {
            account_type: TOKEN_HOLDING_TYPE,
            definition_id: definition_id.clone(),
            balance: 0,
        }
    }

    fn parse(data: &[u8]) -> Option<Self> {
        if data.len() != TOKEN_HOLDING_DATA_SIZE || data[0] != TOKEN_HOLDING_TYPE {
            None
        } else {
            let account_type = data[0];
            let definition_id = AccountId::new(data[1..33].try_into().unwrap());
            let balance = u128::from_le_bytes(data[33..].try_into().unwrap());
            Some(Self {
                definition_id,
                balance,
                account_type,
            })
        }
    }

    fn into_data(self) -> Data {
        let mut bytes = [0; TOKEN_HOLDING_DATA_SIZE];
        bytes[0] = self.account_type;
        bytes[1..33].copy_from_slice(&self.definition_id.to_bytes());
        bytes[33..].copy_from_slice(&self.balance.to_le_bytes());
        bytes.into()
    }
}


fn new_definition(
        pre_states: &[AccountWithMetadata],
        balance_in: &[u128],
        token_program: ProgramId,
    ) -> (Vec<Account>, Vec<ChainedCall>) {

    //Pool accounts: pool itself, and its 2 vaults and LP token
    //2 accounts for funding tokens
    //initial funder's LP account
    if pre_states.len() != 7 {
        panic!("Invalid number of input account")
    }

    if balance_in.len() != 2 {
        panic!("Invalid number of balance")
    }

    let pool = &pre_states[0];
    let vault_a = &pre_states[1];
    let vault_b = &pre_states[2];
    let pool_lp = &pre_states[3];
    let user_a = &pre_states[4];
    let user_b = &pre_states[5];
    let user_lp = &pre_states[6];

    if pool.account != Account::default() || !pool.is_authorized {
        panic!("TODO-1");
    }

    // TODO: temporary band-aid to prevent vault's from being
    // owned by the amm program.
    if vault_a.account == Account::default() || vault_b.account == Account::default() {
        panic!("Vault accounts must be initialized first; issue to be fixed")
    }
    if pool_lp.account == Account::default() {
        panic!("Pool LP must be initialized first; issue to be fixed")
    }

    let amount_a = balance_in[0];
    let amount_b = balance_in[1];

    // Prevents pool constant coefficient (k) from being 0.
    assert!(amount_a > 0);
    assert!(amount_b > 0);

    // Verify token_a and token_b are different
    let definition_token_a_id = TokenHolding::parse(&vault_a.account.data).unwrap().definition_id;
    let definition_token_b_id = TokenHolding::parse(&vault_b.account.data).unwrap().definition_id;
    let user1_id = TokenHolding::parse(&vault_a.account.data).unwrap().definition_id;

    if definition_token_a_id == definition_token_b_id {
        panic!("Vaults are for the same token")
    }

    // 5. Update pool account
    let mut pool_post = Account::default();
    let pool_post_definition = PoolDefinition {
            definition_token_a_id,
            definition_token_b_id,
            vault_a_addr: vault_a.account_id.clone(),
            vault_b_addr: vault_b.account_id.clone(),
            liquidity_pool_id: pool_lp.account_id.clone(),
            liquidity_pool_cap: amount_a,
            reserve_a: amount_a,
            reserve_b: amount_b,
            token_program_id: token_program,  
    };

    pool_post.data = pool_post_definition.into_data();

    let mut chained_call = Vec::new();

    let mut instruction_data = [0; 23];
    instruction_data[0] = 1;

    instruction_data[1..17].copy_from_slice(&amount_a.to_le_bytes());
    let call_token_a = ChainedCall{
            program_id: token_program,
            instruction_data: bytemuck::cast_slice(&instruction_data).to_vec(),
            pre_states: vec![user_a.clone(), vault_a.clone()]
        };


    instruction_data[1..17].copy_from_slice(&amount_b.to_le_bytes());
    let call_token_b = ChainedCall{
            program_id: token_program,
            instruction_data: bytemuck::cast_slice(&instruction_data).to_vec(),
            pre_states: vec![user_b.clone(), vault_b.clone()]
        };

    instruction_data[1..17].copy_from_slice(&amount_a.to_le_bytes());
    let call_token_lp = ChainedCall{
            program_id: token_program,
            instruction_data: bytemuck::cast_slice(&instruction_data).to_vec(),
            pre_states: vec![pool_lp.clone(), user_lp.clone()]
        };

    chained_call.push(call_token_lp);
    chained_call.push(call_token_b);
    chained_call.push(call_token_a);


    let post_states = vec![pool_post.clone(), 
        pre_states[1].account.clone(),
        pre_states[2].account.clone(),
        pre_states[3].account.clone(),
        pre_states[4].account.clone(),
        pre_states[5].account.clone(),
        pre_states[6].account.clone()];

    (post_states.clone(), chained_call)
}


type Instruction = Vec<u8>;
fn main() {
    let ProgramInput {
        pre_states,
        instruction,
    } = read_nssa_inputs::<Instruction>();

    match instruction[0] {
        0 => {
            let balance_a: u128 = u128::from_le_bytes(instruction[1..17].try_into().unwrap());
            let balance_b: u128 = u128::from_le_bytes(instruction[17..33].try_into().unwrap());
        

            let token_program_id : &[u32] = bytemuck::cast_slice(&instruction[33..55]);
            let token_program_id : [u32;8] = token_program_id.try_into().unwrap();

            let (post_states, chained_call) = new_definition(&pre_states,
                &[balance_a, balance_b],
                token_program_id
                );

            write_nssa_outputs_with_chained_call(pre_states, post_states, chained_call);
        }
        1 => {
            let intent = SwapIntent {
                token_id: AccountId::new(instruction[1..33].try_into().unwrap()),
                amount: u128::from_le_bytes(instruction[33..49].try_into().unwrap()),
            };

            let (post_states, chained_call) = swap(&pre_states, &intent);

            write_nssa_outputs_with_chained_call(pre_states, post_states, chained_call);
        }
        2 => {
            let (post_states, chained_call) = add_liquidity(&pre_states,
                        &[u128::from_le_bytes(instruction[1..17].try_into().unwrap()),
                            u128::from_le_bytes(instruction[16..33].try_into().unwrap()),],
                        AccountId::new(instruction[33..65].try_into().unwrap()));
           write_nssa_outputs_with_chained_call(pre_states, post_states, chained_call);
        }
        3 => {

            let (post_states, chained_call) = remove_liquidity(&pre_states);

            write_nssa_outputs_with_chained_call(pre_states, post_states, chained_call);
        }
        _ => panic!("Invalid instruction"),
    };
}

struct SwapIntent {
    token_id: AccountId,
    amount: u128,
}

fn swap(
        pre_states: &[AccountWithMetadata],
        intent: &SwapIntent,
    ) -> (Vec<Account>, Vec<ChainedCall>) {

    if pre_states.len() != 5 {
        panic!("Invalid number of input accounts");
    }

    let pool = &pre_states[0];
    let vault1 = &pre_states[1];
    let vault2 = &pre_states[2];
    let user_a = &pre_states[3];
    let user_b = &pre_states[4];

    // Verify vaults are in fact vaults
    let pool_def_data = PoolDefinition::parse(&pool.account.data).unwrap();

    let mut vault_a = AccountWithMetadata::default();
    let mut vault_b = AccountWithMetadata::default();

    if vault1.account_id == pool_def_data.definition_token_a_id {
            vault_a = vault1.clone();
        } else if vault2.account_id == pool_def_data.definition_token_a_id {
            vault_a = vault2.clone();
        } else {
            panic!("Vault A was no provided");
        }
        
    if vault1.account_id == pool_def_data.definition_token_b_id {
       vault_b = vault1.clone();
    } else if vault2.account_id == pool_def_data.definition_token_b_id {
        vault_b = vault2.clone();
    } else {
        panic!("Vault B was no provided");
    }

    // 1. Identify swap direction (a -> b or b -> a)
    let mut deposit_a = 0;
    let mut deposit_b = 0;
    let a_to_b;
    if intent.token_id == pool_def_data.definition_token_a_id {
        deposit_a = intent.amount;
        a_to_b = true;
    } else if intent.token_id == pool_def_data.definition_token_b_id {
        deposit_b = intent.amount;
        a_to_b = false;
    } else {
        panic!("Intent address is not a token type for the pool");
    }

    // 2. fetch pool reserves
    //validates reserves is at least the vaults' balances
    assert!(vault_a.account.balance >= pool_def_data.reserve_a);
    assert!(vault_b.account.balance >= pool_def_data.reserve_b);
    //Cannot swap if a reserve is 0
    assert!(pool_def_data.reserve_a > 0);
    assert!(pool_def_data.reserve_b > 0);

    // 3. Compute output amount
    // Note: no fees
    // Compute pool's exchange constant
    // let k = pool_def_data.reserve_a * pool_def_data.reserve_b;
    let withdraw_a = if a_to_b { 0 }
            else { (pool_def_data.reserve_a * deposit_b)/(pool_def_data.reserve_b + deposit_b) };   
    let withdraw_b = if a_to_b { (pool_def_data.reserve_b * deposit_a)/(pool_def_data.reserve_a + deposit_a)}
                    else { 0 };                 

    // 4. Slippage check
    if a_to_b {
        assert!(withdraw_b == 0); }
    else{
        assert!(withdraw_a == 0); }

    // 5. Update pool account
    let mut pool_post = pool.account.clone();
    let pool_post_definition = PoolDefinition {
            definition_token_a_id: pool_def_data.definition_token_a_id.clone(),
            definition_token_b_id: pool_def_data.definition_token_b_id.clone(),
            vault_a_addr: pool_def_data.vault_a_addr.clone(),
            vault_b_addr: pool_def_data.vault_b_addr.clone(),
            liquidity_pool_id: pool_def_data.liquidity_pool_id.clone(),
            liquidity_pool_cap: pool_def_data.liquidity_pool_cap.clone(),
            reserve_a: pool_def_data.reserve_a + deposit_a - withdraw_a,
            reserve_b: pool_def_data.reserve_b + deposit_b - withdraw_b,
            token_program_id: pool_def_data.token_program_id.clone(),  
    };

    pool_post.data = pool_post_definition.into_data();

    let mut chained_call = Vec::new();

    let mut instruction_data = [0; 23];
    instruction_data[0] = 1;

    let call_token_a = if a_to_b {
        instruction_data[1..17].copy_from_slice(&deposit_a.to_le_bytes());
        ChainedCall{
            program_id: pool_def_data.token_program_id.clone(),
            instruction_data: bytemuck::cast_slice(&instruction_data).to_vec(),
            pre_states: vec![user_a.clone(), vault_a.clone()]
        }
    } else {
        instruction_data[1..17].copy_from_slice(&withdraw_a.to_le_bytes());
        ChainedCall{
            program_id: pool_def_data.token_program_id.clone(),
            instruction_data: bytemuck::cast_slice(&instruction_data).to_vec(),
            pre_states: vec![vault_a.clone(), user_a.clone()]
        }
    };

    let call_token_b = if a_to_b {
        instruction_data[1..17].copy_from_slice(&deposit_b.to_le_bytes());
        ChainedCall{
            program_id: pool_def_data.token_program_id.clone(),
            instruction_data: bytemuck::cast_slice(&instruction_data).to_vec(),
            pre_states: vec![user_b.clone(), vault_b.clone()]
        }
    } else {
        instruction_data[1..17].copy_from_slice(&withdraw_b.to_le_bytes());
        ChainedCall{
            program_id: pool_def_data.token_program_id.clone(),
            instruction_data: bytemuck::cast_slice(&instruction_data).to_vec(),
            pre_states: vec![vault_b.clone(), user_b.clone()]
        }
    };

    chained_call.push(call_token_a);
    chained_call.push(call_token_b);

    let post_states = vec![pool_post.clone(), 
        pre_states[1].account.clone(),
        pre_states[2].account.clone(),
        pre_states[3].account.clone(),
        pre_states[4].account.clone()];

    (post_states.clone(), chained_call)
}


fn add_liquidity(pre_states: &[AccountWithMetadata],
    max_balance_in: &[u128],
    main_token: AccountId) -> (Vec<Account>, Vec<ChainedCall>) {

    if pre_states.len() != 7 {
       panic!("Invalid number of input accounts");
    }

    let pool = &pre_states[0];
    let vault1 = &pre_states[1];
    let vault2 = &pre_states[2];
    let pool_lp = &pre_states[3];
    let user_a = &pre_states[4];
    let user_b = &pre_states[5];
    let user_lp = &pre_states[6];

    let mut vault_a = AccountWithMetadata::default();
    let mut vault_b = AccountWithMetadata::default();
    
    let pool_def_data = PoolDefinition::parse(&pool.account.data).unwrap();

    if vault1.account_id == pool_def_data.definition_token_a_id {
            vault_a = vault1.clone();
        } else if vault2.account_id == pool_def_data.definition_token_a_id {
            vault_a = vault2.clone();
        } else {
            panic!("Vault A was no provided");
        }
        
    if vault1.account_id == pool_def_data.definition_token_b_id {
       vault_b = vault1.clone();
    } else if vault2.account_id == pool_def_data.definition_token_b_id {
        vault_b = vault2.clone();
    } else {
        panic!("Vault B was no provided");
    }


    if max_balance_in.len() != 2 {
        panic!("Invalid number of input balances");
    }

    let max_amount_a = max_balance_in[0];
    let max_amount_b = max_balance_in[1];

    // 2. Determine deposit amounts
    let mut actual_amount_a = 0;
    let mut actual_amount_b = 0;

    if main_token == pool_def_data.definition_token_a_id {
        actual_amount_a = max_amount_a;
        actual_amount_b = (vault_b.account.balance/vault_a.account.balance)*actual_amount_a;
    } else if main_token == pool_def_data.definition_token_b_id {
        actual_amount_b = max_amount_b;
        actual_amount_a = (vault_a.account.balance/vault_b.account.balance)*actual_amount_b;
    } else {
        panic!("Mismatch of token types"); //main token does not match with vaults.
    }

    
    // 3. Validate amounts
    assert!(user_a.account.balance >= actual_amount_a && actual_amount_a > 0);
    assert!(user_b.account.balance >= actual_amount_b && actual_amount_b > 0);

    // 4. Calculate LP to mint
    let delta_lp : u128 = pool_def_data.liquidity_pool_cap * (actual_amount_b/pool_def_data.reserve_b);

    // 5. Update pool account
    let mut pool_post = pool.account.clone();
    let pool_post_definition = PoolDefinition {
            definition_token_a_id: pool_def_data.definition_token_a_id.clone(),
            definition_token_b_id: pool_def_data.definition_token_b_id.clone(),
            vault_a_addr: pool_def_data.vault_a_addr.clone(),
            vault_b_addr: pool_def_data.vault_b_addr.clone(),
            liquidity_pool_id: pool_def_data.liquidity_pool_id.clone(),
            liquidity_pool_cap: pool_def_data.liquidity_pool_cap + delta_lp,
            reserve_a: pool_def_data.reserve_a + actual_amount_a,
            reserve_b: pool_def_data.reserve_b + actual_amount_b,
            token_program_id: pool_def_data.token_program_id.clone(),  
    };

    pool_post.data = pool_post_definition.into_data();

    let mut chained_call = Vec::new();

    let mut instruction_data = [0; 23];
    instruction_data[0] = 1;

    instruction_data[1..17].copy_from_slice(&actual_amount_a.to_le_bytes());
    let call_token_a = ChainedCall{
            program_id: pool_def_data.token_program_id.clone(),
            instruction_data: bytemuck::cast_slice(&instruction_data).to_vec(),
            pre_states: vec![user_a.clone(), vault_a]
        };


    instruction_data[1..17].copy_from_slice(&actual_amount_b.to_le_bytes());
    let call_token_b = ChainedCall{
            program_id: pool_def_data.token_program_id.clone(),
            instruction_data: bytemuck::cast_slice(&instruction_data).to_vec(),
            pre_states: vec![user_b.clone(), vault_b]
        };

    instruction_data[1..17].copy_from_slice(&delta_lp.to_le_bytes());
    let call_token_lp = ChainedCall{
            program_id: pool_def_data.token_program_id.clone(),
            instruction_data: bytemuck::cast_slice(&instruction_data).to_vec(),
            pre_states: vec![pool_lp.clone(), user_lp.clone()]
        };

    chained_call.push(call_token_lp);
    chained_call.push(call_token_b);
    chained_call.push(call_token_a);


    let post_states = vec![pool_post.clone(), 
        pre_states[1].account.clone(),
        pre_states[2].account.clone(),
        pre_states[3].account.clone(),
        pre_states[4].account.clone(),
        pre_states[5].account.clone(),
        pre_states[6].account.clone(),];

    (post_states.clone(), chained_call)

}


fn remove_liquidity(pre_states: &[AccountWithMetadata]) -> (Vec<Account>, Vec<ChainedCall>) {

    if pre_states.len() != 7 {
       panic!("Invalid number of input accounts");
    }

    let pool = &pre_states[0];
    let vault1 = &pre_states[1];
    let vault2 = &pre_states[2];
    let pool_lp = &pre_states[3];
    let user_a = &pre_states[4];
    let user_b = &pre_states[5];
    let user_lp = &pre_states[6];

    let mut vault_a = AccountWithMetadata::default();
    let mut vault_b = AccountWithMetadata::default();
    
    let pool_def_data = PoolDefinition::parse(&pool.account.data).unwrap();

    if vault1.account_id == pool_def_data.definition_token_a_id {
            vault_a = vault1.clone();
        } else if vault2.account_id == pool_def_data.definition_token_a_id {
            vault_a = vault2.clone();
        } else {
            panic!("Vault A was no provided");
        }
        
    if vault1.account_id == pool_def_data.definition_token_b_id {
       vault_b = vault1.clone();
    } else if vault2.account_id == pool_def_data.definition_token_b_id {
        vault_b = vault2.clone();
    } else {
        panic!("Vault B was no provided");
    }

    // 2. Determine deposit amounts
    let withdraw_amount_a = pool_def_data.reserve_a * (user_lp.account.balance/pool_def_data.liquidity_pool_cap);
    let withdraw_amount_b = pool_def_data.reserve_b * (user_lp.account.balance/pool_def_data.liquidity_pool_cap);

    //3. Validate amounts handled by token programs

    // 4. Calculate LP to reduce cap by
    let delta_lp : u128 = (pool_def_data.liquidity_pool_cap*pool_def_data.liquidity_pool_cap - user_lp.account.balance)/pool_def_data.liquidity_pool_cap;

    // 5. Update pool account
    let mut pool_post = pool.account.clone();
    let pool_post_definition = PoolDefinition {
            definition_token_a_id: pool_def_data.definition_token_a_id.clone(),
            definition_token_b_id: pool_def_data.definition_token_b_id.clone(),
            vault_a_addr: pool_def_data.vault_a_addr.clone(),
            vault_b_addr: pool_def_data.vault_b_addr.clone(),
            liquidity_pool_id: pool_def_data.liquidity_pool_id.clone(),
            liquidity_pool_cap: pool_def_data.liquidity_pool_cap - delta_lp,
            reserve_a: pool_def_data.reserve_a - withdraw_amount_a,
            reserve_b: pool_def_data.reserve_b - withdraw_amount_b,
            token_program_id: pool_def_data.token_program_id.clone(),  
    };

    pool_post.data = pool_post_definition.into_data();

    let mut chained_call = Vec::new();

    let mut instruction_data = [0; 23];
    instruction_data[0] = 1;

    instruction_data[1..17].copy_from_slice(&withdraw_amount_a.to_le_bytes());
    let call_token_a = ChainedCall{
            program_id: pool_def_data.token_program_id.clone(),
            instruction_data: bytemuck::cast_slice(&instruction_data).to_vec(),
            pre_states: vec![vault_a, user_a.clone()]
        };


    instruction_data[1..17].copy_from_slice(&withdraw_amount_b.to_le_bytes());
    let call_token_b = ChainedCall{
            program_id: pool_def_data.token_program_id.clone(),
            instruction_data: bytemuck::cast_slice(&instruction_data).to_vec(),
            pre_states: vec![vault_b, user_b.clone()]
        };

    instruction_data[1..17].copy_from_slice(&delta_lp.to_le_bytes());
    let call_token_lp = ChainedCall{
            program_id: pool_def_data.token_program_id.clone(),
            instruction_data: bytemuck::cast_slice(&instruction_data).to_vec(),
            pre_states: vec![user_lp.clone(), pool_lp.clone()]
        };

    chained_call.push(call_token_lp);
    chained_call.push(call_token_b);
    chained_call.push(call_token_a);


    let post_states = vec![pool_post.clone(), 
        pre_states[1].account.clone(),
        pre_states[2].account.clone(),
        pre_states[3].account.clone(),
        pre_states[4].account.clone(),
        pre_states[5].account.clone(),
        pre_states[6].account.clone()];

    (post_states.clone(), chained_call)

}

