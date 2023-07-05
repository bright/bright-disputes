use scale::Encode as _;

// This file was auto-generated with ink-wrapper (https://crates.io/crates/ink-wrapper).

#[allow(dead_code)]
pub const CODE_HASH: [u8; 32] = [
    206, 40, 233, 94, 177, 91, 101, 22, 171, 11, 5, 89, 43, 242, 9, 87, 188, 185, 138, 211, 4, 63,
    59, 135, 90, 206, 90, 4, 10, 167, 122, 113,
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
pub async fn upload<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
    conn: &C,
) -> Result<TxInfo, E> {
    let wasm = include_bytes!("../contract/target/ink/bright_disputes.wasm");
    let tx_info = conn.upload((*wasm).into(), CODE_HASH.into()).await?;

    Ok(tx_info)
}

impl Instance {
    /// Constructor
    #[allow(dead_code, clippy::too_many_arguments)]
    pub async fn new<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        conn: &C,
        salt: Vec<u8>,
    ) -> Result<Self, E> {
        let data = vec![155, 174, 157, 94];
        let account_id = conn.instantiate(CODE_HASH, salt, data).await?;
        Ok(Self { account_id })
    }

    ///  Get last dispute id
    #[allow(dead_code, clippy::too_many_arguments)]
    pub async fn get_last_dispute_id<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<u32, ink_wrapper_types::InkLangError>, E> {
        let data = vec![241, 53, 223, 85];
        conn.read(self.account_id, data).await
    }

    ///  Get single dispute by id
    #[allow(dead_code, clippy::too_many_arguments)]
    pub async fn get_dispute<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
        dispute_id: u32,
    ) -> Result<Result<Result<Dispute, BrightDisputesError>, ink_wrapper_types::InkLangError>, E>
    {
        let data = {
            let mut data = vec![76, 253, 140, 199];
            dispute_id.encode_to(&mut data);
            data
        };
        conn.read(self.account_id, data).await
    }

    ///  Get all disputes
    #[allow(dead_code, clippy::too_many_arguments)]
    pub async fn get_all_disputes<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<Vec<Dispute>, ink_wrapper_types::InkLangError>, E> {
        let data = vec![29, 12, 242, 122];
        conn.read(self.account_id, data).await
    }

    ///  Get single dispute by id
    #[allow(dead_code, clippy::too_many_arguments)]
    pub async fn remove_dispute<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        dispute_id: u32,
    ) -> Result<TxInfo, E> {
        let data = {
            let mut data = vec![187, 47, 61, 10];
            dispute_id.encode_to(&mut data);
            data
        };
        conn.exec(self.account_id, data).await
    }

    ///  Create new dispute
    #[allow(dead_code, clippy::too_many_arguments)]
    pub async fn create_dispute<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        owner_link: String,
        defendant_id: ink_primitives::AccountId,
        escrow: u128,
    ) -> Result<TxInfo, E> {
        let data = {
            let mut data = vec![30, 25, 167, 107];
            owner_link.encode_to(&mut data);
            defendant_id.encode_to(&mut data);
            escrow.encode_to(&mut data);
            data
        };
        conn.exec(self.account_id, data).await
    }

    ///  Defendant confirms his participation in dispute.
    #[allow(dead_code, clippy::too_many_arguments)]
    pub async fn confirm_defendant<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        dispute_id: u32,
        defendant_link: String,
    ) -> Result<TxInfo, E> {
        let data = {
            let mut data = vec![164, 183, 46, 125];
            dispute_id.encode_to(&mut data);
            defendant_link.encode_to(&mut data);
            data
        };
        conn.exec(self.account_id, data).await
    }

    ///  Update owner link description
    #[allow(dead_code, clippy::too_many_arguments)]
    pub async fn update_owner_description<
        TxInfo,
        E,
        C: ink_wrapper_types::SignedConnection<TxInfo, E>,
    >(
        &self,
        conn: &C,
        dispute_id: u32,
        owner_link: String,
    ) -> Result<TxInfo, E> {
        let data = {
            let mut data = vec![68, 73, 200, 222];
            dispute_id.encode_to(&mut data);
            owner_link.encode_to(&mut data);
            data
        };
        conn.exec(self.account_id, data).await
    }

    ///  Update defendant link description
    #[allow(dead_code, clippy::too_many_arguments)]
    pub async fn update_defendant_description<
        TxInfo,
        E,
        C: ink_wrapper_types::SignedConnection<TxInfo, E>,
    >(
        &self,
        conn: &C,
        dispute_id: u32,
        defendant_link: String,
    ) -> Result<TxInfo, E> {
        let data = {
            let mut data = vec![208, 24, 65, 84];
            dispute_id.encode_to(&mut data);
            defendant_link.encode_to(&mut data);
            data
        };
        conn.exec(self.account_id, data).await
    }

    ///  Voting, only jure can do it.
    #[allow(dead_code, clippy::too_many_arguments)]
    pub async fn vote<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        dispute_id: u32,
        vote: u8,
    ) -> Result<TxInfo, E> {
        let data = {
            let mut data = vec![8, 59, 226, 96];
            dispute_id.encode_to(&mut data);
            vote.encode_to(&mut data);
            data
        };
        conn.exec(self.account_id, data).await
    }

    ///  Register as an active jure. Juries are picked
    ///  from this pool to participate in disputes.
    #[allow(dead_code, clippy::too_many_arguments)]
    pub async fn register_as_an_active_jure<
        TxInfo,
        E,
        C: ink_wrapper_types::SignedConnection<TxInfo, E>,
    >(
        &self,
        conn: &C,
    ) -> Result<TxInfo, E> {
        let data = vec![121, 6, 115, 245];
        conn.exec(self.account_id, data).await
    }

    ///  Unregister jure from the active juries pool.
    #[allow(dead_code, clippy::too_many_arguments)]
    pub async fn unregister_as_an_active_jure<
        TxInfo,
        E,
        C: ink_wrapper_types::SignedConnection<TxInfo, E>,
    >(
        &self,
        conn: &C,
    ) -> Result<TxInfo, E> {
        let data = vec![121, 53, 33, 150];
        conn.exec(self.account_id, data).await
    }

    ///  Assigned jure can confirm his participation in dispute
    #[allow(dead_code, clippy::too_many_arguments)]
    pub async fn confirm_jure_participation_in_dispute<
        TxInfo,
        E,
        C: ink_wrapper_types::SignedConnection<TxInfo, E>,
    >(
        &self,
        conn: &C,
        dispute_id: u32,
    ) -> Result<TxInfo, E> {
        let data = {
            let mut data = vec![149, 58, 196, 227];
            dispute_id.encode_to(&mut data);
            data
        };
        conn.exec(self.account_id, data).await
    }

    ///  Judge can confirm his participation in dispute
    #[allow(dead_code, clippy::too_many_arguments)]
    pub async fn confirm_judge_participation_in_dispute<
        TxInfo,
        E,
        C: ink_wrapper_types::SignedConnection<TxInfo, E>,
    >(
        &self,
        conn: &C,
        dispute_id: u32,
    ) -> Result<TxInfo, E> {
        let data = {
            let mut data = vec![178, 215, 24, 15];
            dispute_id.encode_to(&mut data);
            data
        };
        conn.exec(self.account_id, data).await
    }

    ///  Unregister jure from the active juries pool.
    #[allow(dead_code, clippy::too_many_arguments)]
    pub async fn process_dispute_round<
        TxInfo,
        E,
        C: ink_wrapper_types::SignedConnection<TxInfo, E>,
    >(
        &self,
        conn: &C,
        dispute_id: u32,
    ) -> Result<TxInfo, E> {
        let data = {
            let mut data = vec![14, 13, 134, 23];
            dispute_id.encode_to(&mut data);
            data
        };
        conn.exec(self.account_id, data).await
    }

    ///  Judge can confirm his participation in dispute
    #[allow(dead_code, clippy::too_many_arguments)]
    pub async fn distribute_deposit<
        TxInfo,
        E,
        C: ink_wrapper_types::SignedConnection<TxInfo, E>,
    >(
        &self,
        conn: &C,
        dispute_id: u32,
    ) -> Result<TxInfo, E> {
        let data = {
            let mut data = vec![117, 233, 246, 239];
            dispute_id.encode_to(&mut data);
            data
        };
        conn.exec(self.account_id, data).await
    }
}
