use scale::Encode as _;

// This file was auto-generated with ink-wrapper (https://crates.io/crates/ink-wrapper).

#[allow(dead_code)]
pub const CODE_HASH: [u8; 32] = [
    74, 171, 47, 183, 197, 181, 13, 124, 128, 35, 80, 186, 0, 132, 168, 172, 26, 237, 209, 124,
    143, 150, 217, 23, 239, 111, 34, 83, 70, 21, 47, 9,
];

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
pub struct Vote {
    pub jure: ink_primitives::AccountId,
    pub vote: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
pub struct Dispute {
    pub id: u32,
    pub state: DisputeState,
    pub owner: ink_primitives::AccountId,
    pub owner_link: String,
    pub escrow: u128,
    pub deposit: u128,
    pub defendant: ink_primitives::AccountId,
    pub defendant_link: Option<String>,
    pub dispute_result: Option<DisputeResult>,
    pub dispute_round: Option<DisputeRound>,
    pub dispute_round_counter: u8,
    pub judge: Option<ink_primitives::AccountId>,
    pub juries: Vec<ink_primitives::AccountId>,
    pub banned: Vec<ink_primitives::AccountId>,
    pub votes: Vec<Vote>,
}

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
pub enum DisputeState {
    Created(),
    Running(),
    Ended(),
    Closed(),
}

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
pub enum DisputeResult {
    Owner(),
    Defendant(),
}

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
pub struct DisputeRound {
    pub state: RoundState,
    pub number_of_juries: u8,
    pub state_deadline: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
pub enum RoundState {
    AssignJuriesAndJudge(),
    PickingJuriesAndJudge(),
    Voting(),
    CountingTheVotes(),
}

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
pub enum BrightDisputesError {
    DisputeNotExist(),
    NotAuthorized(),
    InvalidDisputeState(),
    InvalidAction(),
    InvalidEscrowAmount(),
    JureAlreadyVoted(),
    JureAlreadyAdded(),
    JureAlreadyRegistered(),
    JureAlreadyAssignedToDispute(),
    JureIsNotAssignedToDispute(),
    JureAlreadyConfirmedDispute(),
    JureInvalidState(),
    JureNotExist(),
    JuriesPoolIsToSmall(),
    JuriesNotVoted(Vec<ink_primitives::AccountId>),
    JudgeAlreadyAssignedToDispute(),
    DisputeRoundDeadlineReached(),
    DisputeRoundLimitReached(),
    DisputeRoundNotStarted(),
    WrongDisputeRoundState(),
    CanNotSwitchDisputeRound(),
    MajorityOfVotesNotReached(),
    NotRegisteredAsJure(),
    InkError(),
    ChainExtension(BabyLiminalError),
}

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
pub enum BabyLiminalError {
    IdentifierAlreadyInUse(),
    VerificationKeyTooLong(),
    StoreKeyErrorUnknown(),
    UnknownVerificationKeyIdentifier(),
    DeserializingProofFailed(),
    DeserializingPublicInputFailed(),
    DeserializingVerificationKeyFailed(),
    VerificationFailed(),
    IncorrectProof(),
    VerifyErrorUnknown(),
}

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
pub struct Extension();

pub mod event {
    #[allow(dead_code, clippy::large_enum_variant)]
    #[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
    pub enum Event {
        DisputeRaised {
            id: u32,
            owner_id: ink_primitives::AccountId,
            defendant_id: ink_primitives::AccountId,
        },

        DisputeClosed {
            id: u32,
        },

        DefendantConfirmDispute {
            id: u32,
            defendant_id: ink_primitives::AccountId,
        },

