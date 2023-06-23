use ink::primitives::AccountId;

use crate::types::VoteValue;
#[derive(Clone, Debug, PartialEq, scale::Decode, scale::Encode)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout, scale_info::TypeInfo)
)]
pub struct Vote {
    jure: AccountId,
    vote: VoteValue,
}

impl Vote {
    #[allow(dead_code)]
    pub fn create(jure: AccountId, vote: VoteValue) -> Self {
        Vote { jure, vote }
    }
    pub fn jure(&self) -> AccountId {
        self.jure
    }
    pub fn vote(&self) -> VoteValue {
        self.vote
    }
    pub fn is_vote_against_owner(&self) -> bool {
        self.vote == 1u8
    }
}
