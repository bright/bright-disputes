use core::ops::Mul;

use ink::{
    prelude::{string::String, vec::Vec},
    primitives::AccountId,
};

use crate::{
    dispute_round::DisputeRound,
    error::BrightDisputesError,
    juror::{Juror, JuriesMap},
    types::{Balance, DisputeId, Result, Timestamp},
    vote::Vote,
};

#[derive(Clone, Debug, PartialEq, scale::Decode, scale::Encode)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout, scale_info::TypeInfo)
)]
pub enum DisputeState {
    Created,
    Running,
    Ended,
    Closed,
}

#[derive(Clone, Debug, PartialEq, scale::Decode, scale::Encode)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout, scale_info::TypeInfo)
)]
pub enum DisputeResult {
    Owner,
    Defendant,
}

impl DisputeResult {
    pub fn opposite(&self) -> DisputeResult {
        if *self == DisputeResult::Owner {
            return DisputeResult::Defendant;
        }
        return DisputeResult::Owner;
    }
}

#[derive(Clone, Debug, PartialEq, scale::Decode, scale::Encode)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout, scale_info::TypeInfo)
)]
pub struct Dispute {
    id: DisputeId,
    state: DisputeState,
    owner: AccountId,
    owner_link: String,
    escrow: Balance,
    deposit: Balance,

    defendant: AccountId,
    defendant_link: Option<String>,
    dispute_result: Option<DisputeResult>,
    dispute_round: Option<DisputeRound>,
    dispute_round_counter: u8,

    judge: Option<AccountId>,
    juries: Vec<AccountId>,
    banned: Vec<AccountId>,
    votes: Vec<Vote>,
}

impl Dispute {
    const MAX_DISPUTE_ROUNDS: u8 = 3u8;
    const WINING_MAJORITY: usize = 75;
    const LOSING_MAJORITY: usize = 100 - Dispute::WINING_MAJORITY;
    const INCREMENT_JURIES_BY: u8 = 2;

    /// Creates a new dispute
    pub fn create(
        id: DisputeId,
        owner_link: String,
        defendant: AccountId,
        escrow: Balance,
    ) -> Self {
        Dispute {
            id,
            state: DisputeState::Created,
            owner: ink::env::caller::<ink::env::DefaultEnvironment>(),
            owner_link,
            escrow,
            deposit: escrow,
            defendant,
            defendant_link: None,
            dispute_result: None,
            dispute_round: None,
            dispute_round_counter: 0u8,
            judge: None,
            juries: Vec::new(),
            banned: Vec::new(),
            votes: Vec::new(),
        }
    }

    /// Get dispute id
    pub fn id(&self) -> DisputeId {
        return self.id;
    }

    /// Get dispute id
    pub fn owner(&self) -> AccountId {
        return self.owner;
    }

    /// Get dispute id
    pub fn defendant(&self) -> AccountId {
        return self.defendant;
    }

    /// Get dispute escrow
    pub fn escrow(&self) -> Balance {
        return self.escrow;
    }

    /// Get dispute escrow
    pub fn deposit(&self) -> Balance {
        return self.deposit;
    }

    /// Get judge
    pub fn judge(&self) -> Option<AccountId> {
        return self.judge.clone();
    }

    /// Get dispute round
    pub fn dispute_round(&self) -> Option<DisputeRound> {
        return self.dispute_round.clone();
    }

    /// Get juries
    pub fn juries(&self) -> Vec<AccountId> {
        return self.juries.clone();
    }

    /// Get banned
    pub fn banned(&self) -> Vec<AccountId> {
        return self.banned.clone();
    }

    /// Get banned
    pub fn votes(&self) -> Vec<Vote> {
        return self.votes.clone();
    }

    /// Getter dispute result
    pub fn get_dispute_result(&self) -> Option<DisputeResult> {
        self.dispute_result.clone()
    }

    /// Set owner decription link
    pub fn set_owner_link(&mut self, owner_link: String) -> Result<()> {
        self.assert_owner_call()?;
        self.owner_link = owner_link;
        Ok(())
    }

    /// Set defendant description link
    pub fn set_defendant_link(&mut self, defendant_link: String) -> Result<()> {
        self.assert_defendant_call()?;
        self.defendant_link = Some(defendant_link);
        Ok(())
    }

