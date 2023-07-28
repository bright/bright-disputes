use ink::{prelude::vec::Vec, primitives::AccountId};

use crate::{
    dispute::Dispute,
    error::BrightDisputesError,
    juror::JuriesMap,
    types::{Result, Timestamp},
};

#[derive(Clone, Debug, PartialEq, scale::Decode, scale::Encode)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout, scale_info::TypeInfo)
)]
pub enum RoundState {
    AssignJuriesAndJudge,
    PickingJuriesAndJudge,
    Voting,
    CountingTheVotes,
}

#[derive(Clone, Debug, PartialEq, scale::Decode, scale::Encode)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout, scale_info::TypeInfo)
)]
pub struct DisputeRound {
    state: RoundState,
    number_of_juries: u8,
    state_deadline: Timestamp,
}

impl DisputeRound {
    const ASSIGN_JURIES_AND_JUDGE_TIME: u64 = 2;
    const PICKING_JURIES_AND_JUDGE_TIME: u64 = 2;
    const VOTING_TIME: u64 = 4;
    const VOTING_COUNTING: u64 = 1;
    const INITIAL_NUMBER_OF_JURIES: u8 = 3;

    /// Creates new dispute round.
    pub fn create(timestamp: Timestamp, number_of_juries: Option<u8>) -> Self {
        DisputeRound {
            state: RoundState::AssignJuriesAndJudge,
            number_of_juries: number_of_juries.unwrap_or(DisputeRound::INITIAL_NUMBER_OF_JURIES),
            state_deadline: Self::deadline(timestamp, Self::ASSIGN_JURIES_AND_JUDGE_TIME),
        }
    }

    /// Try to switch to the next dispute round, It also check deadlines.
    pub fn process_dispute_round(
        &mut self,
        contract: &mut dyn JuriesMap,
        dispute: &mut Dispute,
        now: Timestamp,
    ) -> Result<()> {
        if let Err(e) = self.try_to_switch_to_next_state(contract, dispute, now) {
            if e != BrightDisputesError::MajorityOfVotesNotReached {
                if now >= self.state_deadline {
                    return Err(BrightDisputesError::DisputeRoundDeadlineReached);
                }
            }
            return Err(e);
        }
        Ok(())
    }

    // Assert when state is not in "Voting" state.
    pub fn assert_if_not_voting_time(&self) -> Result<()> {
        if self.state != RoundState::Voting {
            return Err(BrightDisputesError::WrongDisputeRoundState);
        }
        Ok(())
    }

    fn try_to_switch_to_next_state(
        &mut self,
        contract: &mut dyn JuriesMap,
        dispute: &mut Dispute,
        now: Timestamp,
    ) -> Result<()> {
        match self.state {
            RoundState::AssignJuriesAndJudge => {
                self.handle_assigning_the_juries(contract, dispute)?;
                self.handle_assigning_judge(contract, dispute)?;

                self.state = RoundState::PickingJuriesAndJudge;
                self.state_deadline = Self::deadline(now, Self::PICKING_JURIES_AND_JUDGE_TIME);
            }
            RoundState::PickingJuriesAndJudge => {
                self.handle_picking_the_juries(contract, dispute)?;
                self.handle_picking_the_judge(contract, dispute)?;

                // Request juries to vote
                for juror_id in dispute.juries() {
                    let mut juror = contract.get_juror_or_assert(juror_id)?;
                    juror.request_for_action(dispute.id())?;
                    contract.update_juror(juror);
                }

                self.state = RoundState::Voting;
                self.state_deadline = Self::deadline(now, Self::VOTING_TIME);
            }
            RoundState::Voting => self.handle_voting(contract, dispute, now)?,
            RoundState::CountingTheVotes => self.handle_votes_counting(contract, dispute)?,
        };
        Ok(())
    }

    fn handle_assigning_the_juries(
        &mut self,
        contract: &mut dyn JuriesMap,
        dispute: &mut Dispute,
    ) -> Result<()> {
        dispute.assert_owner_call()?;
        if self.state != RoundState::AssignJuriesAndJudge {
            return Err(BrightDisputesError::WrongDisputeRoundState);
        }

        let extend_juries_by = self.number_of_juries - dispute.juries().len() as u8;
        let mut banned_accounts = dispute.banned();
        banned_accounts.extend([dispute.owner(), dispute.defendant()].to_vec());
        let juries_ids = contract
            .remove_random_juries_from_pool_or_assert(&banned_accounts, extend_juries_by)?;

        for juror_id in juries_ids {
            let mut juror = contract.get_juror_or_assert(juror_id)?;
            dispute.assign_juror(&mut juror)?;
            contract.update_juror(juror);
        }

        Ok(())
    }

