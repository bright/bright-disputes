use ink::{
    prelude::{string::String, vec::Vec},
    primitives::AccountId,
};

use crate::{
    error::BrightDisputesError,
    jure::Jure,
    types::{Balance, DisputeId, Result},
    vote::Vote,
};

#[derive(Clone, Debug, PartialEq, scale::Decode, scale::Encode)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout, scale_info::TypeInfo)
)]
pub enum State {
    Created,
    Confirmed,
    CollectingJuries,
    PickingJudge,
    Voting,
}

#[derive(Clone, Debug, PartialEq, scale::Decode, scale::Encode)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout, scale_info::TypeInfo)
)]
pub struct Dispute {
    id: DisputeId,
    state: State,
    owner: AccountId,
    owner_link: String,
    escrow: Balance,

    defendant: AccountId,
    defendant_link: Option<String>,

    judge: Option<AccountId>,
    juries: Vec<AccountId>,
    votes: Vec<Vote>,
}

impl Dispute {
    /// Creates a new dispute
    pub fn create(
        id: DisputeId,
        owner_link: String,
        defendant: AccountId,
        escrow: Balance,
    ) -> Self {
        Dispute {
            id,
            state: State::Created,
            owner: ink::env::caller::<ink::env::DefaultEnvironment>(),
            owner_link,
            escrow,
            defendant,
            defendant_link: None,
            judge: None,
            juries: Vec::new(),
            votes: Vec::new(),
        }
    }

    /// Confirm defendant participation in dispute
    pub fn confirm_defendant(&mut self, defendant_link: String) -> Result<()> {
        self.assert_defendant_call()?;
        self.assert_state(State::Created)?;
        self.defendant_link = Some(defendant_link);
        self.state = State::Confirmed;
        Ok(())
    }

    /// Make a vote
    #[allow(dead_code)]
    pub fn vote(&mut self, vote: Vote) -> Result<()> {
        self.assert_jure(vote.jure())?;
        self.assert_not_voted(vote.jure())?;
        self.votes.push(vote);
        Ok(())
    }

    /// Add jure to the dispute
    pub fn assign_jure(&mut self, jure: &mut Jure) -> Result<()> {
        self.assert_not_jure(jure.id())?;
        jure.assign_to_dispute(self.id)?;
        self.juries.push(jure.id());
        Ok(())
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

    /// Get juries
    pub fn juries(&self) -> Vec<AccountId> {
        return self.juries.clone();
    }

    /// Set owner decription link
    pub fn set_owner_link(&mut self, owner_link: String) -> Result<()> {
        self.assert_owner_call()?;
        self.owner_link = owner_link;
        Ok(())
    }

    /// Set defendant decription link
    pub fn set_defendant_link(&mut self, defendant_link: String) -> Result<()> {
        self.assert_defendant_call()?;
        self.defendant_link = Some(defendant_link);
        Ok(())
    }

    fn assert_owner_call(&self) -> Result<()> {
        if self.owner != ink::env::caller::<ink::env::DefaultEnvironment>() {
            return Err(BrightDisputesError::NotAuthorized);
        }
        Ok(())
    }

    fn assert_defendant_call(&self) -> Result<()> {
        if self.defendant != ink::env::caller::<ink::env::DefaultEnvironment>() {
            return Err(BrightDisputesError::NotAuthorized);
        }
        Ok(())
    }

    fn assert_state(&self, state: State) -> Result<()> {
        if self.state != state {
            return Err(BrightDisputesError::InvalidState);
        }
        Ok(())
    }

    fn assert_jure(&self, jure: AccountId) -> Result<()> {
        for j in &self.juries {
            if *j == jure {
                return Ok(());
            }
        }
        return Err(BrightDisputesError::NotAuthorized);
    }

    fn assert_not_jure(&self, jure: AccountId) -> Result<()> {
        for j in &self.juries {
            if *j == jure {
                return Err(BrightDisputesError::JureAlreadyAdded);
            }
        }
        return Ok(());
    }

    fn assert_not_voted(&self, jure: AccountId) -> Result<()> {
        for v in &self.votes {
            if v.jure() == jure {
                return Err(BrightDisputesError::JureAlreadyVoted);
            }
        }
        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use ink::env::{test::set_caller, DefaultEnvironment};

    use super::*;

    fn default_test_dispute() -> Dispute {
        let escrow_amount: Balance = 15;
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        set_caller::<DefaultEnvironment>(accounts.alice);

        Dispute::create(
            1,
            "https://brightinventions.pl/owner".into(),
            accounts.bob,
            escrow_amount,
        )
    }

    #[ink::test]
    fn create_dispute() {
        let dispute = default_test_dispute();
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();

        assert_eq!(dispute.id, 1);
        assert_eq!(dispute.state, State::Created);
        assert_eq!(dispute.owner, accounts.alice);
        assert_eq!(dispute.owner_link, "https://brightinventions.pl/owner");
        assert_eq!(dispute.escrow, 15);
        assert_eq!(dispute.defendant, accounts.bob);
        assert_eq!(dispute.defendant_link, None);
        assert_eq!(dispute.judge, None);
        assert_eq!(dispute.juries.len(), 0);
        assert_eq!(dispute.votes.len(), 0);
    }

    #[ink::test]
    fn vote() {
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        let mut dispute = default_test_dispute();

        let mut jure = Jure::create(accounts.charlie);
        dispute.assign_jure(&mut jure).expect("Unable to add jure!");

        // Only jure can vote
        let result = dispute.vote(Vote::create(accounts.bob, 1));
        assert_eq!(result, Err(BrightDisputesError::NotAuthorized));

        // Success
        let result = dispute.vote(Vote::create(accounts.charlie, 1));
        assert_eq!(result, Ok(()));

        // Jure can vote only once
        let result = dispute.vote(Vote::create(accounts.charlie, 0));
        assert_eq!(result, Err(BrightDisputesError::JureAlreadyVoted));
    }

    #[ink::test]
    fn assign_jure() {
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        let mut dispute = default_test_dispute();

        let mut jure = Jure::create(accounts.charlie);

        // Success
        let result = dispute.assign_jure(&mut jure);
        assert_eq!(result, Ok(()));
        assert_eq!(jure.assigned_dispute(), Some(1));

        // Jure already added
        let result = dispute.assign_jure(&mut jure);
        assert_eq!(result, Err(BrightDisputesError::JureAlreadyAdded));
        assert_eq!(jure.assigned_dispute(), Some(1));
    }

    #[ink::test]
    fn confirm_defendant() {
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        let defendant_link: String = "https://brightinventions.pl/defendant".into();
        let mut dispute = default_test_dispute();

        // Only defendant can confirm
        set_caller::<DefaultEnvironment>(accounts.alice);
        let result = dispute.confirm_defendant(defendant_link.clone());
        assert_eq!(result, Err(BrightDisputesError::NotAuthorized));

        // Success
        set_caller::<DefaultEnvironment>(accounts.bob);
        let result = dispute.confirm_defendant(defendant_link.clone());
        assert_eq!(result, Ok(()));
        assert_eq!(dispute.defendant_link, Some(defendant_link.clone()));

        // Invalid state
        let result = dispute.confirm_defendant(defendant_link);
        assert_eq!(result, Err(BrightDisputesError::InvalidState));
    }

    #[ink::test]
    fn set_owner_link() {
        let mut dispute = default_test_dispute();
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
        let mut dispute = default_test_dispute();
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
}