    /// Check if defendant confirmed a dispute
    pub fn has_defendant_confirmed_dispute(&self) -> bool {
        self.defendant_link.is_some()
    }

    /// Start new dispute round
    pub fn set_dispute_round(&mut self, dispute: DisputeRound) {
        self.dispute_round = Some(dispute);
    }

    /// Increment a dispute deposit by an escrow
    pub fn increment_deposit(&mut self) {
        self.deposit += self.escrow;
    }

    /// End the dispute and publish result.
    pub fn end_dispute(&mut self, result: DisputeResult) -> Result<()> {
        self.assert_state(DisputeState::Running)?;
        if self.dispute_result.is_some() {
            return Err(BrightDisputesError::InvalidAction);
        }

        self.state = DisputeState::Ended;
        self.dispute_result = Some(result.clone());
        self.dispute_round = None;

        // Move juries who wrongly voted to banned list
        let juries_to_ban = self.get_juries_voted_against(result.opposite());
        for juror in juries_to_ban {
            self.move_to_banned(juror)
                .expect("Juror is not assigned to the dispute!");
        }
        Ok(())
    }

    /// Close the dispute.
    pub fn close_dispute(&mut self) -> Result<()> {
        self.assert_state(DisputeState::Ended)?;
        self.state = DisputeState::Closed;
        Ok(())
    }

    /// Confirm defendant participation in dispute
    pub fn confirm_defendant(&mut self, defendant_link: String) -> Result<()> {
        self.assert_defendant_call()?;
        self.assert_state(DisputeState::Created)?;
        self.defendant_link = Some(defendant_link);
        self.state = DisputeState::Running;
        Ok(())
    }

    /// Make a vote
    pub fn vote(&mut self, vote: Vote) -> Result<()> {
        self.assert_state(DisputeState::Running)?;
        self.assert_can_vote()?;
        self.assert_juror(vote.juror())?;
        self.assert_not_voted(vote.juror())?;
        self.votes.push(vote);
        Ok(())
    }

    /// Count the votes and return the result.
    pub fn count_votes(&self) -> Result<DisputeResult> {
        self.assert_state(DisputeState::Running)?;
        self.assert_judge()?;

        let total_votes = self.votes.len();
        if total_votes != self.juries.len() || self.votes.is_empty() {
            return Err(BrightDisputesError::MajorityOfVotesNotReached);
        }

        let owner_votes = self
            .votes
            .iter()
            .filter(|&v| v.is_vote_against_owner())
            .count()
            .mul(100);

        if let Some(res) = owner_votes.checked_div(total_votes) {
            if res >= Dispute::WINING_MAJORITY {
                return Ok(DisputeResult::Owner);
            } else if res <= Dispute::LOSING_MAJORITY {
                return Ok(DisputeResult::Defendant);
            }
        }
        Err(BrightDisputesError::MajorityOfVotesNotReached)
    }

    /// Assign juror to the dispute
    pub fn assign_juror(&mut self, juror: &mut Juror) -> Result<()> {
        self.assert_state(DisputeState::Running)?;
        self.assert_not_juror(juror.id())?;
        self.assert_not_owner_call(juror.id())?;
        self.assert_not_defendant_call(juror.id())?;
        juror.assign_to_dispute(self.id)?;
        self.juries.push(juror.id());
        Ok(())
    }

    /// Assign judge to the dispute
    pub fn assign_judge(&mut self, judge: &mut Juror) -> Result<()> {
        self.assert_state(DisputeState::Running)?;
        self.assert_not_owner_call(judge.id())?;
        self.assert_not_defendant_call(judge.id())?;
        if self.judge.is_some() {
            return Err(BrightDisputesError::JudgeAlreadyAssignedToDispute);
        }
        if self.juries().contains(&judge.id()) {
            return Err(BrightDisputesError::JurorAlreadyAssignedToDispute);
        }
        judge.assign_to_dispute(self.id)?;
        self.judge = Some(judge.id());
        Ok(())
    }

    /// Move juror / judge to banned list
    pub fn move_to_banned(&mut self, account_id: AccountId) -> Result<()> {
        if self.judge.is_some() && (self.judge.unwrap() == account_id) {
            self.judge = None;
            self.banned.push(account_id);
        } else if let Some(index) = self.juries.iter().position(|&id| id == account_id) {
            self.juries.remove(index);
            self.banned.push(account_id);
        } else {
            return Err(BrightDisputesError::InvalidAction);
        }
        Ok(())
    }

