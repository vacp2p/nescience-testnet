use serde::{Deserialize, Serialize};

use crate::{
    account::{
        Account, AccountWithMetadata, Commitment, Nonce, Nullifier, NullifierPublicKey,
        NullifierSecretKey,
    },
    program::{ProgramId, ProgramOutput},
};

pub mod account;
pub mod program;

pub type CommitmentSetDigest = [u32; 8];
pub type MembershipProof = Vec<[u8; 32]>;
pub fn verify_membership_proof(
    commitment: &Commitment,
    proof: &MembershipProof,
    digest: &CommitmentSetDigest,
) -> bool {
    todo!()
}

pub type IncomingViewingPublicKey = [u8; 32];
pub type EphemeralSecretKey = [u8; 32];
pub struct EphemeralPublicKey;

impl From<&EphemeralSecretKey> for EphemeralPublicKey {
    fn from(value: &EphemeralSecretKey) -> Self {
        todo!()
    }
}

pub struct Tag(u8);
impl Tag {
    pub fn new(Npk: &NullifierPublicKey, Ipk: &IncomingViewingPublicKey) -> Self {
        todo!()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct EncryptedAccountData;

impl EncryptedAccountData {
    pub fn new(
        account: &Account,
        esk: &EphemeralSecretKey,
        Npk: &NullifierPublicKey,
        Ivk: &IncomingViewingPublicKey,
    ) -> Self {
        // TODO: implement
        Self
    }
}

#[derive(Serialize, Deserialize)]
pub struct PrivacyPreservingCircuitInput {
    pub program_output: ProgramOutput,
    pub visibility_mask: Vec<u8>,
    pub private_account_data: Vec<(
        Nonce,
        NullifierPublicKey,
        IncomingViewingPublicKey,
        EphemeralSecretKey,
    )>,
    pub private_account_auth: Vec<(NullifierSecretKey, MembershipProof)>,
    pub program_id: ProgramId,
    pub commitment_set_digest: CommitmentSetDigest,
}

#[derive(Serialize, Deserialize)]
pub struct PrivacyPreservingCircuitOutput {
    pub public_pre_states: Vec<AccountWithMetadata>,
    pub public_post_states: Vec<Account>,
    pub encrypted_private_post_states: Vec<EncryptedAccountData>,
    pub new_commitments: Vec<Commitment>,
    pub new_nullifiers: Vec<Nullifier>,
    pub commitment_set_digest: CommitmentSetDigest,
}
