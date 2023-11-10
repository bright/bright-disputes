use ink::primitives::AccountId;

use crate::types::VoteHash;
#[derive(Clone, Debug, PartialEq, scale::Decode, scale::Encode)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout, scale_info::TypeInfo)
)]
pub struct Vote {
    juror: AccountId,
    vote: VoteHash,
}

impl Vote {
    #[allow(dead_code)]
    pub fn create(juror: AccountId, vote: VoteHash) -> Self {
        Vote { juror, vote }
    }
    pub fn juror(&self) -> AccountId {
        self.juror
    }
    pub fn vote(&self) -> VoteHash {
        self.vote
    }
}