    /// Handle dispute round deadline
    pub fn on_dispute_round_deadline(&mut self, timestamp: Timestamp) -> Result<()> {
        self.assert_state(DisputeState::Running)?;

        // Clear votes.
        self.votes.clear();

        // Set new dispute round.
        self.dispute_round = Some(DisputeRound::create(timestamp, None));
        Ok(())
    }

    /// Start new dispute round
    pub fn next_dispute_round(&mut self, timestamp: Timestamp) -> Result<()> {
        self.assert_state(DisputeState::Running)?;
        if self.dispute_round_counter >= Dispute::MAX_DISPUTE_ROUNDS {
            return Err(BrightDisputesError::DisputeRoundLimitReached);
        }

        // Clear votes.
        self.votes.clear();

        // Increase the number of juries for the next round.
        let number_of_juries: u8 = self.juries.len() as u8 + Dispute::INCREMENT_JURIES_BY;

        // Set new dispute round
        self.dispute_round = Some(DisputeRound::create(timestamp, Some(number_of_juries)));
        self.dispute_round_counter += 1;

        Ok(())
    }

    /// Handle dispute rounds.
    pub fn process_dispute_round(
        &mut self,
        contract: &mut dyn JuriesMap,
        timestamp: Timestamp,
    ) -> Result<()> {
        self.assert_state(DisputeState::Running)?;
        if let Some(mut round) = self.dispute_round.clone() {
            round.process_dispute_round(contract, self, timestamp)?;
            self.dispute_round = Some(round);
            return Ok(());
        }
        Err(BrightDisputesError::DisputeRoundNotStarted)
    }

    /// Get juries who have not voted.
    pub fn get_not_voted_juries(&self) -> Vec<AccountId> {
        self.juries()
            .iter()
            .filter(|&id| self.votes().iter().position(|v| v.juror() == *id).is_none())
            .map(|&id| id)
            .collect()
    }

    /// Get juries who vote against one of the sides.
    fn get_juries_voted_against(&self, dispute_result: DisputeResult) -> Vec<AccountId> {
        self.votes()
            .iter()
            .filter(|&vote| {
                (dispute_result == DisputeResult::Owner && vote.is_vote_against_owner())
                    || (dispute_result == DisputeResult::Defendant && !vote.is_vote_against_owner())
            })
            .map(|vote| vote.juror())
            .collect()
    }

    /// Assert if call is done not by the owner
    pub fn assert_owner_call(&self) -> Result<()> {
        if self.owner != ink::env::caller::<ink::env::DefaultEnvironment>() {
            return Err(BrightDisputesError::NotAuthorized);
        }
        Ok(())
    }

    /// Assert if call is done not by the judge
    pub fn assert_judge(&self) -> Result<()> {
        if let Some(judge) = self.judge {
            if judge == ink::env::caller::<ink::env::DefaultEnvironment>() {
                return Ok(());
            }
        }
        return Err(BrightDisputesError::NotAuthorized);
    }

    /// Assert if dispute is not ended.
    pub fn assert_dispute_ended(&self) -> Result<()> {
        if self.state != DisputeState::Ended
            && self.dispute_round_counter < Dispute::MAX_DISPUTE_ROUNDS
        {
            return Err(BrightDisputesError::InvalidDisputeState);
        }
        Ok(())
    }

    /// Assert if dispute can not be removed.
    pub fn assert_dispute_remove(&self) -> Result<()> {
        if self.state == DisputeState::Created || self.state == DisputeState::Closed {
            return Ok(());
        }
        return Err(BrightDisputesError::InvalidDisputeState);
    }

    fn assert_defendant_call(&self) -> Result<()> {
        if self.defendant != ink::env::caller::<ink::env::DefaultEnvironment>() {
            return Err(BrightDisputesError::NotAuthorized);
        }
        Ok(())
    }

    fn assert_not_owner_call(&self, account: AccountId) -> Result<()> {
        if self.owner == account {
            return Err(BrightDisputesError::NotAuthorized);
        }
        Ok(())
    }

