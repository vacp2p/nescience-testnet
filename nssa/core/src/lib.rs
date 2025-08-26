use chacha20::{
    ChaCha20,
    cipher::{KeyIvInit, StreamCipher},
};
use risc0_zkvm::{
    serde::to_vec,
    sha::{Impl, Sha256},
};
use serde::{Deserialize, Serialize};

#[cfg(feature = "host")]
use crate::error::NssaCoreError;

use crate::{
    account::{
        Account, AccountWithMetadata, Commitment, Nonce, Nullifier, NullifierPublicKey,
        NullifierSecretKey,
    },
    program::{ProgramId, ProgramOutput},
};

#[cfg(feature = "host")]
use std::io::{Cursor, Read};

pub mod account;
pub mod program;

use k256::{
    AffinePoint, EncodedPoint, FieldBytes, ProjectivePoint, PublicKey, Scalar,
    elliptic_curve::{
        PrimeField,
        sec1::{FromEncodedPoint, ToEncodedPoint},
    },
};

#[cfg(feature = "host")]
pub mod error;

pub type CommitmentSetDigest = [u8; 32];
pub type MembershipProof = (usize, Vec<[u8; 32]>);
pub fn compute_root_associated_to_path(
    commitment: &Commitment,
    proof: &MembershipProof,
) -> CommitmentSetDigest {
    let value_bytes = commitment.to_byte_array();
    let mut result: [u8; 32] = Impl::hash_bytes(&value_bytes)
        .as_bytes()
        .try_into()
        .unwrap();
    let mut level_index = proof.0;
    for node in &proof.1 {
        let is_left_child = level_index & 1 == 0;
        if is_left_child {
            let mut bytes = [0u8; 64];
            bytes[..32].copy_from_slice(&result);
            bytes[32..].copy_from_slice(node);
            result = Impl::hash_bytes(&bytes).as_bytes().try_into().unwrap();
        } else {
            let mut bytes = [0u8; 64];
            bytes[..32].copy_from_slice(node);
            bytes[32..].copy_from_slice(&result);
            result = Impl::hash_bytes(&bytes).as_bytes().try_into().unwrap();
        }
        level_index >>= 1;
    }
    result
}

pub type EphemeralPublicKey = Secp256k1Point;
pub type IncomingViewingPublicKey = Secp256k1Point;

pub type EphemeralSecretKey = [u8; 32];

impl From<&EphemeralSecretKey> for EphemeralPublicKey {
    fn from(value: &EphemeralSecretKey) -> Self {
        Secp256k1Point::from_scalar(*value)
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(any(feature = "host", test), derive(Debug, PartialEq, Eq))]
pub struct Secp256k1Point(pub Vec<u8>);
impl Secp256k1Point {
    pub fn from_scalar(value: [u8; 32]) -> Secp256k1Point {
        let x_bytes: FieldBytes = value.into();
        let x = Scalar::from_repr(x_bytes).unwrap();

        let p = ProjectivePoint::GENERATOR * x;
        let q = AffinePoint::from(p);
        let enc = q.to_encoded_point(true);

        Self(enc.as_bytes().to_vec())
    }
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(any(feature = "host", test), derive(Debug, Clone, PartialEq, Eq))]
pub struct EncryptedAccountData {
    ciphertext: Vec<u8>,
    epk: EphemeralPublicKey,
    view_tag: u8,
}

impl EncryptedAccountData {
    #[cfg(feature = "host")]
    pub fn decrypt(self, isk: &[u8; 32], output_index: u32) -> Option<Account> {
        let ss_bytes = Self::ecdh(isk, &self.epk.0.clone().try_into().unwrap());
        let ipk = IncomingViewingPublicKey::from_scalar(*isk);

        let key = Self::kdf(
            ss_bytes,
            &self.epk,
            &ipk,
            // &commitment.to_byte_array(),
            output_index,
        );
        let mut cipher = ChaCha20::new(&key.into(), &[0; 12].into());
        let mut buffer = self.ciphertext;

        cipher.apply_keystream(&mut buffer);
        let mut cursor = Cursor::new(buffer.as_slice());
        Account::from_cursor(&mut cursor).ok()
    }

