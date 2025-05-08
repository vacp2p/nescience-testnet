//Current version of private state provability
//There is no document on private state provability
//So consider this module as a proposal to how we should handle it

use sha2::{digest::FixedOutput, Digest};

use super::private_state_storage::{PrivateDataBlob, PrivateSCState};

#[derive(thiserror::Error, Debug)]
pub enum PrivateDataProveError {
    #[error("Given blob id is too big: {0}, proofs len {1}")]
    BlobIdTooBig(usize, usize),
}

/// Hashing of all private state blobs
///
/// This sequence will be stored in public state
pub fn produce_private_state_hash(private_state: &PrivateSCState) -> Vec<[u8; 32]> {
    let mut pub_vec = vec![];

    for (_, blob) in private_state {
        let mut hasher = sha2::Sha256::new();

        hasher.update(blob);

        let hash = hasher.finalize_fixed();

        pub_vec.push(hash.into());
    }

    pub_vec
}

/// Checking, that subset of blobs is commited in public state
///
/// This function must be run at the start of every smart contract execution
pub fn check_private_state_subset(
    blob_seq: &[(usize, PrivateDataBlob)],
    public_proofs: &[[u8; 32]],
) -> Result<bool, PrivateDataProveError> {
    let proofs_len = public_proofs.len();
    let mut data_fit = true;

    for (idx, blob) in blob_seq {
        if *idx > proofs_len {
            return Err(PrivateDataProveError::BlobIdTooBig(*idx, proofs_len));
        }

        let mut hasher = sha2::Sha256::new();

        hasher.update(blob);

        let hash = <[u8; 32]>::from(hasher.finalize_fixed());

        if hash != public_proofs[*idx] {
            data_fit = false;
            break;
        }
    }

    Ok(data_fit)
}
