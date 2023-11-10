use crate::error::BrightDisputesError;
use ink::{prelude::vec::Vec, primitives::AccountId};

pub type Balance = u128;
pub type DisputeId = u32;
pub type Proof = Vec<u8>;
pub type Result<T> = core::result::Result<T, BrightDisputesError>;
pub type Timestamp = u64;
pub type VoteHash = [u64; 4];
pub type AccountsVec = Vec<AccountId>;
pub type PublicKey = Vec<u8>;
