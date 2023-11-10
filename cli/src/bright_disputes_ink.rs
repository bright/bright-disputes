use scale::Encode as _;

// This file was auto-generated with ink-wrapper (https://crates.io/crates/ink-wrapper).

#[allow(dead_code)]
pub const CODE_HASH: [u8; 32] = [
    130, 214, 182, 242, 229, 248, 203, 44, 209, 112, 26, 249, 97, 59, 55, 140, 79, 16, 61, 217,
    180, 9, 197, 216, 115, 133, 245, 120, 19, 53, 156, 20,
];

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
pub struct Vote {
    pub juror: ink_primitives::AccountId,
    pub vote: [u64; 4],
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
    pub votes_hash: [u64; 4],
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
    JurorAlreadyVoted(),
    JurorAlreadyAdded(),
    JurorAlreadyRegistered(),
    JurorAlreadyAssignedToDispute(),
    JurorIsNotAssignedToDispute(),
    JurorAlreadyConfirmedDispute(),
    JurorHasNotConfirmedDispute(),
    JurorInvalidState(),
    JurorNotExist(),
    JuriesPoolIsToSmall(),
    JuriesNotVoted(Vec<ink_primitives::AccountId>),
    JudgeAlreadyAssignedToDispute(),
    DisputeRoundDeadlineReached(),
    DisputeRoundLimitReached(),
    DisputeRoundNotStarted(),
    WrongDisputeRoundState(),
    CanNotSwitchDisputeRound(),
    MajorityOfVotesNotReached(),
    NotRegisteredAsJuror(),
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
pub enum Verdict {
    None(),
    Negative(),
    Positive(),
}

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
pub enum Relation {
    Vote(),
    VerdictNegative(),
    VerdictNone(),
    VerdictPositive(),
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

    ///  Voting, only juror can do it.
    #[allow(dead_code, clippy::too_many_arguments)]
    pub fn vote(
        &self,
        dispute_id: u32,
        vote: [u64; 4],
        hash_of_all_votes: [u64; 4],
        proof: Vec<u8>,
    ) -> ink_wrapper_types::ExecCall {
        let data = {
            let mut data = vec![8, 59, 226, 96];
            dispute_id.encode_to(&mut data);
            vote.encode_to(&mut data);
            hash_of_all_votes.encode_to(&mut data);
            proof.encode_to(&mut data);
            data
        };
        ink_wrapper_types::ExecCall::new(self.account_id, data)
    }

    ///  Register as an active juror. Juries are picked
    ///  from this pool to participate in disputes.
    #[allow(dead_code, clippy::too_many_arguments)]
    pub fn register_as_an_active_juror(&self) -> ink_wrapper_types::ExecCall {
        let data = vec![80, 47, 210, 239];
        ink_wrapper_types::ExecCall::new(self.account_id, data)
    }

    ///  Unregister juror from the active juries pool.
    #[allow(dead_code, clippy::too_many_arguments)]
    pub fn unregister_as_an_active_juror(&self) -> ink_wrapper_types::ExecCall {
        let data = vec![217, 7, 103, 12];
        ink_wrapper_types::ExecCall::new(self.account_id, data)
    }

    ///  Assigned juror can confirm his participation in dispute
    #[allow(dead_code, clippy::too_many_arguments)]
    pub fn confirm_juror_participation_in_dispute(
        &self,
        dispute_id: u32,
        public_key: Vec<u8>,
    ) -> ink_wrapper_types::ExecCallNeedsValue {
        let data = {
            let mut data = vec![141, 200, 7, 55];
            dispute_id.encode_to(&mut data);
            public_key.encode_to(&mut data);
            data
        };
        ink_wrapper_types::ExecCallNeedsValue::new(self.account_id, data)
    }

    ///  Judge can confirm his participation in dispute
    #[allow(dead_code, clippy::too_many_arguments)]
    pub fn confirm_judge_participation_in_dispute(
        &self,
        dispute_id: u32,
        public_key: Vec<u8>,
    ) -> ink_wrapper_types::ExecCallNeedsValue {
        let data = {
            let mut data = vec![178, 215, 24, 15];
            dispute_id.encode_to(&mut data);
            public_key.encode_to(&mut data);
            data
        };
        ink_wrapper_types::ExecCallNeedsValue::new(self.account_id, data)
    }

    #[allow(dead_code, clippy::too_many_arguments)]
    pub fn issue_the_verdict(
        &self,
        dispute_id: u32,
        votes_maximum: u8,
        votes_minimum: u8,
        verdict: Verdict,
        hashed_votes: [u64; 4],
        jurors_banned: Vec<ink_primitives::AccountId>,
        proof: Vec<u8>,
    ) -> ink_wrapper_types::ExecCall {
        let data = {
            let mut data = vec![6, 210, 102, 105];
            dispute_id.encode_to(&mut data);
            votes_maximum.encode_to(&mut data);
            votes_minimum.encode_to(&mut data);
            verdict.encode_to(&mut data);
            hashed_votes.encode_to(&mut data);
            jurors_banned.encode_to(&mut data);
            proof.encode_to(&mut data);
            data
        };
        ink_wrapper_types::ExecCall::new(self.account_id, data)
    }

    ///  Unregister juror from the active juries pool.
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

    ///  Register a verification key.
    #[allow(dead_code, clippy::too_many_arguments)]
    pub fn register_vk(&self, relation: Relation, vk: Vec<u8>) -> ink_wrapper_types::ExecCall {
        let data = {
            let mut data = vec![165, 234, 12, 120];
            relation.encode_to(&mut data);
            vk.encode_to(&mut data);
            data
        };
        ink_wrapper_types::ExecCall::new(self.account_id, data)
    }

    ///  Get Juror/Judge public key if he is assigned to the dispute.
    #[allow(dead_code, clippy::too_many_arguments)]
    pub fn juror_public_key(
        &self,
        dispute_id: u32,
        juror_id: ink_primitives::AccountId,
    ) -> ink_wrapper_types::ReadCall<
        Result<Result<Vec<u8>, BrightDisputesError>, ink_wrapper_types::InkLangError>,
    > {
        let data = {
            let mut data = vec![198, 90, 15, 199];
            dispute_id.encode_to(&mut data);
            juror_id.encode_to(&mut data);
            data
        };
        ink_wrapper_types::ReadCall::new(self.account_id, data)
    }
}
