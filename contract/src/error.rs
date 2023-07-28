use baby_liminal_extension::BabyLiminalError;
use ink::{prelude::vec::Vec, primitives::AccountId};
use scale::{Decode, Encode};

#[derive(Eq, PartialEq, Debug, Decode, Encode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum BrightDisputesError {
    DisputeNotExist,
    NotAuthorized,
    InvalidDisputeState,
    InvalidAction,
    InvalidEscrowAmount,

    JurorAlreadyVoted,
    JurorAlreadyAdded,
    JurorAlreadyRegistered,
    JurorAlreadyAssignedToDispute,
    JurorIsNotAssignedToDispute,
    JurorAlreadyConfirmedDispute,
    JurorInvalidState,
    JurorNotExist,
    JuriesPoolIsToSmall,
    JuriesNotVoted(Vec<AccountId>),
    JudgeAlreadyAssignedToDispute,    

    DisputeRoundDeadlineReached,
    DisputeRoundLimitReached,
    DisputeRoundNotStarted,
    WrongDisputeRoundState,
    CanNotSwitchDisputeRound,

    MajorityOfVotesNotReached,

    NotRegisteredAsJuror,

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
