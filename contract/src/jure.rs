use ink::{prelude::vec::Vec, primitives::AccountId};

use crate::{
    error::BrightDisputesError,
    types::{DisputeId, Result},
};

pub trait JuriesMap {
    fn get_jure_or_assert(&self, jure_id: AccountId) -> Result<Jure>;
    fn remove_random_juries_from_pool_or_assert(
        &mut self,
        except: &Vec<AccountId>,
        number: u8,
    ) -> Result<Vec<AccountId>>;
    fn update_jure(&mut self, jure: Jure);
}

#[derive(Clone, Debug, PartialEq, scale::Decode, scale::Encode)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout, scale_info::TypeInfo)
)]
pub enum State {
    Pending,
    Assigned,
    Confirmed,
    RequestAction,
    ActionDone,
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

    pub fn is_requested_for_action(&self, dispute_id: DisputeId) -> bool {
        self.dispute_id.is_some()
            && self.dispute_id.unwrap() == dispute_id
            && self.state == State::RequestAction
    }

    pub fn request_for_action(&mut self, dispute_id: DisputeId) -> Result<()> {
        if let Some(id) = self.dispute_id {
            if id == dispute_id
                && (self.state == State::ActionDone || self.state == State::Confirmed)
            {
                self.state = State::RequestAction;
                return Ok(());
            }
        }
        Err(BrightDisputesError::JureInvalidState)
    }

    pub fn action_done(&mut self, dispute_id: DisputeId) -> Result<()> {
        if let Some(id) = self.dispute_id {
            if id == dispute_id && self.state == State::RequestAction {
                self.state = State::ActionDone;
                return Ok(());
            }
        }
        Err(BrightDisputesError::JureInvalidState)
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
        if self.dispute_id.is_none() || self.dispute_id.unwrap() != dispute_id {
            return Err(BrightDisputesError::JureIsNotAssignedToDispute);
        } else if self.state != State::Assigned {
            return Err(BrightDisputesError::JureAlreadyConfirmedDispute);
        }
        self.state = State::Confirmed;
        Ok(())
    }
}

#[cfg(test)]
pub mod mock {
    use core::ops::{Deref, DerefMut};

    use super::*;
    pub struct JuriesMapMock {
        juries_pool: Vec<AccountId>,
        juries: Vec<Jure>,
    }

    impl JuriesMapMock {
        pub fn create(jure: Jure) -> Self {
            let juries_pool: Vec<AccountId> = vec![jure.id()];
            let juries: Vec<Jure> = vec![jure];
            JuriesMapMock {
                juries_pool,
                juries,
            }
        }

        pub fn create_vec(juries: Vec<Jure>) -> Self {
            let juries_pool: Vec<AccountId> = juries.iter().map(|jure| jure.id()).collect();
            JuriesMapMock {
                juries_pool,
                juries,
            }
        }
    }

    impl Deref for JuriesMapMock {
        type Target = Vec<Jure>;

        fn deref(&self) -> &Self::Target {
            &self.juries
        }
    }

    impl DerefMut for JuriesMapMock {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.juries
        }
    }

    impl JuriesMap for JuriesMapMock {
        fn get_jure_or_assert(&self, jure_id: AccountId) -> Result<Jure> {
            let jure = self.juries.iter().find(|j| j.id() == jure_id);
            if jure.is_none() {
                return Err(BrightDisputesError::JureNotExist);
            }
            Ok(jure.unwrap().clone())
        }

        fn remove_random_juries_from_pool_or_assert(
            &mut self,
            _except: &Vec<AccountId>,
            number: u8,
        ) -> Result<Vec<AccountId>> {
            let number_of_juries: usize = number as usize;
            if self.juries_pool.len() < number_of_juries {
                return Err(BrightDisputesError::JuriesPoolIsToSmall);
            }

            let juries = self.juries_pool[0..number_of_juries].to_vec();
            self.juries_pool.drain(0..number_of_juries);
            Ok(juries)
        }

        fn update_jure(&mut self, jure: Jure) {
            let index = self.juries.iter().position(|j| j.id() == jure.id());
            if index.is_some() {
                self.juries[index.unwrap()] = jure;
            } else {
                self.juries.push(jure);
            }
        }
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
    fn action_done() {
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        let mut jure = Jure::create(accounts.bob);

        let dispute_id: DisputeId = 1;

        // Failed, no dispute assigned
        let result = jure.request_for_action(dispute_id);
        assert_eq!(result, Err(BrightDisputesError::JureInvalidState));

        // Failed, no dispute assigned
        let result = jure.action_done(dispute_id);
        assert_eq!(result, Err(BrightDisputesError::JureInvalidState));

        jure.assign_to_dispute(dispute_id)
            .expect("Failed to assign jure to dispute!");

        // Failed, no action assigned
        let result = jure.action_done(dispute_id);
        assert_eq!(result, Err(BrightDisputesError::JureInvalidState));

        // Failed, jure not Confirmed
        let result = jure.request_for_action(dispute_id);
        assert_eq!(result, Err(BrightDisputesError::JureInvalidState));

        jure.confirm_participation_in_dispute(dispute_id)
            .expect("Failed to confirm jure participation in dispute!");

        // Success
        let result = jure.request_for_action(dispute_id);
        assert_eq!(result, Ok(()));

        // Success
        let result = jure.action_done(dispute_id);
        assert_eq!(result, Ok(()));

        // Failed, already done
        let result = jure.action_done(dispute_id);
        assert_eq!(result, Err(BrightDisputesError::JureInvalidState));

        // Success
        let result = jure.request_for_action(dispute_id);
        assert_eq!(result, Ok(()));
    }

    #[ink::test]
    fn assign_to_dispute() {
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        let mut jure = Jure::create(accounts.alice);

        // Success
        let result = jure.assign_to_dispute(2);
        assert_eq!(result, Ok(()));
        assert_eq!(jure.dispute_id, Some(2));

        // Failed to assign, already assigned
        let result = jure.assign_to_dispute(2);
        assert_eq!(
            result,
            Err(BrightDisputesError::JureAlreadyAssignedToDispute)
        );
    }

    #[ink::test]
    fn confirm_participation_in_dispute() {
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        let mut jure = Jure::create(accounts.alice);

        // Failed to confirm participation in dispute, when jure is not assigned.
        let result = jure.confirm_participation_in_dispute(1);
        assert_eq!(result, Err(BrightDisputesError::JureIsNotAssignedToDispute));

        // Assign jure to dispute.
        jure.assign_to_dispute(1).expect("Unable to add jure!");

        // Success
        let result = jure.confirm_participation_in_dispute(1);
        assert_eq!(result, Ok(()));

        // Failed to confirm participation in dispute, jure already confirmed.
        let result = jure.confirm_participation_in_dispute(1);
        assert_eq!(
            result,
            Err(BrightDisputesError::JureAlreadyConfirmedDispute)
        );
    }
}