    fn assert_not_defendant_call(&self, account: AccountId) -> Result<()> {
        if self.defendant == account {
            return Err(BrightDisputesError::NotAuthorized);
        }
        Ok(())
    }

    fn assert_state(&self, state: DisputeState) -> Result<()> {
        if self.state != state {
            return Err(BrightDisputesError::InvalidDisputeState);
        }
        Ok(())
    }

    fn assert_juror(&self, juror: AccountId) -> Result<()> {
        for j in &self.juries {
            if *j == juror {
                return Ok(());
            }
        }
        return Err(BrightDisputesError::NotAuthorized);
    }

    fn assert_not_juror(&self, juror: AccountId) -> Result<()> {
        for j in &self.juries {
            if *j == juror {
                return Err(BrightDisputesError::JurorAlreadyAdded);
            }
        }
        return Ok(());
    }

    fn assert_not_voted(&self, juror: AccountId) -> Result<()> {
        for v in &self.votes {
            if v.juror() == juror {
                return Err(BrightDisputesError::JurorAlreadyVoted);
            }
        }
        return Ok(());
    }

    fn assert_can_vote(&self) -> Result<()> {
        if let Some(round) = &self.dispute_round {
            round.assert_if_not_voting_time()?;
            return Ok(());
        }
        Err(BrightDisputesError::WrongDisputeRoundState)
    }
}

#[cfg(test)]
mod tests {
    use ink::env::{test::set_caller, DefaultEnvironment};

    use super::*;
    use crate::{dispute_round::mock::DisputeRoundFake, juror::mock::JuriesMapMock};

    fn default_test_running_dispute() -> Dispute {
        let escrow_amount: Balance = 15;
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        set_caller::<DefaultEnvironment>(accounts.alice);

        let mut dispute = Dispute::create(
            1,
            "https://brightinventions.pl/owner".into(),
            accounts.bob,
            escrow_amount,
        );
        dispute.state = DisputeState::Running;
        dispute
    }

    #[ink::test]
    fn create_dispute() {
        let escrow_amount: Balance = 15;
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        set_caller::<DefaultEnvironment>(accounts.alice);

        let dispute = Dispute::create(
            1,
            "https://brightinventions.pl/owner".into(),
            accounts.bob,
            escrow_amount,
        );
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();

        assert_eq!(dispute.id, 1);
        assert_eq!(dispute.state, DisputeState::Created);
        assert_eq!(dispute.owner, accounts.alice);
        assert_eq!(dispute.owner_link, "https://brightinventions.pl/owner");
        assert_eq!(dispute.escrow, 15);
        assert_eq!(dispute.deposit, 15);
        assert_eq!(dispute.defendant, accounts.bob);
        assert_eq!(dispute.defendant_link, None);
        assert_eq!(dispute.dispute_result, None);
        assert_eq!(dispute.dispute_round_counter, 0u8);
        assert_eq!(dispute.judge, None);
        assert_eq!(dispute.juries.len(), 0);
        assert_eq!(dispute.banned.len(), 0);
        assert_eq!(dispute.votes.len(), 0);
    }

    #[ink::test]
    fn vote() {
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        let mut dispute = default_test_running_dispute();

        let mut juror = Juror::create(accounts.charlie);
        dispute.assign_juror(&mut juror).expect("Unable to add juror!");

        // Force "Voting" state
        dispute.dispute_round = Some(DisputeRoundFake::voting(0u64));

        // Only juror can vote
        let result = dispute.vote(Vote::create(accounts.bob, 1));
        assert_eq!(result, Err(BrightDisputesError::NotAuthorized));

        // Success
        let result = dispute.vote(Vote::create(accounts.charlie, 1));
        assert_eq!(result, Ok(()));

        // Juror can vote only once
        let result = dispute.vote(Vote::create(accounts.charlie, 0));
        assert_eq!(result, Err(BrightDisputesError::JurorAlreadyVoted));
    }

