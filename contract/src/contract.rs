#[ink::contract(env = baby_liminal_extension::ink::BabyLiminalEnvironment)]
mod bright_disputes {

    use ink::{
        prelude::{string::String, vec::Vec},
        storage::Mapping,
    };

    use crate::{
        dispute::Dispute,
        error::BrightDisputesError,
        jure::Jure,
        types::{DisputeId, Result},
    };

    #[ink(event)]
    pub struct DisputeRaised {
        id: DisputeId,
        owner_id: AccountId,
        defendant_id: AccountId,
    }

    #[ink(event)]
    pub struct DisputeClosed {
        id: DisputeId,
    }

    #[ink(event)]
    pub struct DefendantConfirmDispute {
        id: DisputeId,
        defendant_id: AccountId,
    }

    /// Main contract storage
    #[ink(storage)]
    pub struct BrightDisputes {
        last_dispute_id: DisputeId,
        juries_pool: Vec<AccountId>,
        juries: Mapping<AccountId, Jure>,
        disputes: Mapping<DisputeId, Dispute>,
    }

    impl BrightDisputes {
        /// Constructor
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                last_dispute_id: 0,
                juries_pool: Vec::new(),
                juries: Mapping::default(),
                disputes: Mapping::default(),
            }
        }

        /// Get single dispute by id
        #[ink(message)]
        pub fn get_dispute(&self, dispute_id: DisputeId) -> Result<Dispute> {
            self.get_dispute_or_assert(dispute_id)
        }

        /// Get all disputes
        #[ink(message)]
        pub fn get_all_disputes(&self) -> Vec<Dispute> {
            (1..=self.last_dispute_id)
                .flat_map(|id| self.disputes.get(id))
                .collect()
        }

        /// Get single dispute by id
        #[ink(message)]
        pub fn remove_dispute(&mut self, dispute_id: DisputeId) -> Result<()> {
            self.get_dispute_or_assert(dispute_id)?;
            self.disputes.remove(dispute_id);

            self.env().emit_event(DisputeClosed { id: dispute_id });

            Ok(())
        }

        /// Create new dispute
        #[ink(message, payable)]
        pub fn create_dispute(
            &mut self,
            owner_link: String,
            defendant_id: AccountId,
            escrow: Balance,
        ) -> Result<DisputeId> {
            let owner_id = ink::env::caller::<ink::env::DefaultEnvironment>();
            self.last_dispute_id = self.generate_dispute_id()?;
            let dispute = Dispute::create(self.last_dispute_id, owner_link, defendant_id, escrow);
            self.update_dispute(dispute);

            self.env().transfer(self.env().account_id(), escrow)?;

            self.env().emit_event(DisputeRaised {
                id: self.last_dispute_id,
                owner_id,
                defendant_id,
            });

            Ok(self.last_dispute_id)
        }

        /// Defendant confirms his participation in dispute.
        #[ink(message)]
        pub fn confirm_defendant(
            &mut self,
            dispute_id: DisputeId,
            defendant_link: String,
        ) -> Result<()> {
            let mut dispute = self.get_dispute_or_assert(dispute_id)?;
            let id = dispute.id();
            dispute.confirm_defendant(defendant_link)?;
            self.env()
                .transfer(self.env().account_id(), dispute.escrow())?;
            self.update_dispute(dispute);

            self.env().emit_event(DefendantConfirmDispute {
                id,
                defendant_id: ink::env::caller::<ink::env::DefaultEnvironment>(),
            });

            Ok(())
        }

        /// Update owner link description
        #[ink(message)]
        pub fn update_owner_description(
            &mut self,
            dispute_id: DisputeId,
            owner_link: String,
        ) -> Result<()> {
            let mut dispute = self.get_dispute_or_assert(dispute_id)?;
            dispute.set_owner_link(owner_link)?;
            self.update_dispute(dispute);
            Ok(())
        }

        /// Update defendant link description
        #[ink(message)]
        pub fn update_defendant_description(
            &mut self,
            dispute_id: DisputeId,
            defendant_link: String,
        ) -> Result<()> {
            let mut dispute = self.get_dispute_or_assert(dispute_id)?;
            dispute.set_defendant_link(defendant_link)?;
            self.update_dispute(dispute);
            Ok(())
        }

        /// Register as an active jure. Juries are picked
        /// from this pool to participate in disputes.
        #[ink(message)]
        pub fn register_as_an_active_jure(&mut self) -> Result<()> {
            let caller = ink::env::caller::<ink::env::DefaultEnvironment>();
            self.assert_jure_not_in_pool(caller)?;

            let jure = self
                .juries
                .get(caller)
                .unwrap_or_else(|| Jure::create(caller));
            self.juries_pool.push(jure.id());
            self.update_jure(jure);
            Ok(())
        }

        /// Unregister jure from the active juries pool.
        #[ink(message)]
        pub fn unregister_as_an_active_jure(&mut self) -> Result<()> {
            let caller = ink::env::caller::<ink::env::DefaultEnvironment>();
            self.remove_jure_from_pool_or_assert(caller)?;
            Ok(())
        }

        /// Assign jure to dispute
        #[ink(message)]
        pub fn assign_juries_to_dispute(
            &mut self,
            dispute_id: DisputeId,
            number: u8,
        ) -> Result<()> {
            let mut dispute = self.get_dispute_or_assert(dispute_id)?;
            let juries_ids = self.remove_random_juries_from_pool_or_assert(
                &[dispute.owner(), dispute.defendant()].to_vec(),
                number,
            )?;

            for jure_id in juries_ids {
                let mut jure = self.get_jure_or_assert(jure_id)?;
                dispute.assign_jure(&mut jure)?;
                self.update_jure(jure);
            }
            self.update_dispute(dispute);

            Ok(())
        }

        /// Assigned jure can confirm his participation in dispute
        #[ink(message)]
        pub fn confirm_jure_participation_in_dispute(
            &mut self,
            dispute_id: DisputeId,
        ) -> Result<()> {
            let dispute = self.get_dispute_or_assert(dispute_id)?;

            let caller = ink::env::caller::<ink::env::DefaultEnvironment>();
            let mut jure = self.get_jure_or_assert(caller)?;
            jure.confirm_participation_in_dispute(dispute_id)?;
            self.env()
                .transfer(self.env().account_id(), dispute.escrow())?;
            self.update_jure(jure);
            Ok(())
        }

        fn update_dispute(&mut self, dispute: Dispute) {
            self.disputes.insert(dispute.id(), &dispute);
        }

        fn generate_dispute_id(&self) -> Result<DisputeId> {
            if let Some(id) = self.last_dispute_id.checked_add(1) {
                Ok(id)
            } else {
                Err(BrightDisputesError::InvalidAction)
            }
        }

        fn get_dispute_or_assert(&self, dispute_id: DisputeId) -> Result<Dispute> {
            self.disputes
                .get(dispute_id)
                .ok_or(BrightDisputesError::DisputeNotExist)
        }

        pub fn remove_random_juries_from_pool_or_assert(
            &mut self,
            except: &Vec<AccountId>,
            number: u8,
        ) -> Result<Vec<AccountId>> {
            let juries_ids = self.get_random_jures_from_pool(&except, number, 123 as u64)?;
            for id in &juries_ids {
                self.remove_jure_from_pool_or_assert(*id)?;
            }
            Ok(juries_ids)
        }

        fn get_jure_or_assert(&mut self, jure_id: AccountId) -> Result<Jure> {
            self.juries
                .get(jure_id)
                .ok_or(BrightDisputesError::JureNotExist)
        }

        fn update_jure(&mut self, jure: Jure) {
            self.juries.insert(jure.id(), &jure);
        }

        fn remove_jure_from_pool_or_assert(&mut self, jure_id: AccountId) -> Result<()> {
            if let Some(index) = self.juries_pool.iter().position(|&j| j == jure_id) {
                self.juries_pool.remove(index);
                return Ok(());
            }
            return Err(BrightDisputesError::NotRegisteredAsJure);
        }

        fn get_random_jures_from_pool(
            &self,
            except_juries: &Vec<AccountId>,
            number_of_juries: u8,
            seed: u64,
        ) -> Result<Vec<AccountId>> {
            let filtered_pool: Vec<AccountId> = self
                .juries_pool
                .iter()
                .filter(|jure_id| !except_juries.contains(&jure_id))
                .map(|&jure_id| jure_id)
                .collect();

            let pool_len: u64 = filtered_pool.len().try_into().unwrap();
            if filtered_pool.len() < number_of_juries.into() {
                return Err(BrightDisputesError::JuriesPoolIsToSmall);
            }

            let mut juries: Vec<AccountId> = Vec::new();
            let random_start = self.pseudo_random(seed);
            for i in 0..number_of_juries {
                let index: usize = ((i as u64 + random_start) % pool_len).try_into().unwrap();
                juries.push(filtered_pool[index]);
            }
            Ok(juries)
        }

        fn pseudo_random(&self, seed: u64) -> u64 {
            let random: u64 = self.env().block_timestamp();
            random + seed
        }

        fn assert_jure_not_in_pool(&self, jure_id: AccountId) -> Result<()> {
            for j in &self.juries_pool {
                if *j == jure_id {
                    return Err(BrightDisputesError::JureAlreadyRegistered);
                }
            }
            return Ok(());
        }
    }

    #[cfg(test)]
    mod tests {
        use ink::env::{test::set_caller, DefaultEnvironment};

        use super::*;

        fn register_valid_juries(bright_disputes: &mut BrightDisputes) {
            let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
            set_caller::<DefaultEnvironment>(accounts.charlie);
            bright_disputes
                .register_as_an_active_jure()
                .expect("Failed to register charlie as a jure!");
            set_caller::<DefaultEnvironment>(accounts.eve);
            bright_disputes
                .register_as_an_active_jure()
                .expect("Failed to register eve as a jure!");
            set_caller::<DefaultEnvironment>(accounts.frank);
            bright_disputes
                .register_as_an_active_jure()
                .expect("Failed to register frank as a jure!");
            set_caller::<DefaultEnvironment>(accounts.django);
            bright_disputes
                .register_as_an_active_jure()
                .expect("Failed to register django as a jure!");

            assert_eq!(bright_disputes.juries_pool.len(), 4);
            assert!(bright_disputes.juries.contains(accounts.charlie));
            assert!(bright_disputes.juries.contains(accounts.eve));
            assert!(bright_disputes.juries.contains(accounts.frank));
            assert!(bright_disputes.juries.contains(accounts.django));
        }

        /// Test if we can create only one single dispute.
        #[ink::test]
        fn create_single_dispute() {
            let mut bright_disputes = BrightDisputes::new();

            let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
            let owner_link = "https://brightinventions.pl/";
            let escrow_amount: Balance = 15;
            set_caller::<DefaultEnvironment>(accounts.alice);
            let result =
                bright_disputes.create_dispute(owner_link.into(), accounts.bob, escrow_amount);
            assert_eq!(result, Ok(1));
        }

        /// Test if we can create multiple disputes.
        #[ink::test]
        fn create_multiple_dispute() {
            let mut bright_disputes = BrightDisputes::new();

            let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
            set_caller::<DefaultEnvironment>(accounts.alice);

            // Alice creates first dispute
            let result = bright_disputes.create_dispute(
                "https://brightinventions.pl/".into(),
                accounts.bob,
                10,
            );
            assert_eq!(result, Ok(1));

            // Alice creates second dispute
            let result = bright_disputes.create_dispute(
                "https://brightinventions.pl/".into(),
                accounts.bob,
                10,
            );
            assert_eq!(result, Ok(2));
        }

        /// Test if we can get single disputes.
        #[ink::test]
        fn get_single_dispute() {
            let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
            let mut bright_disputes = BrightDisputes::new();

            let result = bright_disputes.get_dispute(1);
            assert_eq!(result, Err(BrightDisputesError::DisputeNotExist));

            bright_disputes
                .create_dispute("https://brightinventions.pl/1".into(), accounts.bob, 10)
                .expect("Failed to create a dispute!");

            let bob_dispute = bright_disputes.get_dispute(1);
            assert!(bob_dispute.is_ok());
            assert_eq!(bob_dispute.unwrap().id(), 1);
        }

        /// Test if we can get multiple disputes.
        #[ink::test]
        fn get_all_dispute() {
            let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
            let mut bright_disputes = BrightDisputes::new();

            bright_disputes
                .create_dispute("https://brightinventions.pl/1".into(), accounts.bob, 10)
                .expect("Failed to create a dispute!");

            bright_disputes
                .create_dispute("https://brightinventions.pl/2".into(), accounts.alice, 10)
                .expect("Failed to create a dispute!");

            let bob_dispute = bright_disputes.get_dispute(1);
            assert!(bob_dispute.is_ok());

            let alice_dispute = bright_disputes.get_dispute(2);
            assert!(alice_dispute.is_ok());

            let all_disputes = bright_disputes.get_all_disputes();
            assert_eq!(
                all_disputes,
                vec![bob_dispute.unwrap(), alice_dispute.unwrap()]
            );
        }

        /// Test if we can remove a single dispute.
        #[ink::test]
        fn remove_dispute() {
            let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
            let mut bright_disputes = BrightDisputes::new();

            let result = bright_disputes.remove_dispute(1);
            assert_eq!(result, Err(BrightDisputesError::DisputeNotExist));

            bright_disputes
                .create_dispute("https://brightinventions.pl".into(), accounts.bob, 10)
                .expect("Failed to create a dispute!");

            let result = bright_disputes.remove_dispute(1);
            assert_eq!(result, Ok(()));
        }

        /// Test confirmation of the defendant
        #[ink::test]
        fn confirm_defendant() {
            let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
            set_caller::<DefaultEnvironment>(accounts.alice);

            let mut bright_disputes = BrightDisputes::new();

            let defendant_link = "https://brightinventions.pl/";

            // Check when there is no dispute
            let result = bright_disputes.confirm_defendant(1, defendant_link.into());
            assert_eq!(result, Err(BrightDisputesError::DisputeNotExist));

            bright_disputes
                .create_dispute("".into(), accounts.bob, 10)
                .expect("Failed to create a dispute!");

            // Check when dispute exist, but there someone else try to assign
            let result = bright_disputes.confirm_defendant(1, defendant_link.into());
            assert_eq!(result, Err(BrightDisputesError::NotAuthorized));

            // Check when dispute exist, but call refers to wrong dispute.
            let result = bright_disputes.confirm_defendant(0, defendant_link.into());
            assert_eq!(result, Err(BrightDisputesError::DisputeNotExist));

            // Check when defendant assign.
            set_caller::<DefaultEnvironment>(accounts.bob);
            let result = bright_disputes.confirm_defendant(1, defendant_link.into());
            assert_eq!(result, Ok(()));
        }

        /// Test dispute owner description update
        #[ink::test]
        fn update_owner_description() {
            let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
            set_caller::<DefaultEnvironment>(accounts.alice);

            let mut bright_disputes = BrightDisputes::new();

            bright_disputes
                .create_dispute("https://brightinventions.pl/".into(), accounts.bob, 10)
                .expect("Failed to create a dispute!");

            // Failed to update, wrong dispute
            let result = bright_disputes
                .update_owner_description(0, "https://brightinventions.pl/owner".into());
            assert_eq!(result, Err(BrightDisputesError::DisputeNotExist));

            // Failed to update, only owner can make an update
            set_caller::<DefaultEnvironment>(accounts.bob);
            let result = bright_disputes
                .update_owner_description(1, "https://brightinventions.pl/owner".into());
            assert_eq!(result, Err(BrightDisputesError::NotAuthorized));

            // Success
            set_caller::<DefaultEnvironment>(accounts.alice);
            let result = bright_disputes
                .update_owner_description(1, "https://brightinventions.pl/owner".into());
            assert_eq!(result, Ok(()));
        }

        /// Test dispute defendant description update
        #[ink::test]
        fn update_defendant_description() {
            let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
            set_caller::<DefaultEnvironment>(accounts.alice);

            let mut bright_disputes = BrightDisputes::new();

            bright_disputes
                .create_dispute("https://brightinventions.pl/".into(), accounts.bob, 10)
                .expect("Failed to create a dispute!");

            // Failed to update, wrong dispute
            let result = bright_disputes
                .update_defendant_description(0, "https://brightinventions.pl/defendant".into());
            assert_eq!(result, Err(BrightDisputesError::DisputeNotExist));

            // Failed to update, only defendant can make an update
            let result = bright_disputes
                .update_defendant_description(1, "https://brightinventions.pl/defendant".into());
            assert_eq!(result, Err(BrightDisputesError::NotAuthorized));

            // Success
            set_caller::<DefaultEnvironment>(accounts.bob);
            let result = bright_disputes
                .update_defendant_description(1, "https://brightinventions.pl/defendant".into());
            assert_eq!(result, Ok(()));
        }

        /// Test jure registration
        #[ink::test]
        fn jure_registration() {
            let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
            set_caller::<DefaultEnvironment>(accounts.alice);

            let mut bright_disputes = BrightDisputes::new();
            assert_eq!(bright_disputes.juries_pool.len(), 0);
            assert!(!bright_disputes.juries.contains(accounts.alice));

            // Success
            let result = bright_disputes.register_as_an_active_jure();
            assert_eq!(result, Ok(()));
            assert_eq!(bright_disputes.juries_pool.len(), 1);
            assert!(bright_disputes.juries.contains(accounts.alice));

            // Failed to register already registered jure
            let result = bright_disputes.register_as_an_active_jure();
            assert_eq!(result, Err(BrightDisputesError::JureAlreadyRegistered));
            assert_eq!(bright_disputes.juries_pool.len(), 1);
            assert!(bright_disputes.juries.contains(accounts.alice));
        }

        /// Test jure unregistration
        #[ink::test]
        fn jure_unregistration() {
            let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
            set_caller::<DefaultEnvironment>(accounts.alice);

            let mut bright_disputes = BrightDisputes::new();

            // Failed to unregister jure, no jure in the pool
            let result = bright_disputes.unregister_as_an_active_jure();
            assert_eq!(result, Err(BrightDisputesError::NotRegisteredAsJure));
            assert_eq!(bright_disputes.juries_pool.len(), 0);
            assert!(!bright_disputes.juries.contains(accounts.alice));

            bright_disputes
                .register_as_an_active_jure()
                .expect("Failed to register a jure!");

            // Success
            let result = bright_disputes.unregister_as_an_active_jure();
            assert_eq!(result, Ok(()));
            assert_eq!(bright_disputes.juries_pool.len(), 0);
            assert!(bright_disputes.juries.contains(accounts.alice));

            // Failed to unregister jure, jure already unregistered
            let result = bright_disputes.unregister_as_an_active_jure();
            assert_eq!(result, Err(BrightDisputesError::NotRegisteredAsJure));
        }

        // Test juries assigment to the dispute case.
        #[ink::test]
        fn assign_juries_to_dispute() {
            let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
            set_caller::<DefaultEnvironment>(accounts.alice);

            let mut bright_disputes = BrightDisputes::new();

            // Create a dispute
            let dispute_id = bright_disputes
                .create_dispute("https://brightinventions.pl/".into(), accounts.bob, 10)
                .expect("Failed to create a dispute!");

            // Try to assign more juries when pool is empty
            let result = bright_disputes.assign_juries_to_dispute(dispute_id, 1);
            assert_eq!(result, Err(BrightDisputesError::JuriesPoolIsToSmall));

            // Register charlie, django, eve and frank as a juries.
            register_valid_juries(&mut bright_disputes);

            // Try to assign more juries than there is in the pool
            let result = bright_disputes.assign_juries_to_dispute(dispute_id, 5);
            assert_eq!(result, Err(BrightDisputesError::JuriesPoolIsToSmall));
            assert_eq!(bright_disputes.juries_pool.len(), 4);

            // Success
            let result = bright_disputes.assign_juries_to_dispute(dispute_id, 4);
            assert_eq!(result, Ok(()));
            assert_eq!(bright_disputes.juries_pool.len(), 0);

            // Check that there is no more juries left in the pool
            let result = bright_disputes.assign_juries_to_dispute(dispute_id, 1);
            assert_eq!(result, Err(BrightDisputesError::JuriesPoolIsToSmall));
        }

        // Test juries assignment to the dispute case.
        #[ink::test]
        fn assign_juries_to_dispute_fail_for_owner() {
            let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
            set_caller::<DefaultEnvironment>(accounts.alice);

            let mut bright_disputes = BrightDisputes::new();

            // Alice creates a dispute
            let dispute_id = bright_disputes
                .create_dispute("https://brightinventions.pl/".into(), accounts.bob, 10)
                .expect("Failed to create a dispute!");

            // Alice register as a jure in the pool
            bright_disputes
                .register_as_an_active_jure()
                .expect("Failed to register alice as a jure!");
            assert_eq!(bright_disputes.juries_pool.len(), 1);

            // Fail with assigning Alice as a Jure in her own dispute
            let result = bright_disputes.assign_juries_to_dispute(dispute_id, 1);
            assert_eq!(result, Err(BrightDisputesError::JuriesPoolIsToSmall));
            assert_eq!(bright_disputes.juries_pool.len(), 1);
        }

        // Test juries assignment to the dispute case.
        #[ink::test]
        fn assign_juries_to_dispute_fail_for_defendant() {
            let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
            set_caller::<DefaultEnvironment>(accounts.alice);

            let mut bright_disputes = BrightDisputes::new();

            // Alice creates a dispute and bob is an defendant
            let dispute_id = bright_disputes
                .create_dispute("https://brightinventions.pl/".into(), accounts.bob, 10)
                .expect("Failed to create a dispute!");

            // Bob register as a jure in the pool
            set_caller::<DefaultEnvironment>(accounts.bob);
            bright_disputes
                .register_as_an_active_jure()
                .expect("Failed to register bob as a jure!");
            assert_eq!(bright_disputes.juries_pool.len(), 1);

            // Fail with assigning Bob as a Jure when he is a defendant of the dispute
            let result = bright_disputes.assign_juries_to_dispute(dispute_id, 1);
            assert_eq!(result, Err(BrightDisputesError::JuriesPoolIsToSmall));
            assert_eq!(bright_disputes.juries_pool.len(), 1);
        }
    }
}