    fn handle_assigning_judge(
        &mut self,
        contract: &mut dyn JuriesMap,
        dispute: &mut Dispute,
    ) -> Result<()> {
        dispute.assert_owner_call()?;
        if self.state != RoundState::AssignJuriesAndJudge {
            return Err(BrightDisputesError::WrongDisputeRoundState);
        } else if dispute.judge().is_none() {
            let mut banned_accounts = dispute.banned();
            banned_accounts.extend([dispute.owner(), dispute.defendant()].to_vec());
            let judge_id =
                contract.remove_random_juries_from_pool_or_assert(&banned_accounts, 1)?;

            let mut juror = contract.get_juror_or_assert(judge_id[0])?;
            dispute.assign_judge(&mut juror)?;
            contract.update_juror(juror);
        }
        Ok(())
    }

    fn handle_picking_the_juries(
        &mut self,
        contract: &dyn JuriesMap,
        dispute: &Dispute,
    ) -> Result<()> {
        dispute.assert_owner_call()?;
        if self.state != RoundState::PickingJuriesAndJudge {
            return Err(BrightDisputesError::WrongDisputeRoundState);
        } else if dispute.juries().is_empty() {
            return Err(BrightDisputesError::CanNotSwitchDisputeRound);
        }

        let dispute_id = dispute.id();
        for juror_id in dispute.juries() {
            let juror = contract.get_juror_or_assert(juror_id)?;
            if !juror.is_confirmed(dispute_id) {
                return Err(BrightDisputesError::CanNotSwitchDisputeRound);
            }
        }
        Ok(())
    }

    fn handle_picking_the_judge(
        &mut self,
        contract: &dyn JuriesMap,
        dispute: &Dispute,
    ) -> Result<()> {
        dispute.assert_owner_call()?;
        if self.state != RoundState::PickingJuriesAndJudge {
            return Err(BrightDisputesError::WrongDisputeRoundState);
        } else if dispute.judge().is_none() {
            return Err(BrightDisputesError::CanNotSwitchDisputeRound);
        }

        let judge = contract.get_juror_or_assert(dispute.judge().unwrap())?;
        if !judge.is_confirmed(dispute.id()) {
            return Err(BrightDisputesError::CanNotSwitchDisputeRound);
        }
        Ok(())
    }

    fn handle_voting(
        &mut self,
        contract: &mut dyn JuriesMap,
        dispute: &Dispute,
        timestamp: Timestamp,
    ) -> Result<()> {
        dispute.assert_owner_call()?;
        if self.state != RoundState::Voting {
            return Err(BrightDisputesError::WrongDisputeRoundState);
        } else if dispute.votes().is_empty() {
            return Err(BrightDisputesError::JuriesNotVoted(dispute.juries()));
        }

        let not_voted: Vec<AccountId> = dispute.get_not_voted_juries();
        if !not_voted.is_empty() {
            return Err(BrightDisputesError::JuriesNotVoted(not_voted));
        }

        // Request judge to count the votes.
        let judge_id = dispute.judge().unwrap();
        let mut judge = contract.get_juror_or_assert(judge_id)?;
        judge.request_for_action(dispute.id())?;
        contract.update_juror(judge);

        self.state = RoundState::CountingTheVotes;
        self.state_deadline = Self::deadline(timestamp, Self::VOTING_COUNTING);

        Ok(())
    }

    fn handle_votes_counting(
        &mut self,
        contract: &mut dyn JuriesMap,
        dispute: &mut Dispute,
    ) -> Result<()> {
        if self.state != RoundState::CountingTheVotes {
            return Err(BrightDisputesError::WrongDisputeRoundState);
        }
        dispute.assert_judge()?;

        // Mark judge work as done.
        let judge_id = dispute.judge().unwrap();
        let mut judge = contract.get_juror_or_assert(judge_id)?;
        judge.action_done(dispute.id())?;
        contract.update_juror(judge);

        let result = dispute.count_votes()?;
        dispute.end_dispute(result)?;

        Ok(())
    }

