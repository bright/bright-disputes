use baby_liminal_extension::BabyLiminalError;
use scale::{Decode, Encode};

#[derive(Eq, PartialEq, Debug, Decode, Encode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum BrightDisputesError {
    DisputeAlreadyCreated,
    DisputeNotExist,
    NotAuthorized,
    InvalidState,
    InvalidAction,
    JureAlreadyVoted,
    JureAlreadyAdded,

    /// Pallet returned an error (through chain extension).
    ChainExtension(BabyLiminalError),
}

impl From<BabyLiminalError> for BrightDisputesError {
    fn from(e: BabyLiminalError) -> Self {
        BrightDisputesError::ChainExtension(e)
    }
}