    pub fn new(
        account: &Account,
        // commitment: &Commitment,
        esk: &EphemeralSecretKey,
        npk: &NullifierPublicKey,
        ipk: &IncomingViewingPublicKey,
        output_index: u32,
    ) -> Self {
        let mut buffer = account.to_bytes().to_vec();

        let ss_bytes = Self::ecdh(esk, &ipk.0.clone().try_into().unwrap());
        let epk = EphemeralPublicKey::from(esk);

        let key = Self::kdf(
            ss_bytes,
            &epk,
            ipk,
            // &commitment.to_byte_array(),
            output_index,
        );
        let mut cipher = ChaCha20::new(&key.into(), &[0; 12].into());
        cipher.apply_keystream(&mut buffer);

        let view_tag = Self::view_tag(&npk, &ipk);
        Self {
            ciphertext: buffer,
            epk,
            view_tag,
        }
    }

    pub fn kdf(
        ss_bytes: [u8; 32],
        epk: &EphemeralPublicKey,
        ipk: &IncomingViewingPublicKey,
        // commitment: &[u8; 32],
        output_index: u32,
    ) -> [u8; 32] {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(b"NSSA/v0.1/KDF-SHA256");
        bytes.extend_from_slice(&ss_bytes);
        bytes.extend_from_slice(&epk.0[..]);
        bytes.extend_from_slice(&ipk.0[..]);
        // bytes.extend_from_slice(&commitment[..]);
        bytes.extend_from_slice(&output_index.to_le_bytes());

        Impl::hash_bytes(&bytes).as_bytes().try_into().unwrap()
    }

    pub fn ecdh(scalar: &[u8; 32], point: &[u8; 33]) -> [u8; 32] {
        let scalar = Scalar::from_repr((*scalar).into()).unwrap();

        let encoded = EncodedPoint::from_bytes(point).unwrap();
        let pubkey_affine = AffinePoint::from_encoded_point(&encoded).unwrap();

        let shared = ProjectivePoint::from(pubkey_affine) * scalar;
        let shared_affine = shared.to_affine();

        let encoded = shared_affine.to_encoded_point(false);
        let x_bytes_slice = encoded.x().unwrap();
        let mut x_bytes = [0u8; 32];
        x_bytes.copy_from_slice(x_bytes_slice);

        x_bytes
    }

    #[cfg(feature = "host")]
    pub fn from_cursor(cursor: &mut Cursor<&[u8]>) -> Result<Self, NssaCoreError> {
        let mut u32_bytes = [0; 4];

        cursor.read_exact(&mut u32_bytes)?;
        let ciphertext_lenght = u32::from_le_bytes(u32_bytes);
        let mut ciphertext = vec![0; ciphertext_lenght as usize];
        cursor.read_exact(&mut ciphertext)?;

        let mut epk_bytes = vec![0; 33];
        cursor.read_exact(&mut epk_bytes)?;

        let mut tag_bytes = [0; 1];
        cursor.read_exact(&mut tag_bytes)?;

        Ok(Self {
            ciphertext,
            epk: Secp256k1Point(epk_bytes),
            view_tag: tag_bytes[0],
        })
    }

    fn view_tag(npk: &NullifierPublicKey, ipk: &&IncomingViewingPublicKey) -> u8 {
        // TODO: implement
        0
    }
}

impl EncryptedAccountData {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let ciphertext_length: u32 = self.ciphertext.len() as u32;
        bytes.extend_from_slice(&ciphertext_length.to_le_bytes());
        bytes.extend_from_slice(&self.ciphertext);
        bytes.extend_from_slice(&self.epk.0);
        bytes.push(self.view_tag);

        bytes
    }
}

