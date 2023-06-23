use crate::error::BrightDisputesError;

pub type Balance = u128;
pub type DisputeId = u32;
pub type VoteValue = u8;
pub type Timestamp = u64;
pub type Result<T> = core::result::Result<T, BrightDisputesError>;
