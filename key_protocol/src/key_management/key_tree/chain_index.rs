use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize)]
pub struct ChainIndex(Vec<u32>);

impl FromStr for ChainIndex {
    type Err = hex::FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(Self(vec![]));
        }

        let hex_decoded = hex::decode(s)?;

        if !hex_decoded.len().is_multiple_of(4) {
            Err(hex::FromHexError::InvalidStringLength)
        } else {
            let mut res_vec = vec![];

            for i in 0..(hex_decoded.len() / 4) {
                res_vec.push(u32::from_le_bytes([
                    hex_decoded[4 * i],
                    hex_decoded[4 * i + 1],
                    hex_decoded[4 * i + 2],
                    hex_decoded[4 * i + 3],
                ]));
            }

            Ok(Self(res_vec))
        }
    }
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for ChainIndex {
    fn to_string(&self) -> String {
        if self.0.is_empty() {
            return "".to_string();
        }

        let mut res_vec = vec![];

        for index in &self.0 {
            res_vec.extend_from_slice(&index.to_le_bytes());
        }

        hex::encode(res_vec)
    }
}

impl ChainIndex {
    pub fn root() -> Self {
        ChainIndex::from_str("").unwrap()
    }

    pub fn chain(&self) -> &[u32] {
        &self.0
    }

    pub fn next_in_line(&self) -> ChainIndex {
        let mut chain = self.0.clone();
        //ToDo: Add overflow check
        if let Some(last_p) = chain.last_mut() {
            *last_p += 1
        }

        ChainIndex(chain)
    }

    pub fn n_th_child(&self, child_id: u32) -> ChainIndex {
        let mut chain = self.0.clone();
        chain.push(child_id);

        ChainIndex(chain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_id_root_correct() {
        let chain_id = ChainIndex::root();
        let chain_id_2 = ChainIndex::from_str("").unwrap();

        assert_eq!(chain_id, chain_id_2);
    }

    #[test]
    fn test_chain_id_deser_correct() {
        let chain_id = ChainIndex::from_str("01010000").unwrap();

        assert_eq!(chain_id.chain(), &[257]);
    }

    #[test]
    fn test_chain_id_next_in_line_correct() {
        let chain_id = ChainIndex::from_str("01010000").unwrap();
        let next_in_line = chain_id.next_in_line();

        assert_eq!(next_in_line, ChainIndex::from_str("02010000").unwrap());
    }

    #[test]
    fn test_chain_id_child_correct() {
        let chain_id = ChainIndex::from_str("01010000").unwrap();
        let child = chain_id.n_th_child(3);

        assert_eq!(child, ChainIndex::from_str("0101000003000000").unwrap());
    }
}