    #[ink::test]
    fn count_votes() {
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        let mut dispute = default_test_running_dispute();

        // Force "Voting" state
        dispute.dispute_round = Some(DisputeRoundFake::voting(0u64));

        let mut charlie = Juror::create(accounts.charlie);
        dispute
            .assign_juror(&mut charlie)
            .expect("Unable to add juror!");

        let mut eve = Juror::create(accounts.eve);
        dispute.assign_juror(&mut eve).expect("Unable to add juror!");

        let mut frank = Juror::create(accounts.frank);
        dispute
            .assign_juror(&mut frank)
            .expect("Unable to add juror!");

        let mut django = Juror::create(accounts.django);
        dispute
            .assign_judge(&mut django)
            .expect("Unable to add juror!");

        // Test, only judge can count the votes.
        set_caller::<DefaultEnvironment>(accounts.alice);
        let result = dispute.count_votes();
        assert_eq!(result, Err(BrightDisputesError::NotAuthorized));

        set_caller::<DefaultEnvironment>(accounts.bob);
        let result = dispute.count_votes();
        assert_eq!(result, Err(BrightDisputesError::NotAuthorized));

        set_caller::<DefaultEnvironment>(accounts.charlie);
        let result = dispute.count_votes();
        assert_eq!(result, Err(BrightDisputesError::NotAuthorized));

        set_caller::<DefaultEnvironment>(accounts.django);
        let result = dispute.count_votes();
        assert_eq!(result, Err(BrightDisputesError::MajorityOfVotesNotReached));

        // Voting
        dispute
            .vote(Vote::create(accounts.charlie, 1))
            .expect("Failed to vote!");

        dispute
            .vote(Vote::create(accounts.eve, 1))
            .expect("Failed to vote!");

        set_caller::<DefaultEnvironment>(accounts.django);
        let result = dispute.count_votes();
        assert_eq!(result, Err(BrightDisputesError::MajorityOfVotesNotReached));

        dispute
            .vote(Vote::create(accounts.frank, 1))
            .expect("Failed to vote!");

        set_caller::<DefaultEnvironment>(accounts.django);
        let result = dispute.count_votes();
        assert_eq!(result, Ok(DisputeResult::Owner));
    }

    #[ink::test]
    fn assign_juror() {
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        let mut dispute = default_test_running_dispute();

        // Failed, owner can't assign as a juror to dispute
        let mut alice = Juror::create(accounts.alice);
        let result = dispute.assign_juror(&mut alice);
        assert_eq!(result, Err(BrightDisputesError::NotAuthorized));

        // Failed, defendant can't assign as a juror to dispute
        let mut bob = Juror::create(accounts.bob);
        let result = dispute.assign_juror(&mut bob);
        assert_eq!(result, Err(BrightDisputesError::NotAuthorized));

        // Success
        let mut juror = Juror::create(accounts.charlie);
        let result = dispute.assign_juror(&mut juror);
        assert_eq!(result, Ok(()));
        assert_eq!(juror.assigned_dispute(), Some(1));

        // Juror already added
        let result = dispute.assign_juror(&mut juror);
        assert_eq!(result, Err(BrightDisputesError::JurorAlreadyAdded));
        assert_eq!(juror.assigned_dispute(), Some(1));
    }

    #[ink::test]
    fn assign_judge() {
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        let mut dispute = default_test_running_dispute();

        let mut charlie = Juror::create(accounts.charlie);
        dispute
            .assign_juror(&mut charlie)
            .expect("Unable to assign \"charlie\" as a juror!");

        // Unable to assign juror as a judge
        let result = dispute.assign_judge(&mut charlie);
        assert_eq!(
            result,
            Err(BrightDisputesError::JurorAlreadyAssignedToDispute)
        );

        // Failed, owner can't assign as a judge to dispute
        let mut alice = Juror::create(accounts.alice);
        let result = dispute.assign_judge(&mut alice);
        assert_eq!(result, Err(BrightDisputesError::NotAuthorized));

        // Failed, defendant can't assign as a judge to dispute
        let mut bob = Juror::create(accounts.bob);
        let result = dispute.assign_judge(&mut bob);
        assert_eq!(result, Err(BrightDisputesError::NotAuthorized));

        let mut eve = Juror::create(accounts.eve);

        // Success
        let result = dispute.assign_judge(&mut eve);
        assert_eq!(result, Ok(()));
        assert_eq!(eve.assigned_dispute(), Some(1));

        // Juror already added
        let result = dispute.assign_judge(&mut eve);
        assert_eq!(
            result,
            Err(BrightDisputesError::JudgeAlreadyAssignedToDispute)
        );
        assert_eq!(eve.assigned_dispute(), Some(1));
    }