#[derive(Serialize, Deserialize)]
pub struct PrivacyPreservingCircuitInput {
    pub program_output: ProgramOutput,
    pub visibility_mask: Vec<u8>,
    pub private_account_nonces: Vec<Nonce>,
    pub private_account_keys: Vec<(
        NullifierPublicKey,
        IncomingViewingPublicKey,
        EphemeralSecretKey,
    )>,
    pub private_account_auth: Vec<(NullifierSecretKey, MembershipProof)>,
    pub program_id: ProgramId,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(any(feature = "host", test), derive(Debug, PartialEq, Eq))]
pub struct PrivacyPreservingCircuitOutput {
    pub public_pre_states: Vec<AccountWithMetadata>,
    pub public_post_states: Vec<Account>,
    pub encrypted_private_post_states: Vec<EncryptedAccountData>,
    pub new_commitments: Vec<Commitment>,
    pub new_nullifiers: Vec<(Nullifier, CommitmentSetDigest)>,
}

#[cfg(feature = "host")]
impl PrivacyPreservingCircuitOutput {
    pub fn to_bytes(&self) -> Vec<u8> {
        bytemuck::cast_slice(&to_vec(&self).unwrap()).to_vec()
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use risc0_zkvm::serde::from_slice;

    use crate::{
        EncryptedAccountData, EphemeralPublicKey, PrivacyPreservingCircuitOutput, Secp256k1Point,
        account::{Account, AccountWithMetadata, Commitment, Nullifier, NullifierPublicKey},
    };

    #[test]
    fn test_privacy_preserving_circuit_output_to_bytes_is_compatible_with_from_slice() {
        let output = PrivacyPreservingCircuitOutput {
            public_pre_states: vec![
                AccountWithMetadata {
                    account: Account {
                        program_owner: [1, 2, 3, 4, 5, 6, 7, 8],
                        balance: 12345678901234567890,
                        data: b"test data".to_vec(),
                        nonce: 18446744073709551614,
                    },
                    is_authorized: true,
                },
                AccountWithMetadata {
                    account: Account {
                        program_owner: [9, 9, 9, 8, 8, 8, 7, 7],
                        balance: 123123123456456567112,
                        data: b"test data".to_vec(),
                        nonce: 9999999999999999999999,
                    },
                    is_authorized: false,
                },
            ],
            public_post_states: vec![Account {
                program_owner: [1, 2, 3, 4, 5, 6, 7, 8],
                balance: 100,
                data: b"post state data".to_vec(),
                nonce: 18446744073709551615,
            }],
            encrypted_private_post_states: vec![EncryptedAccountData {
                ciphertext: vec![255, 255, 1, 1, 2, 2],
                epk: EphemeralPublicKey::from_scalar([123; 32]),
                view_tag: 1,
            }],
            new_commitments: vec![Commitment::new(
                &NullifierPublicKey::from(&[1; 32]),
                &Account::default(),
            )],
            new_nullifiers: vec![(
                Nullifier::new(
                    &Commitment::new(&NullifierPublicKey::from(&[2; 32]), &Account::default()),
                    &[1; 32],
                ),
                [0xab; 32],
            )],
        };
        let bytes = output.to_bytes();
        let output_from_slice: PrivacyPreservingCircuitOutput = from_slice(&bytes).unwrap();
        assert_eq!(output, output_from_slice);
    }

    #[test]
    fn test_encrypted_account_data_to_bytes_roundtrip() {
        let data = EncryptedAccountData {
            ciphertext: vec![255, 255, 1, 1, 2, 2],
            epk: EphemeralPublicKey::from_scalar([123; 32]),
            view_tag: 95,
        };
        let bytes = data.to_bytes();
        let mut cursor = Cursor::new(bytes.as_slice());
        let data_from_cursor = EncryptedAccountData::from_cursor(&mut cursor).unwrap();
        assert_eq!(data, data_from_cursor);
    }
}
