use nssa_core::program::{AccountPostState, ProgramInput, read_nssa_inputs, write_nssa_outputs};

type Instruction = ();

fn main() {
    let ProgramInput { pre_states, .. } = read_nssa_inputs::<Instruction>();

    let [pre] = match pre_states.try_into() {
        Ok(array) => array,
        Err(_) => return,
    };

    let account_pre = &pre.account;
    let mut account_post = account_pre.clone();
    let mut data_vec = account_post.data.into_inner();
    data_vec.push(0);
    account_post.data = data_vec.try_into().expect("data_vec should fit into Data");

    write_nssa_outputs(vec![pre], vec![AccountPostState::new_claimed(account_post)]);
}