    #[ink::test]
    fn move_to_banned() {
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        let mut dispute = default_test_running_dispute();

        let mut juror = Juror::create(accounts.charlie);
        dispute.assign_juror(&mut juror).expect("Unable to add juror!");

        let mut judge = Juror::create(accounts.django);
        dispute
            .assign_judge(&mut judge)
            .expect("Unable to add judge!");

        // Failed to moved unassigned juror or judge.
        assert_eq!(
            dispute.move_to_banned(accounts.frank),
            Err(BrightDisputesError::InvalidAction)
        );
        assert_eq!(dispute.banned.len(), 0);

        // Failed to moved owner.
        assert_eq!(
            dispute.move_to_banned(accounts.alice),
            Err(BrightDisputesError::InvalidAction)
        );
        assert_eq!(dispute.banned.len(), 0);

        // Failed to moved defendant.
        assert_eq!(
            dispute.move_to_banned(accounts.bob),
            Err(BrightDisputesError::InvalidAction)
        );
        assert_eq!(dispute.banned.len(), 0);

        // Success, moved juror.
        let result = dispute.move_to_banned(accounts.charlie);
        assert_eq!(result, Ok(()));
        assert_eq!(dispute.juries.len(), 0);
        assert_eq!(dispute.banned.len(), 1);
        assert_eq!(dispute.banned[0], accounts.charlie);

        // Success, moved judge.
        let result = dispute.move_to_banned(accounts.django);
        assert_eq!(result, Ok(()));
        assert_eq!(dispute.judge, None);
        assert_eq!(dispute.banned.len(), 2);
        assert_eq!(dispute.banned[1], accounts.django);
    }

    #[ink::test]
    fn set_dispute_round() {
        let mut dispute = default_test_running_dispute();

        // Check initial state
        assert!(dispute.dispute_round.is_none());

        // Success
        dispute.set_dispute_round(DisputeRound::create(0u64, None));
        assert!(dispute.dispute_round.is_some());
    }

    #[ink::test]
    fn process_dispute_round() {
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();

        set_caller::<DefaultEnvironment>(accounts.alice);

        let mut dispute = Dispute::create(
            1,
            "https://brightinventions.pl/owner".into(),
            accounts.bob,
            20,
        );

        let mut juries = JuriesMapMock::create(Juror::create(accounts.charlie));

        // Failed, dispute round not started
        let result = dispute.process_dispute_round(&mut juries, 0u64);
        assert_eq!(result, Err(BrightDisputesError::InvalidDisputeState));

        set_caller::<DefaultEnvironment>(accounts.bob);
        dispute
            .confirm_defendant("".into())
            .expect("Failed to confirm defendant.");

        set_caller::<DefaultEnvironment>(accounts.alice);

        // Failed, dispute round not started
        let result = dispute.process_dispute_round(&mut juries, 0u64);
        assert_eq!(result, Err(BrightDisputesError::DisputeRoundNotStarted));

        dispute.set_dispute_round(DisputeRound::create(0u64, None));

        // Failed, condition not meet.
        let result = dispute.process_dispute_round(&mut juries, 0u64);
        assert_eq!(result, Err(BrightDisputesError::JuriesPoolIsToSmall));
    }

    #[ink::test]
    fn confirm_defendant() {
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        let defendant_link: String = "https://brightinventions.pl/defendant".into();
        set_caller::<DefaultEnvironment>(accounts.alice);

        let mut dispute = Dispute::create(
            1,
            "https://brightinventions.pl/owner".into(),
            accounts.bob,
            15,
        );

        // Only defendant can confirm
        set_caller::<DefaultEnvironment>(accounts.alice);
        let result = dispute.confirm_defendant(defendant_link.clone());
        assert_eq!(result, Err(BrightDisputesError::NotAuthorized));

        // Success
        set_caller::<DefaultEnvironment>(accounts.bob);
        assert_eq!(dispute.has_defendant_confirmed_dispute(), false);
        let result = dispute.confirm_defendant(defendant_link.clone());
        assert_eq!(result, Ok(()));
        assert_eq!(dispute.has_defendant_confirmed_dispute(), true);
        assert_eq!(dispute.defendant_link, Some(defendant_link.clone()));

        // Invalid state
        let result = dispute.confirm_defendant(defendant_link);
        assert_eq!(result, Err(BrightDisputesError::InvalidDisputeState));
    }

