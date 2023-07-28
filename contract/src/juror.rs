use ink::{prelude::vec::Vec, primitives::AccountId};

use crate::{
    error::BrightDisputesError,
    types::{DisputeId, Result},
};

pub trait JuriesMap {
    fn get_juror_or_assert(&self, juror_id: AccountId) -> Result<Juror>;
    fn remove_random_juries_from_pool_or_assert(
        &mut self,
        except: &Vec<AccountId>,
        number: u8,
    ) -> Result<Vec<AccountId>>;
    fn update_juror(&mut self, juror: Juror);
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
pub struct Juror {
    id: AccountId,
    state: State,
    dispute_id: Option<DisputeId>,
}

impl Juror {
    #[allow(dead_code)]
    pub fn create(id: AccountId) -> Self {
        Juror {
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
            && self.state != State::Pending
            && self.state != State::Assigned
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
        Err(BrightDisputesError::JurorInvalidState)
    }

    pub fn action_done(&mut self, dispute_id: DisputeId) -> Result<()> {
        if let Some(id) = self.dispute_id {
            if id == dispute_id && self.state == State::RequestAction {
                self.state = State::ActionDone;
                return Ok(());
            }
        }
        Err(BrightDisputesError::JurorInvalidState)
    }

    pub fn assigned_dispute(&self) -> Option<DisputeId> {
        self.dispute_id
    }

    pub fn assign_to_dispute(&mut self, dispute_id: DisputeId) -> Result<()> {
        if self.state != State::Pending {
            return Err(BrightDisputesError::JurorAlreadyAssignedToDispute);
        }
        self.state = State::Assigned;
        self.dispute_id = Some(dispute_id);
        Ok(())
    }

    pub fn confirm_participation_in_dispute(&mut self, dispute_id: DisputeId) -> Result<()> {
        if self.dispute_id.is_none() || self.dispute_id.unwrap() != dispute_id {
            return Err(BrightDisputesError::JurorIsNotAssignedToDispute);
        } else if self.state != State::Assigned {
            return Err(BrightDisputesError::JurorAlreadyConfirmedDispute);
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
        juries: Vec<Juror>,
    }

    impl JuriesMapMock {
        pub fn create(juror: Juror) -> Self {
            let juries_pool: Vec<AccountId> = vec![juror.id()];
            let juries: Vec<Juror> = vec![juror];
            JuriesMapMock {
                juries_pool,
                juries,
            }
        }

        pub fn create_vec(juries: Vec<Juror>) -> Self {
            let juries_pool: Vec<AccountId> = juries.iter().map(|juror| juror.id()).collect();
            JuriesMapMock {
                juries_pool,
                juries,
            }
        }
    }

    impl Deref for JuriesMapMock {
        type Target = Vec<Juror>;

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
        fn get_juror_or_assert(&self, juror_id: AccountId) -> Result<Juror> {
            let juror = self.juries.iter().find(|j| j.id() == juror_id);
            if juror.is_none() {
                return Err(BrightDisputesError::JurorNotExist);
            }
            Ok(juror.unwrap().clone())
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

        fn update_juror(&mut self, juror: Juror) {
            let index = self.juries.iter().position(|j| j.id() == juror.id());
            if index.is_some() {
                self.juries[index.unwrap()] = juror;
            } else {
                self.juries.push(juror);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use ink::env::DefaultEnvironment;

    use super::*;

    #[ink::test]
    fn juror_created() {
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        let juror = Juror::create(accounts.bob);
        assert_eq!(juror.id, accounts.bob);
        assert_eq!(juror.dispute_id, None);
    }

    #[ink::test]
    fn action_done() {
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        let mut juror = Juror::create(accounts.bob);

        let dispute_id: DisputeId = 1;

        // Failed, no dispute assigned
        let result = juror.request_for_action(dispute_id);
        assert_eq!(result, Err(BrightDisputesError::JurorInvalidState));

        // Failed, no dispute assigned
        let result = juror.action_done(dispute_id);
        assert_eq!(result, Err(BrightDisputesError::JurorInvalidState));

        juror.assign_to_dispute(dispute_id)
            .expect("Failed to assign juror to dispute!");

        // Failed, no action assigned
        let result = juror.action_done(dispute_id);
        assert_eq!(result, Err(BrightDisputesError::JurorInvalidState));

        // Failed, juror not Confirmed
        let result = juror.request_for_action(dispute_id);
        assert_eq!(result, Err(BrightDisputesError::JurorInvalidState));

        juror.confirm_participation_in_dispute(dispute_id)
            .expect("Failed to confirm juror participation in dispute!");

        // Success
        let result = juror.request_for_action(dispute_id);
        assert_eq!(result, Ok(()));

        // Success
        let result = juror.action_done(dispute_id);
        assert_eq!(result, Ok(()));

        // Failed, already done
        let result = juror.action_done(dispute_id);
        assert_eq!(result, Err(BrightDisputesError::JurorInvalidState));

        // Success
        let result = juror.request_for_action(dispute_id);
        assert_eq!(result, Ok(()));
    }

    #[ink::test]
    fn assign_to_dispute() {
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        let mut juror = Juror::create(accounts.alice);

        // Success
        let result = juror.assign_to_dispute(2);
        assert_eq!(result, Ok(()));
        assert_eq!(juror.dispute_id, Some(2));

        // Failed to assign, already assigned
        let result = juror.assign_to_dispute(2);
        assert_eq!(
            result,
            Err(BrightDisputesError::JurorAlreadyAssignedToDispute)
        );
    }

    #[ink::test]
    fn confirm_participation_in_dispute() {
        let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
        let mut juror = Juror::create(accounts.alice);

        // Failed to confirm participation in dispute, when juror is not assigned.
        let result = juror.confirm_participation_in_dispute(1);
        assert_eq!(result, Err(BrightDisputesError::JurorIsNotAssignedToDispute));

        // Assign juror to dispute.
        juror.assign_to_dispute(1).expect("Unable to add juror!");

        // Success
        let result = juror.confirm_participation_in_dispute(1);
        assert_eq!(result, Ok(()));

        // Failed to confirm participation in dispute, juror already confirmed.
        let result = juror.confirm_participation_in_dispute(1);
        assert_eq!(
            result,
            Err(BrightDisputesError::JurorAlreadyConfirmedDispute)
        );
    }
}
