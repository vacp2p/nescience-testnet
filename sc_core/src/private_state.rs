use std::collections::BTreeMap;

use serde::{de::Error, Deserialize, Serialize};

pub const PRIVATE_BLOB_SIZE: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrivateDataBlob(pub [u8; PRIVATE_BLOB_SIZE]);

pub type PrivateSCState = BTreeMap<usize, PrivateDataBlob>;

#[derive(thiserror::Error, Debug)]
pub enum PrivateDataBlobError {
    #[error("Given data len: {0} does not fit into {PRIVATE_BLOB_SIZE}")]
    DataDoesNotFit(usize),
}

#[derive(thiserror::Error, Debug)]
pub enum PrivateStateError {
    #[error("Trying to read from slot too big: Read slot {0}, max_slot {1}")]
    ReadSizeMismatch(usize, usize),
    #[error("Can not write empty bytes into state")]
    EmptyWrite,
    #[error("Error occured while interacting ith PrivateDataBlob {0}")]
    PrivateDataBlobError(#[from] PrivateDataBlobError),
}

impl From<[u8; PRIVATE_BLOB_SIZE]> for PrivateDataBlob {
    fn from(value: [u8; PRIVATE_BLOB_SIZE]) -> Self {
        Self(value)
    }
}

impl Serialize for PrivateDataBlob {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let data_vec = self.0.to_vec();
        data_vec.serialize(serializer)
    }
}

impl AsRef<[u8]> for PrivateDataBlob {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl<'de> Deserialize<'de> for PrivateDataBlob {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let data_vec = Vec::<u8>::deserialize(deserializer)?;
        let chunk: [u8; PRIVATE_BLOB_SIZE] = data_vec
            .try_into()
            .map_err(|data| {
                anyhow::anyhow!("failed to fit vec {data:?} to {:?}", PRIVATE_BLOB_SIZE)
            })
            .map_err(D::Error::custom)?;
        Ok(Self(chunk))
    }
}

impl PrivateDataBlob {
    ///Produce `DataBlob` from vector of size <= `PRIVATE_BLOB_SIZE`
    ///
    ///Extends to `PRIVATE_BLOB_SIZE`, if necessary.
    ///
    /// Returns an error, if size > `PRIVATE_BLOB_SIZE`
    pub fn try_produce_blob_from_fit_vec(data: Vec<u8>) -> Result<Self, PrivateDataBlobError> {
        let data_len = data.len();

        if data_len <= PRIVATE_BLOB_SIZE {
            return Err(PrivateDataBlobError::DataDoesNotFit(data_len));
        }

        let mut blob: PrivateDataBlob = [0; PRIVATE_BLOB_SIZE].into();

        for (idx, item) in data.into_iter().enumerate() {
            blob.0[idx] = item
        }

        Ok(blob)
    }

    ///Produce `DataBlob` from slice of size <= `PRIVATE_BLOB_SIZE`
    ///
    ///Extends to `PRIVATE_BLOB_SIZE`, if necessary.
    ///
    /// Returns an error, if size > `PRIVATE_BLOB_SIZE`
    pub fn try_produce_blob_from_fit_slice(data: &[u8]) -> Result<Self, PrivateDataBlobError> {
        let data_len = data.len();

        if data_len <= PRIVATE_BLOB_SIZE {
            return Err(PrivateDataBlobError::DataDoesNotFit(data_len));
        }

        let mut blob: PrivateDataBlob = [0; PRIVATE_BLOB_SIZE].into();

        for (idx, item) in data.into_iter().enumerate() {
            blob.0[idx] = *item
        }

        Ok(blob)
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

pub fn calculate_offset_slot(offset: usize) -> usize {
    offset / PRIVATE_BLOB_SIZE
}

pub fn max_slot(state: &PrivateSCState) -> usize {
    *state.keys().max().unwrap_or(&0)
}

///Read at least `num` bytes from the start of a state
pub fn read_num_bytes_start(
    state: &PrivateSCState,
    num: usize,
) -> Result<Vec<PrivateDataBlob>, PrivateStateError> {
    let mut resp = vec![];

    let max_offset_slot = calculate_offset_slot(num);
    let max_slot_state = max_slot(state);
    if max_offset_slot > max_slot_state {
        return Err(PrivateStateError::ReadSizeMismatch(
            max_offset_slot,
            max_slot_state,
        ));
    }

    for i in 0..max_offset_slot {
        resp.push(*state.get(&i).unwrap());
    }

    Ok(resp)
}

///Read at least `num` bytes from the `offset` slot
pub fn read_num_bytes_offset(
    state: &PrivateSCState,
    num: usize,
    offset: usize,
) -> Result<Vec<PrivateDataBlob>, PrivateStateError> {
    let mut resp = vec![];

    let max_offset_slot = offset + calculate_offset_slot(num);
    let max_slot_state = max_slot(state);
    if max_offset_slot > max_slot_state {
        return Err(PrivateStateError::ReadSizeMismatch(
            max_offset_slot,
            max_slot_state,
        ));
    }

    for i in offset..max_offset_slot {
        resp.push(*state.get(&i).unwrap());
    }

    Ok(resp)
}

///Write at least `bytes.len()` bytes at the end of the state
///
/// Returns new last slot
pub fn write_num_bytes_append(
    state: &mut PrivateSCState,
    bytes: Vec<u8>,
) -> Result<usize, PrivateStateError> {
    if bytes.is_empty() {
        return Err(PrivateStateError::EmptyWrite);
    }

    let mut max_slot_state = max_slot(state) + 1;

    let mut curr = 0;

    while (curr + PRIVATE_BLOB_SIZE) < bytes.len() {
        let data_blob = PrivateDataBlob::try_produce_blob_from_fit_slice(
            &bytes[curr..(curr + PRIVATE_BLOB_SIZE)],
        )?;

        state.insert(max_slot_state, data_blob);

        curr += PRIVATE_BLOB_SIZE;
        max_slot_state += 1;
    }

    let data_blob = PrivateDataBlob::try_produce_blob_from_fit_slice(&bytes[curr..(bytes.len())])?;

    state.insert(max_slot_state, data_blob);

    Ok(max_slot_state)
}

/// Rewrite at least `bytes.len()` bytes starting from the offset slot
///
/// Returns last (re)written slot
pub fn write_num_bytes_rewrite(
    state: &mut PrivateSCState,
    bytes: Vec<u8>,
    offset: usize,
) -> Result<usize, PrivateStateError> {
    if bytes.is_empty() {
        return Err(PrivateStateError::EmptyWrite);
    }

    let mut curr_slot = offset;

    let mut curr = 0;

    while (curr + PRIVATE_BLOB_SIZE) < bytes.len() {
        let data_blob = PrivateDataBlob::try_produce_blob_from_fit_slice(
            &bytes[curr..(curr + PRIVATE_BLOB_SIZE)],
        )?;

        state.insert(curr_slot, data_blob);

        curr += PRIVATE_BLOB_SIZE;
        curr_slot += 1;
    }

    let data_blob = PrivateDataBlob::try_produce_blob_from_fit_slice(&bytes[curr..(bytes.len())])?;

    state.insert(curr_slot, data_blob);

    Ok(curr_slot)
}