        DisputeResultEvent {
            id: u32,
            result: super::DisputeResult,
        },
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Instance {
    account_id: ink_primitives::AccountId,
}

impl From<ink_primitives::AccountId> for Instance {
    fn from(account_id: ink_primitives::AccountId) -> Self {
        Self { account_id }
    }
}

impl From<Instance> for ink_primitives::AccountId {
    fn from(instance: Instance) -> Self {
        instance.account_id
    }
}

impl ink_wrapper_types::EventSource for Instance {
    type Event = event::Event;
}

#[allow(dead_code)]
pub fn upload() -> ink_wrapper_types::UploadCall {
    let wasm = include_bytes!("../contract/target/ink/bright_disputes.wasm");
    ink_wrapper_types::UploadCall::new(wasm.to_vec(), CODE_HASH)
}

impl Instance {
    /// Constructor
    #[allow(dead_code, clippy::too_many_arguments)]
    pub fn new() -> ink_wrapper_types::InstantiateCall<Self> {
        let data = vec![155, 174, 157, 94];
        ink_wrapper_types::InstantiateCall::new(CODE_HASH, data)
    }

    ///  Get last dispute id
    #[allow(dead_code, clippy::too_many_arguments)]
    pub fn get_last_dispute_id(
        &self,
    ) -> ink_wrapper_types::ReadCall<Result<u32, ink_wrapper_types::InkLangError>> {
        let data = vec![241, 53, 223, 85];
        ink_wrapper_types::ReadCall::new(self.account_id, data)
    }

    ///  Get single dispute by id
    #[allow(dead_code, clippy::too_many_arguments)]
    pub fn get_dispute(
        &self,
        dispute_id: u32,
    ) -> ink_wrapper_types::ReadCall<
        Result<Result<Dispute, BrightDisputesError>, ink_wrapper_types::InkLangError>,
    > {
        let data = {
            let mut data = vec![76, 253, 140, 199];
            dispute_id.encode_to(&mut data);
            data
        };
        ink_wrapper_types::ReadCall::new(self.account_id, data)
    }

    ///  Get all disputes
    #[allow(dead_code, clippy::too_many_arguments)]
    pub fn get_all_disputes(
        &self,
    ) -> ink_wrapper_types::ReadCall<Result<Vec<Dispute>, ink_wrapper_types::InkLangError>> {
        let data = vec![29, 12, 242, 122];
        ink_wrapper_types::ReadCall::new(self.account_id, data)
    }

    ///  Get single dispute by id
    #[allow(dead_code, clippy::too_many_arguments)]
    pub fn remove_dispute(&self, dispute_id: u32) -> ink_wrapper_types::ExecCall {
        let data = {
            let mut data = vec![187, 47, 61, 10];
            dispute_id.encode_to(&mut data);
            data
        };
        ink_wrapper_types::ExecCall::new(self.account_id, data)
    }

    ///  Create new dispute
    #[allow(dead_code, clippy::too_many_arguments)]
    pub fn create_dispute(
        &self,
        owner_link: String,
        defendant_id: ink_primitives::AccountId,
        escrow: u128,
    ) -> ink_wrapper_types::ExecCallNeedsValue {
        let data = {
            let mut data = vec![30, 25, 167, 107];
            owner_link.encode_to(&mut data);
            defendant_id.encode_to(&mut data);
            escrow.encode_to(&mut data);
            data
        };
        ink_wrapper_types::ExecCallNeedsValue::new(self.account_id, data)
    }

    ///  Defendant confirms his participation in dispute.
    #[allow(dead_code, clippy::too_many_arguments)]
    pub fn confirm_defendant(
        &self,
        dispute_id: u32,
        defendant_link: String,
    ) -> ink_wrapper_types::ExecCallNeedsValue {
        let data = {
            let mut data = vec![164, 183, 46, 125];
            dispute_id.encode_to(&mut data);
            defendant_link.encode_to(&mut data);
            data
        };
        ink_wrapper_types::ExecCallNeedsValue::new(self.account_id, data)
    }