    #[ink::test]
    fn set_owner_link() {
        let mut dispute = default_test_running_dispute();
        let owner_link = dispute.owner_link.clone();

        // Only owner can change link
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        set_caller::<DefaultEnvironment>(accounts.bob);
        let link1: String = "https://brightinventions.pl/owner1".into();
        let result = dispute.set_owner_link(link1.clone());
        assert_eq!(result, Err(BrightDisputesError::NotAuthorized));
        assert_eq!(dispute.owner_link, owner_link);

        set_caller::<DefaultEnvironment>(accounts.alice);

        // Success
        let link1: String = "https://brightinventions.pl/owner1".into();
        let result = dispute.set_owner_link(link1.clone());
        assert_eq!(result, Ok(()));
        assert_eq!(dispute.owner_link, link1);

        // Success
        let link2: String = "https://brightinventions.pl/owner2".into();
        let result = dispute.set_owner_link(link2.clone());
        assert_eq!(result, Ok(()));
        assert_eq!(dispute.owner_link, link2);
    }

    #[ink::test]
    fn set_defendant_link() {
        let mut dispute = default_test_running_dispute();
        let defendant_link = dispute.defendant_link.clone();

        // Only defendant can change link
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        set_caller::<DefaultEnvironment>(accounts.alice);
        let link1: String = "https://brightinventions.pl/defendant1".into();
        let result = dispute.set_defendant_link(link1.clone());
        assert_eq!(result, Err(BrightDisputesError::NotAuthorized));
        assert_eq!(dispute.defendant_link, defendant_link);

        set_caller::<DefaultEnvironment>(accounts.bob);

        // Success
        let link1: String = "https://brightinventions.pl/defendant1".into();
        let result = dispute.set_defendant_link(link1.clone());
        assert_eq!(result, Ok(()));
        assert_eq!(dispute.defendant_link, Some(link1));

        // Success
        let link2: String = "https://brightinventions.pl/defendant2".into();
        let result = dispute.set_defendant_link(link2.clone());
        assert_eq!(result, Ok(()));
        assert_eq!(dispute.defendant_link, Some(link2));
    }

    #[ink::test]
    fn get_not_voted_juries() {
        let mut dispute = default_test_running_dispute();

        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        let mut charlie = Juror::create(accounts.charlie);
        dispute
            .assign_juror(&mut charlie)
            .expect("Unable to add juror!");

        let mut eve = Juror::create(accounts.eve);
        dispute.assign_juror(&mut eve).expect("Unable to add juror!");

        // Force "Voting" state
        dispute.dispute_round = Some(DisputeRoundFake::voting(0u64));

        assert_eq!(
            dispute.get_not_voted_juries(),
            vec![accounts.charlie, accounts.eve]
        );

        dispute
            .vote(Vote::create(accounts.charlie, 1))
            .expect("Failed to vote!");

        assert_eq!(dispute.get_not_voted_juries(), vec![accounts.eve]);
    }

    #[ink::test]
    fn next_dispute_round_limit() {
        let mut dispute = default_test_running_dispute();

        for _ in 0..Dispute::MAX_DISPUTE_ROUNDS {
            let result = dispute.next_dispute_round(0u64);
            assert_eq!(result, Ok(()));
        }
        let result = dispute.next_dispute_round(0u64);
        assert_eq!(result, Err(BrightDisputesError::DisputeRoundLimitReached));
    }

    #[ink::test]
    fn next_dispute_round() {
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        let mut dispute = default_test_running_dispute();

        // Force "Voting" state
        dispute.dispute_round = Some(DisputeRoundFake::voting(0u64));

        let mut charlie = Juror::create(accounts.charlie);
        dispute
            .assign_juror(&mut charlie)
            .expect("Unable to add juror!");
        dispute
            .vote(Vote::create(accounts.charlie, 1))
            .expect("Failed to vote!");

        let mut eve = Juror::create(accounts.eve);
        dispute.assign_juror(&mut eve).expect("Unable to add juror!");

        assert_eq!(dispute.votes().len(), 1);
        assert_eq!(dispute.juries().len(), 2);
        assert_eq!(dispute.banned().len(), 0);

        let result = dispute.next_dispute_round(0u64);
        assert_eq!(result, Ok(()));

        assert_eq!(dispute.votes().len(), 0);
        assert_eq!(dispute.juries().len(), 2);
    }

