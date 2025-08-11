use nssa_core::{
    account::Nonce,
    program::{InstructionData, ProgramId},
};
use serde::{Deserialize, Serialize};

use crate::{Address, error::NssaError, program::Program};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Message {
    pub(crate) program_id: ProgramId,
    pub(crate) addresses: Vec<Address>,
    pub(crate) nonces: Vec<Nonce>,
    pub(crate) instruction_data: InstructionData,
}

impl Message {
    pub fn try_new<T: Serialize>(
        program_id: ProgramId,
        addresses: Vec<Address>,
        nonces: Vec<Nonce>,
        instruction: T,
    ) -> Result<Self, NssaError> {
        let instruction_data = Program::serialize_instruction(instruction)?;
        Ok(Self {
            program_id,
            addresses,
            nonces,
            instruction_data,
        })
    }

    /// Serializes a `Message` into bytes in the following layout:
    /// TAG || <program_id>  (bytes LE) * 8 || addresses_len (4 bytes LE) || addresses (32 bytes * N) || nonces_len (4 bytes LE) || nonces (16 bytes * M) || instruction_data_len || instruction_data (4 bytes * K)
    /// Integers and words are encoded in little-endian byte order, and fields appear in the above order.
    pub(crate) fn message_to_bytes(&self) -> Vec<u8> {
        const MESSAGE_ENCODING_PREFIX: &[u8; 19] = b"NSSA/v0.1/TxMessage";

        let mut bytes = MESSAGE_ENCODING_PREFIX.to_vec();
        // program_id: [u32; 8]
        for word in &self.program_id {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        // addresses: Vec<[u8;32]>
        // serialize length as u32 little endian, then all addresses concatenated
        let addresses_len = self.addresses.len() as u32;
        bytes.extend(&addresses_len.to_le_bytes());
        for addr in &self.addresses {
            bytes.extend_from_slice(addr.value());
        }
        // nonces: Vec<u128>
        let nonces_len = self.nonces.len() as u32;
        bytes.extend(&nonces_len.to_le_bytes());
        for nonce in &self.nonces {
            bytes.extend(&nonce.to_le_bytes());
        }
        // instruction_data: Vec<u32>
        // serialize length as u32 little endian, then all addresses concatenated
        let instr_len = self.instruction_data.len() as u32;
        bytes.extend(&instr_len.to_le_bytes());
        for word in &self.instruction_data {
            bytes.extend(&word.to_le_bytes());
        }

        bytes
    }
}