    ///  Update owner link description
    #[allow(dead_code, clippy::too_many_arguments)]
    pub fn update_owner_description(
        &self,
        dispute_id: u32,
        owner_link: String,
    ) -> ink_wrapper_types::ExecCall {
        let data = {
            let mut data = vec![68, 73, 200, 222];
            dispute_id.encode_to(&mut data);
            owner_link.encode_to(&mut data);
            data
        };
        ink_wrapper_types::ExecCall::new(self.account_id, data)
    }

    ///  Update defendant link description
    #[allow(dead_code, clippy::too_many_arguments)]
    pub fn update_defendant_description(
        &self,
        dispute_id: u32,
        defendant_link: String,
    ) -> ink_wrapper_types::ExecCall {
        let data = {
            let mut data = vec![208, 24, 65, 84];
            dispute_id.encode_to(&mut data);
            defendant_link.encode_to(&mut data);
            data
        };
        ink_wrapper_types::ExecCall::new(self.account_id, data)
    }

    ///  Voting, only jure can do it.
    #[allow(dead_code, clippy::too_many_arguments)]
    pub fn vote(&self, dispute_id: u32, vote: u8) -> ink_wrapper_types::ExecCall {
        let data = {
            let mut data = vec![8, 59, 226, 96];
            dispute_id.encode_to(&mut data);
            vote.encode_to(&mut data);
            data
        };
        ink_wrapper_types::ExecCall::new(self.account_id, data)
    }

    ///  Register as an active jure. Juries are picked
    ///  from this pool to participate in disputes.
    #[allow(dead_code, clippy::too_many_arguments)]
    pub fn register_as_an_active_jure(&self) -> ink_wrapper_types::ExecCall {
        let data = vec![121, 6, 115, 245];
        ink_wrapper_types::ExecCall::new(self.account_id, data)
    }

    ///  Unregister jure from the active juries pool.
    #[allow(dead_code, clippy::too_many_arguments)]
    pub fn unregister_as_an_active_jure(&self) -> ink_wrapper_types::ExecCall {
        let data = vec![121, 53, 33, 150];
        ink_wrapper_types::ExecCall::new(self.account_id, data)
    }

    ///  Assigned jure can confirm his participation in dispute
    #[allow(dead_code, clippy::too_many_arguments)]
    pub fn confirm_jure_participation_in_dispute(
        &self,
        dispute_id: u32,
    ) -> ink_wrapper_types::ExecCallNeedsValue {
        let data = {
            let mut data = vec![149, 58, 196, 227];
            dispute_id.encode_to(&mut data);
            data
        };
        ink_wrapper_types::ExecCallNeedsValue::new(self.account_id, data)
    }

    ///  Judge can confirm his participation in dispute
    #[allow(dead_code, clippy::too_many_arguments)]
    pub fn confirm_judge_participation_in_dispute(
        &self,
        dispute_id: u32,
    ) -> ink_wrapper_types::ExecCallNeedsValue {
        let data = {
            let mut data = vec![178, 215, 24, 15];
            dispute_id.encode_to(&mut data);
            data
        };
        ink_wrapper_types::ExecCallNeedsValue::new(self.account_id, data)
    }

    ///  Unregister jure from the active juries pool.
    #[allow(dead_code, clippy::too_many_arguments)]
    pub fn process_dispute_round(&self, dispute_id: u32) -> ink_wrapper_types::ExecCall {
        let data = {
            let mut data = vec![14, 13, 134, 23];
            dispute_id.encode_to(&mut data);
            data
        };
        ink_wrapper_types::ExecCall::new(self.account_id, data)
    }

    ///  Judge can confirm his participation in dispute
    #[allow(dead_code, clippy::too_many_arguments)]
    pub fn distribute_deposit(&self, dispute_id: u32) -> ink_wrapper_types::ExecCall {
        let data = {
            let mut data = vec![117, 233, 246, 239];
            dispute_id.encode_to(&mut data);
            data
        };
        ink_wrapper_types::ExecCall::new(self.account_id, data)
    }
}