    fn deadline(begin: Timestamp, days: u64) -> Timestamp {
        begin + days * 24 * 3600 * 1000
    }
}

#[cfg(test)]
pub mod mock {
    use super::*;

    pub struct DisputeRoundFake {}

    impl DisputeRoundFake {
        pub fn voting(state_deadline: Timestamp) -> DisputeRound {
            DisputeRound {
                state: RoundState::Voting,
                number_of_juries: DisputeRound::INITIAL_NUMBER_OF_JURIES,
                state_deadline,
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use ink::env::{test::set_caller, DefaultEnvironment};

    use super::*;
    use crate::{
        dispute::DisputeResult,
        juror::{mock::JuriesMapMock, Juror},
        vote::Vote,
    };

    #[ink::test]
    fn create_dispute_round() {
        let round = DisputeRound::create(0u64, None);
        assert_eq!(round.state, RoundState::AssignJuriesAndJudge);
    }

    #[ink::test]
    fn handle_assigning_the_juries() {
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        set_caller::<DefaultEnvironment>(accounts.alice);

        // Create a dispute
        let mut dispute = Dispute::create(
            1,
            "https://brightinventions.pl/owner".into(),
            accounts.bob,
            10,
        );

        // Confirm defendant participation
        set_caller::<DefaultEnvironment>(accounts.bob);
        dispute
            .confirm_defendant("".into())
            .expect("Failed to confirm defendant a dispute!");

        // Create a dispute round.
        let start_timestamp = 0u64;
        let picking_judge_deadline =
            DisputeRound::deadline(start_timestamp, DisputeRound::ASSIGN_JURIES_AND_JUDGE_TIME);
        let mut round = DisputeRound::create(start_timestamp, None);

        let mut juries = JuriesMapMock::create_vec(vec![
            Juror::create(accounts.charlie),
            Juror::create(accounts.eve),
        ]);

        // Failed to switch, deadline reached!
        set_caller::<DefaultEnvironment>(accounts.alice);
        let result = round.process_dispute_round(&mut juries, &mut dispute, picking_judge_deadline);
        assert_eq!(
            result,
            Err(BrightDisputesError::DisputeRoundDeadlineReached)
        );

        let mut juries = JuriesMapMock::create_vec(vec![
            Juror::create(accounts.charlie),
            Juror::create(accounts.eve),
            Juror::create(accounts.frank),
            Juror::create(accounts.django),
        ]);

        // Failed, only owner can switch the state
        set_caller::<DefaultEnvironment>(accounts.bob);
        let result = round.process_dispute_round(&mut juries, &mut dispute, start_timestamp);
        assert_eq!(result, Err(BrightDisputesError::NotAuthorized));

        // Success
        set_caller::<DefaultEnvironment>(accounts.alice);
        let result = round.process_dispute_round(&mut juries, &mut dispute, start_timestamp);
        assert_eq!(result, Ok(()));
        assert_eq!(round.state, RoundState::PickingJuriesAndJudge);
    }

    #[ink::test]
    fn handle_picking_the_judge_and_juries() {
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        set_caller::<DefaultEnvironment>(accounts.alice);

        // Create a dispute
        let mut dispute = Dispute::create(
            1,
            "https://brightinventions.pl/owner".into(),
            accounts.bob,
            10,
        );

        // Confirm defendant participation
        set_caller::<DefaultEnvironment>(accounts.bob);
        dispute
            .confirm_defendant("".into())
            .expect("Failed to confirm defendant a dispute!");

        // Create a dispute round.
        let start_timestamp = 0u64;
        let mut round = DisputeRound::create(start_timestamp, None);

        let mut juries = JuriesMapMock::create_vec(vec![
            Juror::create(accounts.charlie),
            Juror::create(accounts.eve),
            Juror::create(accounts.frank),
            Juror::create(accounts.django),
        ]);

        // Assign juries and judge to dispute
        set_caller::<DefaultEnvironment>(accounts.alice);
        round
            .process_dispute_round(&mut juries, &mut dispute, start_timestamp)
            .expect("Failed to assign juries and judge!");

        for juror in juries.iter_mut() {
            juror.confirm_participation_in_dispute(1)
                .expect("Unable confirm juror participation in dispute!");
        }

        // Failed, only owner can switch the state
        set_caller::<DefaultEnvironment>(accounts.bob);
        let result = round.process_dispute_round(&mut juries, &mut dispute, start_timestamp);
        assert_eq!(result, Err(BrightDisputesError::NotAuthorized));

        // Success
        set_caller::<DefaultEnvironment>(accounts.alice);
        let result = round.process_dispute_round(&mut juries, &mut dispute, start_timestamp);
        assert_eq!(result, Ok(()));
        assert_eq!(round.state, RoundState::Voting);
    }

    #[ink::test]
    fn process_dispute_round_handle_voting() {
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        set_caller::<DefaultEnvironment>(accounts.alice);

        // Create a dispute
        let mut dispute = Dispute::create(
            1,
            "https://brightinventions.pl/owner".into(),
            accounts.bob,
            10,
        );

        // Confirm defendant participation
        set_caller::<DefaultEnvironment>(accounts.bob);
        dispute
            .confirm_defendant("".into())
            .expect("Failed to confirm defendant a dispute!");

        let start_timestamp = 0u64;
        let mut round = DisputeRound::create(start_timestamp, None);

        let mut juries = JuriesMapMock::create_vec(vec![
            Juror::create(accounts.charlie),
            Juror::create(accounts.eve),
            Juror::create(accounts.frank),
            Juror::create(accounts.django),
        ]);

        set_caller::<DefaultEnvironment>(accounts.alice);

        // Assign juries and judge to dispute
        round
            .process_dispute_round(&mut juries, &mut dispute, start_timestamp)
            .expect("Failed to assign juries and judge!");

        for juror in juries.iter_mut() {
            juror.confirm_participation_in_dispute(1)
                .expect("Unable confirm juror participation in dispute!");
        }

        // Move to voting state
        round
            .process_dispute_round(&mut juries, &mut dispute, start_timestamp)
            .expect("Failed move voting state");

        // Failed, only owner can switch the state
        set_caller::<DefaultEnvironment>(accounts.bob);
        let result = round.process_dispute_round(&mut juries, &mut dispute, start_timestamp);
        assert_eq!(result, Err(BrightDisputesError::NotAuthorized));

        // Juries not voted
        set_caller::<DefaultEnvironment>(accounts.alice);
        let result = round.process_dispute_round(&mut juries, &mut dispute, start_timestamp);
        assert_eq!(
            result,
            Err(BrightDisputesError::JuriesNotVoted(dispute.juries()))
        );
        assert_eq!(round.state, RoundState::Voting);

        // Before voting, we need to update dispute round
        dispute.set_dispute_round(round.clone());

        for juror_id in dispute.juries() {
            set_caller::<DefaultEnvironment>(juror_id);
            dispute
                .vote(Vote::create(juror_id, 1))
                .expect("Failed make a vote!");
        }

        // Success
        set_caller::<DefaultEnvironment>(accounts.alice);
        let result = round.process_dispute_round(&mut juries, &mut dispute, start_timestamp);
        assert_eq!(result, Ok(()));
        assert_eq!(round.state, RoundState::CountingTheVotes);
    }

    #[ink::test]
    fn process_dispute_round_handle_votes_counting() {
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        set_caller::<DefaultEnvironment>(accounts.alice);

        // Create a dispute
        let mut dispute = Dispute::create(
            1,
            "https://brightinventions.pl/owner".into(),
            accounts.bob,
            10,
        );

        // Confirm defendant participation
        set_caller::<DefaultEnvironment>(accounts.bob);
        dispute
            .confirm_defendant("".into())
            .expect("Failed to confirm defendant a dispute!");

        let start_timestamp = 0u64;
        let mut round = DisputeRound::create(start_timestamp, None);

        let mut juries = JuriesMapMock::create_vec(vec![
            Juror::create(accounts.charlie),
            Juror::create(accounts.eve),
            Juror::create(accounts.frank),
            Juror::create(accounts.django),
        ]);

        set_caller::<DefaultEnvironment>(accounts.alice);

        // Assign juries and judge to dispute
        round
            .process_dispute_round(&mut juries, &mut dispute, start_timestamp)
            .expect("Failed to assign juries and judge!");

        for juror in juries.iter_mut() {
            juror.confirm_participation_in_dispute(1)
                .expect("Unable confirm juror participation in dispute!");
        }

        // Move to voting state
        round
            .process_dispute_round(&mut juries, &mut dispute, start_timestamp)
            .expect("Failed move voting state");

        // Before voting, we need to update dispute round
        dispute.set_dispute_round(round.clone());

        let voters = dispute.juries();
        dispute
            .vote(Vote::create(voters[0], 1))
            .expect("Failed make a vote!");
        dispute
            .vote(Vote::create(voters[1], 1))
            .expect("Failed make a vote!");
        dispute
            .vote(Vote::create(voters[2], 1))
            .expect("Failed make a vote!");

        // Move to counting the votes state.
        round
            .process_dispute_round(&mut juries, &mut dispute, start_timestamp)
            .expect("Failed to move to counting votes state.");

        // Failed, only judge can process this state
        set_caller::<DefaultEnvironment>(accounts.alice);
        let result = round.process_dispute_round(&mut juries, &mut dispute, start_timestamp);
        assert_eq!(result, Err(BrightDisputesError::NotAuthorized));

        // Success
        set_caller::<DefaultEnvironment>(dispute.judge().unwrap());
        assert_eq!(dispute.get_dispute_result(), None);
        let result = round.process_dispute_round(&mut juries, &mut dispute, start_timestamp);
        assert_eq!(result, Ok(()));
        assert_eq!(dispute.get_dispute_result(), Some(DisputeResult::Owner));
    }

    #[ink::test]
    fn process_dispute_round_handle_votes_counting_majority_not_reached() {
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        set_caller::<DefaultEnvironment>(accounts.alice);

        // Create a dispute
        let mut dispute = Dispute::create(
            1,
            "https://brightinventions.pl/owner".into(),
            accounts.bob,
            10,
        );

        // Confirm defendant participation
        set_caller::<DefaultEnvironment>(accounts.bob);
        dispute
            .confirm_defendant("".into())
            .expect("Failed to confirm defendant a dispute!");

        let start_timestamp = 0u64;
        let mut round = DisputeRound::create(start_timestamp, None);

        let mut juries = JuriesMapMock::create_vec(vec![
            Juror::create(accounts.charlie),
            Juror::create(accounts.eve),
            Juror::create(accounts.frank),
            Juror::create(accounts.django),
        ]);

        set_caller::<DefaultEnvironment>(accounts.alice);

        // Assign juries and judge to dispute
        round
            .process_dispute_round(&mut juries, &mut dispute, start_timestamp)
            .expect("Failed to assign juries and judge!");

        for juror in juries.iter_mut() {
            juror.confirm_participation_in_dispute(1)
                .expect("Unable confirm juror participation in dispute!");
        }

        // Move to voting state
        round
            .process_dispute_round(&mut juries, &mut dispute, start_timestamp)
            .expect("Failed move voting state");

        // Before voting, we need to update dispute round
        dispute.set_dispute_round(round.clone());

        let voters = dispute.juries();
        dispute
            .vote(Vote::create(voters[0], 1))
            .expect("Failed make a vote!");
        dispute
            .vote(Vote::create(voters[1], 1))
            .expect("Failed make a vote!");
        dispute
            .vote(Vote::create(voters[2], 0))
            .expect("Failed make a vote!");

        // Move to counting the votes state.
        round
            .process_dispute_round(&mut juries, &mut dispute, start_timestamp)
            .expect("Failed to move to counting votes state.");

        // Failed, only judge can process this state
        set_caller::<DefaultEnvironment>(accounts.alice);
        let result = round.process_dispute_round(&mut juries, &mut dispute, start_timestamp);
        assert_eq!(result, Err(BrightDisputesError::NotAuthorized));

        // Success
        set_caller::<DefaultEnvironment>(dispute.judge().unwrap());
        let result = round.process_dispute_round(&mut juries, &mut dispute, start_timestamp);
        assert_eq!(result, Err(BrightDisputesError::MajorityOfVotesNotReached));
    }

    #[ink::test]
    fn assert_if_not_voting_time() {
        let mut round = DisputeRound::create(0u64, None);

        let result = round.assert_if_not_voting_time();
        assert_eq!(result, Err(BrightDisputesError::WrongDisputeRoundState));

        // Force Voting state
        round.state = RoundState::Voting;

        let result = round.assert_if_not_voting_time();
        assert_eq!(result, Ok(()));
    }
}
