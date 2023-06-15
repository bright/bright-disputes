use baby_liminal_extension::BabyLiminalError;
use scale::{Decode, Encode};

#[derive(Eq, PartialEq, Debug, Decode, Encode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum BrightDisputesError {
    DisputeNotExist,
    NotAuthorized,
    InvalidState,
    InvalidAction,

    JureAlreadyVoted,
    JureAlreadyAdded,
    JureAlreadyRegistered,
    JureAlreadyAssignedToDispute,
    JureIsNotAssignedToDispute,
    JureNotExist,
    JuriesPoolIsToSmall,

    NotRegisteredAsJure,

    /// Pallet returned an error (through chain extension).
    InkError,

    /// Pallet returned an error (through chain extension).
    ChainExtension(BabyLiminalError),
}

impl From<ink::env::Error> for BrightDisputesError {
    fn from(_e: ink::env::Error) -> Self {
        BrightDisputesError::InkError
    }
}

impl From<BabyLiminalError> for BrightDisputesError {
    fn from(e: BabyLiminalError) -> Self {
        BrightDisputesError::ChainExtension(e)
    }
}