    #[ink::test]
    fn increment_deposit() {
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        set_caller::<DefaultEnvironment>(accounts.alice);
        let mut dispute = Dispute::create(1, "".into(), accounts.bob, 15);

        assert_eq!(dispute.deposit, 15);
        dispute.increment_deposit();
        assert_eq!(dispute.deposit, 30);
    }

    #[ink::test]
    fn end_dispute() {
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        set_caller::<DefaultEnvironment>(accounts.alice);

        let mut dispute = Dispute::create(1, "".into(), accounts.bob, 15);
        let result = dispute.end_dispute(DisputeResult::Owner);
        assert_eq!(result, Err(BrightDisputesError::InvalidDisputeState));

        // Force "Voting" state
        dispute.dispute_round = Some(DisputeRoundFake::voting(0u64));
        dispute.state = DisputeState::Running;

        // Charlie vote against Owner
        let mut charlie = Juror::create(accounts.charlie);
        dispute
            .assign_juror(&mut charlie)
            .expect("Unable to add juror!");
        dispute
            .vote(Vote::create(accounts.charlie, 1))
            .expect("Failed to vote!");

        // Eve vote against Defendant
        let mut eve = Juror::create(accounts.eve);
        dispute.assign_juror(&mut eve).expect("Unable to add juror!");
        dispute
            .vote(Vote::create(accounts.eve, 0))
            .expect("Failed to vote!");

        let result = dispute.end_dispute(DisputeResult::Owner);
        assert_eq!(result, Ok(()));

        assert_eq!(dispute.banned().len(), 1);
        assert_eq!(dispute.banned()[0], accounts.eve);
    }

    #[ink::test]
    fn close_dispute() {
        let mut dispute = default_test_running_dispute();

        let result = dispute.close_dispute();
        assert_eq!(result, Err(BrightDisputesError::InvalidDisputeState));

        dispute.state = DisputeState::Ended;

        let result = dispute.close_dispute();
        assert_eq!(result, Ok(()));
    }

    #[ink::test]
    fn assert_owner_call() {
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        set_caller::<DefaultEnvironment>(accounts.alice);
        let dispute = Dispute::create(1, "".into(), accounts.bob, 15);

        set_caller::<DefaultEnvironment>(accounts.bob);
        let result = dispute.assert_owner_call();
        assert_eq!(result, Err(BrightDisputesError::NotAuthorized));

        set_caller::<DefaultEnvironment>(accounts.alice);
        let result = dispute.assert_owner_call();
        assert_eq!(result, Ok(()));
    }

    #[ink::test]
    fn assert_dispute_ended() {
        let mut dispute = default_test_running_dispute();

        let result = dispute.assert_dispute_ended();
        assert_eq!(result, Err(BrightDisputesError::InvalidDisputeState));

        dispute.dispute_round_counter = Dispute::MAX_DISPUTE_ROUNDS;
        let result = dispute.assert_dispute_ended();
        assert_eq!(result, Ok(()));

        dispute.dispute_round_counter = 0u8;
        dispute.state = DisputeState::Ended;
        let result = dispute.assert_dispute_ended();
        assert_eq!(result, Ok(()));
    }

    #[ink::test]
    fn assert_dispute_remove() {
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        set_caller::<DefaultEnvironment>(accounts.alice);
        let mut dispute = Dispute::create(1, "".into(), accounts.bob, 15);

        let result = dispute.assert_dispute_remove();
        assert_eq!(result, Ok(()));

        dispute.state = DisputeState::Running;
        let result = dispute.assert_dispute_remove();
        assert_eq!(result, Err(BrightDisputesError::InvalidDisputeState));

        dispute.state = DisputeState::Ended;
        let result = dispute.assert_dispute_remove();
        assert_eq!(result, Err(BrightDisputesError::InvalidDisputeState));

        dispute.state = DisputeState::Closed;
        let result = dispute.assert_dispute_remove();
        assert_eq!(result, Ok(()));
    }
}
