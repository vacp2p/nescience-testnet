use k256::elliptic_curve::group::GroupEncoding;
use k256::{elliptic_curve::PrimeField, AffinePoint, FieldBytes, Scalar};
use rand::{rngs::OsRng, RngCore};
use sha2::{digest::FixedOutput, Digest};
use storage::merkle_tree_public::TreeHashType;

pub const NULLIFIER_SECRET_CONST: [u8; 32] = [
    38, 29, 97, 210, 148, 172, 75, 220, 36, 249, 27, 111, 73, 14, 250, 38, 55, 87, 164, 169, 95,
    101, 135, 28, 212, 241, 107, 46, 162, 60, 59, 93,
];
pub const VIEVING_SECRET_CONST: [u8; 32] = [
    97, 23, 175, 117, 11, 48, 215, 162, 150, 103, 46, 195, 179, 178, 93, 52, 137, 190, 202, 60,
    254, 87, 112, 250, 57, 242, 117, 206, 195, 149, 213, 206,
];

#[derive(Debug)]
///Seed holder. Non-clonable to ensure that different holders use different seeds.
/// Produces `TopSecretKeyHolder` objects.
pub struct SeedHolder {
    seed: Scalar,
}

#[derive(Debug, Clone)]
///Secret spending key holder. Produces `UTXOSecretKeyHolder` objects.
pub struct TopSecretKeyHolder {
    secret_spending_key: Scalar,
}

#[derive(Debug, Clone)]
///Nullifier secret key and viewing secret key holder. Produces public keys. Can produce address. Can produce shared secret for recepient.
pub struct UTXOSecretKeyHolder {
    nullifier_secret_key: Scalar,
    viewing_secret_key: Scalar,
}

impl SeedHolder {
    pub fn new_os_random() -> Self {
        let mut bytes = FieldBytes::default();

        OsRng.fill_bytes(&mut bytes);

        Self {
            seed: Scalar::from_repr(bytes).unwrap(),
        }
    }

    pub fn generate_secret_spending_key_hash(&self) -> TreeHashType {
        let mut hasher = sha2::Sha256::new();

        hasher.update(self.seed.to_bytes());

        <TreeHashType>::from(hasher.finalize_fixed())
    }

    pub fn generate_secret_spending_key_scalar(&self) -> Scalar {
        let hash = self.generate_secret_spending_key_hash();

        Scalar::from_repr(hash.into()).unwrap()
    }

    pub fn produce_top_secret_key_holder(&self) -> TopSecretKeyHolder {
        TopSecretKeyHolder {
            secret_spending_key: self.generate_secret_spending_key_scalar(),
        }
    }
}

impl TopSecretKeyHolder {
    pub fn generate_nullifier_secret_key(&self) -> Scalar {
        let mut hasher = sha2::Sha256::new();

        hasher.update(self.secret_spending_key.to_bytes());
        hasher.update(NULLIFIER_SECRET_CONST);

        let hash = <TreeHashType>::from(hasher.finalize_fixed());

        Scalar::from_repr(hash.into()).unwrap()
    }

    pub fn generate_viewing_secret_key(&self) -> Scalar {
        let mut hasher = sha2::Sha256::new();

        hasher.update(self.secret_spending_key.to_bytes());
        hasher.update(VIEVING_SECRET_CONST);

        let hash = <TreeHashType>::from(hasher.finalize_fixed());

        Scalar::from_repr(hash.into()).unwrap()
    }

    pub fn produce_utxo_secret_holder(&self) -> UTXOSecretKeyHolder {
        UTXOSecretKeyHolder {
            nullifier_secret_key: self.generate_nullifier_secret_key(),
            viewing_secret_key: self.generate_viewing_secret_key(),
        }
    }
}

impl UTXOSecretKeyHolder {
    pub fn generate_nullifier_public_key(&self) -> AffinePoint {
        (AffinePoint::GENERATOR * self.nullifier_secret_key).into()
    }

    pub fn generate_viewing_public_key(&self) -> AffinePoint {
        (AffinePoint::GENERATOR * self.viewing_secret_key).into()
    }

    pub fn generate_address(&self) -> TreeHashType {
        let npk = self.generate_nullifier_public_key();
        let vpk = self.generate_viewing_public_key();

        let mut hasher = sha2::Sha256::new();

        hasher.update(npk.to_bytes());
        hasher.update(vpk.to_bytes());

        <TreeHashType>::from(hasher.finalize_fixed())
    }
}

#[derive(Debug)]
///Ephemeral secret key holder. Non-clonable as intended for one-time use. Produces ephemeral public keys. Can produce shared secret for sender.
pub struct EphemeralKeyHolder {
    ephemeral_secret_key: Scalar,
}

impl EphemeralKeyHolder {
    pub fn new_os_random() -> Self {
        let mut bytes = FieldBytes::default();

        OsRng.fill_bytes(&mut bytes);

        Self {
            ephemeral_secret_key: Scalar::from_repr(bytes).unwrap(),
        }
    }

    pub fn generate_ephemeral_public_key(&self) -> AffinePoint {
        (AffinePoint::GENERATOR * self.ephemeral_secret_key).into()
    }

    pub fn calculate_shared_secret_sender(
        &self,
        viewing_public_key_receiver: AffinePoint,
    ) -> AffinePoint {
        (viewing_public_key_receiver * self.ephemeral_secret_key).into()
    }

    pub fn encrypt_data(&self) {
        //ToDo: Implement that
        //Need clarification on exact symmetric encoding, which we want to use for ECIES
        todo!()
    }
}

#[derive(Debug)]
///Entrypoint to key management
pub struct AddressKeyHolder {
    utxo_secret_key_holder: UTXOSecretKeyHolder,
    pub address: TreeHashType,
    pub nullifer_public_key: AffinePoint,
    pub viewing_public_key: AffinePoint,
}

impl AddressKeyHolder {
    pub fn new_os_random() -> Self {
        //Currently dropping SeedHolder and TopSecretKeyHolder at the end of initialization.
        //Now entirely sure if we need them in the future.
        let seed_holder = SeedHolder::new_os_random();
        let top_secret_key_holder = seed_holder.produce_top_secret_key_holder();

        let utxo_secret_key_holder = top_secret_key_holder.produce_utxo_secret_holder();

        let address = utxo_secret_key_holder.generate_address();
        let nullifer_public_key = utxo_secret_key_holder.generate_nullifier_public_key();
        let viewing_public_key = utxo_secret_key_holder.generate_viewing_public_key();

        Self {
            utxo_secret_key_holder,
            address,
            nullifer_public_key,
            viewing_public_key,
        }
    }

    pub fn calculate_shared_secret_receiver(
        &self,
        ephemeral_public_key_sender: AffinePoint,
    ) -> AffinePoint {
        (ephemeral_public_key_sender * self.utxo_secret_key_holder.viewing_secret_key).into()
    }

    pub fn produce_ephemeral_key_holder(&self) -> EphemeralKeyHolder {
        EphemeralKeyHolder::new_os_random()
    }
}
