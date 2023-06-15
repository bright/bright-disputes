use ink::primitives::AccountId;

use crate::{
    error::BrightDisputesError,
    types::{DisputeId, Result},
};

#[derive(Clone, Debug, PartialEq, scale::Decode, scale::Encode)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout, scale_info::TypeInfo)
)]
pub enum State {
    Pending,
    Assigned,
    Confirmed,
}

#[derive(Clone, Debug, PartialEq, scale::Decode, scale::Encode)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout, scale_info::TypeInfo)
)]
pub struct Jure {
    id: AccountId,
    state: State,
    dispute_id: Option<DisputeId>,
}

impl Jure {
    #[allow(dead_code)]
    pub fn create(id: AccountId) -> Self {
        Jure {
            id,
            state: State::Pending,
            dispute_id: None,
        }
    }

    pub fn id(&self) -> AccountId {
        self.id
    }

    pub fn is_confirmed(&self, dispute_id: DisputeId) -> bool {
        self.dispute_id.is_some()
            && self.dispute_id.unwrap() == dispute_id
            && self.state == State::Confirmed
    }

    pub fn assigned_dispute(&self) -> Option<DisputeId> {
        self.dispute_id
    }

    pub fn assign_to_dispute(&mut self, dispute_id: DisputeId) -> Result<()> {
        if self.state != State::Pending {
            return Err(BrightDisputesError::JureAlreadyAssignedToDispute);
        }
        self.state = State::Assigned;
        self.dispute_id = Some(dispute_id);
        Ok(())
    }

    pub fn confirm_participation_in_dispute(&mut self, dispute_id: DisputeId) -> Result<()> {
        if self.dispute_id.is_none()
            || self.dispute_id.unwrap() != dispute_id
            || self.state != State::Assigned
        {
            return Err(BrightDisputesError::JureIsNotAssignedToDispute);
        }
        self.state = State::Confirmed;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use ink::env::DefaultEnvironment;

    use super::*;

    #[ink::test]
    fn jure_created() {
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        let jure = Jure::create(accounts.bob);
        assert_eq!(jure.id, accounts.bob);
        assert_eq!(jure.dispute_id, None);
    }

    #[ink::test]
    fn assign_to_dispute() {
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        let mut jure = Jure::create(accounts.alice);

        let result = jure.assign_to_dispute(2);
        assert_eq!(result, Ok(()));
        assert_eq!(jure.dispute_id, Some(2));

        let result = jure.assign_to_dispute(2);
        assert_eq!(
            result,
            Err(BrightDisputesError::JureAlreadyAssignedToDispute)
        );
    }
}
