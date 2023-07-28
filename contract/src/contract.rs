#[ink::contract(env = baby_liminal_extension::ink::BabyLiminalEnvironment)]
pub mod bright_disputes {

    use ink::{
        prelude::{string::String, vec::Vec},
        storage::Mapping,
    };

    use crate::{
        dispute::{Dispute, DisputeResult},
        dispute_round::DisputeRound,
        error::BrightDisputesError,
        juror::{Juror, JuriesMap},
        types::{DisputeId, Result, VoteValue},
        vote::Vote,
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

    #[ink(event)]
    pub struct DisputeResultEvent {
        id: DisputeId,
        result: DisputeResult,
    }

    /// Main contract storage
    #[ink(storage)]
    #[derive(Default)]
    pub struct BrightDisputes {
        last_dispute_id: DisputeId,
        juries_pool: Vec<AccountId>,
        juries: Mapping<AccountId, Juror>,
        disputes: Mapping<DisputeId, Dispute>,
    }

    impl JuriesMap for BrightDisputes {
        fn get_juror_or_assert(&self, juror_id: AccountId) -> Result<Juror> {
            self.juries
                .get(juror_id)
                .ok_or(BrightDisputesError::JurorNotExist)
        }

        fn remove_random_juries_from_pool_or_assert(
            &mut self,
            except: &Vec<AccountId>,
            number: u8,
        ) -> Result<Vec<AccountId>> {
            let juries_ids = self.get_random_juries_from_pool(&except, number, 123 as u64)?;
            for id in &juries_ids {
                self.remove_juror_from_pool_or_assert(*id)?;
            }
            Ok(juries_ids)
        }

        /// Update juror
        fn update_juror(&mut self, juror: Juror) {
            self.juries.insert(juror.id(), &juror);
        }
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

        /// Get last dispute id
        #[ink(message)]
        pub fn get_last_dispute_id(&self) -> DisputeId {
            self.last_dispute_id
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
            let dispute = self.get_dispute_or_assert(dispute_id)?;
            dispute.assert_dispute_remove()?;
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
            self.assert_transferred(escrow)?;
            let owner_id = ink::env::caller::<ink::env::DefaultEnvironment>();
            self.last_dispute_id = self.generate_dispute_id()?;
            let dispute = Dispute::create(self.last_dispute_id, owner_link, defendant_id, escrow);
            self.update_dispute(dispute);

            self.env().emit_event(DisputeRaised {
                id: self.last_dispute_id,
                owner_id,
                defendant_id,
            });

            Ok(self.last_dispute_id)
        }

        /// Defendant confirms his participation in dispute.
        #[ink(message, payable)]
        pub fn confirm_defendant(
            &mut self,
            dispute_id: DisputeId,
            defendant_link: String,
        ) -> Result<()> {
            let mut dispute = self.get_dispute_or_assert(dispute_id)?;
            let id = dispute.id();
            self.assert_transferred(dispute.escrow())?;
            dispute.confirm_defendant(defendant_link)?;
            dispute.set_dispute_round(DisputeRound::create(self.env().block_timestamp(), None));
            dispute.increment_deposit();
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

        /// Voting, only juror can do it.
        #[ink(message)]
        pub fn vote(&mut self, dispute_id: DisputeId, vote: VoteValue) -> Result<()> {
            let caller = ink::env::caller::<ink::env::DefaultEnvironment>();
            let mut dispute = self.get_dispute_or_assert(dispute_id)?;
            let mut juror = self.get_juror_or_assert(caller)?;
            dispute.vote(Vote::create(caller, vote))?;
            juror.action_done(dispute.id())?;
            self.update_juror(juror);
            self.update_dispute(dispute);
            Ok(())
        }

        /// Register as an active juror. Juries are picked
        /// from this pool to participate in disputes.
        #[ink(message)]
        pub fn register_as_an_active_juror(&mut self) -> Result<()> {
            let caller = ink::env::caller::<ink::env::DefaultEnvironment>();
            self.assert_juror_not_in_pool(caller)?;

            let juror = self
                .juries
                .get(caller)
                .unwrap_or_else(|| Juror::create(caller));
            self.juries_pool.push(juror.id());
            self.update_juror(juror);
            Ok(())
        }

        /// Unregister juror from the active juries pool.
        #[ink(message)]
        pub fn unregister_as_an_active_juror(&mut self) -> Result<()> {
            let caller = ink::env::caller::<ink::env::DefaultEnvironment>();
            self.remove_juror_from_pool_or_assert(caller)?;
            Ok(())
        }

        /// Assigned juror can confirm his participation in dispute
        #[ink(message, payable)]
        pub fn confirm_juror_participation_in_dispute(
            &mut self,
            dispute_id: DisputeId,
        ) -> Result<()> {
            let mut dispute = self.get_dispute_or_assert(dispute_id)?;
            self.assert_transferred(dispute.escrow())?;

            let caller = ink::env::caller::<ink::env::DefaultEnvironment>();
            let mut juror = self.get_juror_or_assert(caller)?;
            juror.confirm_participation_in_dispute(dispute_id)?;
            self.update_juror(juror);

            dispute.increment_deposit();
            self.update_dispute(dispute);
            Ok(())
        }

        /// Judge can confirm his participation in dispute
        #[ink(message, payable)]
        pub fn confirm_judge_participation_in_dispute(
            &mut self,
            dispute_id: DisputeId,
        ) -> Result<()> {
            let mut dispute = self.get_dispute_or_assert(dispute_id)?;
            self.assert_transferred(dispute.escrow())?;

            let caller = ink::env::caller::<ink::env::DefaultEnvironment>();
            let mut juror = self.get_juror_or_assert(caller)?;
            juror.confirm_participation_in_dispute(dispute_id)?;
            self.update_juror(juror);

            dispute.increment_deposit();
            self.update_dispute(dispute);
            Ok(())
        }

        /// Unregister juror from the active juries pool.
        #[ink(message)]
        pub fn process_dispute_round(&mut self, dispute_id: DisputeId) -> Result<()> {
            let timestamp = self.env().block_timestamp();
            let mut dispute = self.get_dispute_or_assert(dispute_id)?;
            if let Err(e) = dispute.process_dispute_round(self, timestamp) {
                match e {
                    BrightDisputesError::DisputeRoundDeadlineReached => {
                        // Check if judge counted the votes.
                        if let Some(judge_id) = dispute.judge() {
                            let judge = self.get_juror_or_assert(judge_id)?;
                            if judge.is_requested_for_action(dispute_id) {
                                dispute.move_to_banned(judge_id)?;
                            }
                        }

                        // Check if juries votes
                        for juror_id in dispute.juries() {
                            let juror = self.get_juror_or_assert(juror_id)?;
                            if juror.is_requested_for_action(dispute_id) {
                                dispute.move_to_banned(juror_id)?;
                            }
                        }

                        // Whenever deadline is reached, we start a new dispute round.
                        dispute.on_dispute_round_deadline(timestamp)?;
                    }

                    BrightDisputesError::MajorityOfVotesNotReached => {
                        // Check if juries votes
                        for juror_id in dispute.juries() {
                            let juror = self.get_juror_or_assert(juror_id)?;
                            if juror.is_requested_for_action(dispute_id) {
                                dispute.move_to_banned(juror_id)?;
                            }
                        }

                        // Dispute round ended, but the majority of votes is not reached.
                        if let Err(_) = dispute.next_dispute_round(timestamp) {
                            return Err(e);
                        }
                    }
                    _ => {
                        self.update_dispute(dispute);
                        return Err(e);
                    }
                }
            }
            self.update_dispute(dispute);
            Ok(())
        }

        /// Judge can confirm his participation in dispute
        #[ink(message)]
        pub fn distribute_deposit(&mut self, dispute_id: DisputeId) -> Result<()> {
            let mut dispute = self.get_dispute_or_assert(dispute_id)?;

            // Check if dispute has ended.
            dispute.assert_dispute_ended()?;

            // Close dispute
            dispute.close_dispute()?;
            self.update_dispute(dispute.clone());

            let mut accounts: Vec<AccountId> = Vec::new();

            // Add owner
            accounts.push(dispute.owner());

            // Add defendant, only if he confirmed dispute
            if dispute.has_defendant_confirmed_dispute() {
                accounts.push(dispute.defendant());
            }

            // Add judge
            if let Some(judge_id) = dispute.judge() {
                accounts.push(judge_id);
            }

            // Add juries, who were not banned.
            for juror_id in dispute.juries() {
                accounts.push(juror_id);
            }

            // Split deposit and transfer founds.
            let founds = dispute.deposit() / accounts.len() as Balance;
            for account in accounts {
                self.env().transfer(account, founds)?;
            }

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

        fn remove_juror_from_pool_or_assert(&mut self, juror_id: AccountId) -> Result<()> {
            if let Some(index) = self.juries_pool.iter().position(|&j| j == juror_id) {
                self.juries_pool.remove(index);
                return Ok(());
            }
            return Err(BrightDisputesError::NotRegisteredAsJuror);
        }

        fn get_random_juries_from_pool(
            &self,
            except_juries: &Vec<AccountId>,
            number_of_juries: u8,
            seed: u64,
        ) -> Result<Vec<AccountId>> {
            let filtered_pool: Vec<AccountId> = self
                .juries_pool
                .iter()
                .filter(|juror_id| !except_juries.contains(&juror_id))
                .map(|&juror_id| juror_id)
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

        fn assert_juror_not_in_pool(&self, juror_id: AccountId) -> Result<()> {
            for j in &self.juries_pool {
                if *j == juror_id {
                    return Err(BrightDisputesError::JurorAlreadyRegistered);
                }
            }
            return Ok(());
        }

        fn assert_transferred(&self, expected_amount: Balance) -> Result<()> {
            let transferred = self.env().transferred_value();
            if transferred != expected_amount {
                return Err(BrightDisputesError::InvalidEscrowAmount);
            }
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use ink::env::{
            test::{set_caller, set_value_transferred},
            DefaultEnvironment,
        };

        use super::*;

        fn create_test_bright_dispute_with_running_dispute() -> BrightDisputes {
            let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
            let mut bright_disputes = BrightDisputes::new();

            set_value_transferred::<DefaultEnvironment>(10);

            // Alice creates a dispute
            let dispute_id = bright_disputes
                .create_dispute("https://brightinventions.pl/".into(), accounts.bob, 10)
                .expect("Failed to create a dispute!");

            // Confirm bob participation
            set_caller::<DefaultEnvironment>(accounts.bob);
            bright_disputes
                .confirm_defendant(dispute_id, "".into())
                .expect("Failed to confirm defendant a dispute!");
            return bright_disputes;
        }

        fn register_valid_juries(bright_disputes: &mut BrightDisputes) {
            let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
            set_caller::<DefaultEnvironment>(accounts.charlie);
            bright_disputes
                .register_as_an_active_juror()
                .expect("Failed to register charlie as a juror!");
            set_caller::<DefaultEnvironment>(accounts.eve);
            bright_disputes
                .register_as_an_active_juror()
                .expect("Failed to register eve as a juror!");
            set_caller::<DefaultEnvironment>(accounts.frank);
            bright_disputes
                .register_as_an_active_juror()
                .expect("Failed to register frank as a juror!");
            set_caller::<DefaultEnvironment>(accounts.django);
            bright_disputes
                .register_as_an_active_juror()
                .expect("Failed to register django as a juror!");

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
            set_value_transferred::<DefaultEnvironment>(escrow_amount);

            let result =
                bright_disputes.create_dispute(owner_link.into(), accounts.bob, escrow_amount);
            assert_eq!(result, Ok(1));

            // Failed, escrow amount doesn't match the transferred value.
            let result =
                bright_disputes.create_dispute(owner_link.into(), accounts.bob, escrow_amount + 1);
            assert_eq!(result, Err(BrightDisputesError::InvalidEscrowAmount));
        }

        /// Test if we can create multiple disputes.
        #[ink::test]
        fn create_multiple_dispute() {
            let mut bright_disputes = BrightDisputes::new();

            let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
            set_caller::<DefaultEnvironment>(accounts.alice);
            set_value_transferred::<DefaultEnvironment>(10);

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

            set_value_transferred::<DefaultEnvironment>(10);

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

            set_value_transferred::<DefaultEnvironment>(10);

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

            // Create dispute
            set_value_transferred::<DefaultEnvironment>(10);
            bright_disputes
                .create_dispute("https://brightinventions.pl".into(), accounts.bob, 10)
                .expect("Failed to create a dispute!");
            let result = bright_disputes.remove_dispute(1);
            assert_eq!(result, Ok(()));

            // Failed to remove dispute in "Running" state.
            let mut bright_disputes = create_test_bright_dispute_with_running_dispute();
            let result = bright_disputes.remove_dispute(1);
            assert_eq!(result, Err(BrightDisputesError::InvalidDisputeState));

            // Failed to remove dispute in "Ended" state.
            let mut dispute = bright_disputes
                .get_dispute(1)
                .expect("Failed to get dispute!");
            dispute
                .end_dispute(DisputeResult::Owner)
                .expect("Failed to end dispute!");
            bright_disputes.update_dispute(dispute.clone());
            let result = bright_disputes.remove_dispute(1);
            assert_eq!(result, Err(BrightDisputesError::InvalidDisputeState));

            // Success
            dispute.close_dispute().expect("Failed to close dispute!");
            bright_disputes.update_dispute(dispute);
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

            set_value_transferred::<DefaultEnvironment>(10);
            let dispute_id = bright_disputes
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

            // Check if dispute round was started.
            let dispute = bright_disputes
                .get_dispute(dispute_id)
                .expect("Failed to get dispute!");
            assert!(dispute.dispute_round().is_some());
        }

        /// Test dispute owner description update
        #[ink::test]
        fn update_owner_description() {
            let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
            set_caller::<DefaultEnvironment>(accounts.alice);

            let mut bright_disputes = create_test_bright_dispute_with_running_dispute();

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
            set_value_transferred::<DefaultEnvironment>(10);

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

        /// Test juror registration
        #[ink::test]
        fn juror_register() {
            let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
            set_caller::<DefaultEnvironment>(accounts.alice);

            let mut bright_disputes = BrightDisputes::new();
            assert_eq!(bright_disputes.juries_pool.len(), 0);
            assert!(!bright_disputes.juries.contains(accounts.alice));

            // Success
            let result = bright_disputes.register_as_an_active_juror();
            assert_eq!(result, Ok(()));
            assert_eq!(bright_disputes.juries_pool.len(), 1);
            assert!(bright_disputes.juries.contains(accounts.alice));

            // Failed to register already registered juror
            let result = bright_disputes.register_as_an_active_juror();
            assert_eq!(result, Err(BrightDisputesError::JurorAlreadyRegistered));
            assert_eq!(bright_disputes.juries_pool.len(), 1);
            assert!(bright_disputes.juries.contains(accounts.alice));
        }

        /// Test juror unregistration
        #[ink::test]
        fn juror_unregister() {
            let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
            set_caller::<DefaultEnvironment>(accounts.alice);

            let mut bright_disputes = BrightDisputes::new();

            // Failed to unregister juror, no juror in the pool
            let result = bright_disputes.unregister_as_an_active_juror();
            assert_eq!(result, Err(BrightDisputesError::NotRegisteredAsJuror));
            assert_eq!(bright_disputes.juries_pool.len(), 0);
            assert!(!bright_disputes.juries.contains(accounts.alice));

            bright_disputes
                .register_as_an_active_juror()
                .expect("Failed to register a juror!");

            // Success
            let result = bright_disputes.unregister_as_an_active_juror();
            assert_eq!(result, Ok(()));
            assert_eq!(bright_disputes.juries_pool.len(), 0);
            assert!(bright_disputes.juries.contains(accounts.alice));

            // Failed to unregister juror, juror already unregistered
            let result = bright_disputes.unregister_as_an_active_juror();
            assert_eq!(result, Err(BrightDisputesError::NotRegisteredAsJuror));
        }

        // Test juries confirmation to the dispute case.
        #[ink::test]
        fn confirm_judge_and_juror_participation_in_dispute() {
            let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
            set_caller::<DefaultEnvironment>(accounts.alice);

            let mut bright_disputes = create_test_bright_dispute_with_running_dispute();
            let dispute_id = 1;

            // Register charlie, django, eve and frank as a juries.
            register_valid_juries(&mut bright_disputes);

            // Fail to confirm juror, when he is not assigned.
            set_caller::<DefaultEnvironment>(accounts.charlie);
            let result = bright_disputes.confirm_juror_participation_in_dispute(dispute_id);
            assert_eq!(result, Err(BrightDisputesError::JurorIsNotAssignedToDispute));

            // Switch to "PickingJuriesAndJudge" state.
            set_caller::<DefaultEnvironment>(accounts.alice);
            set_value_transferred::<DefaultEnvironment>(10);
            bright_disputes
                .process_dispute_round(dispute_id)
                .expect("Failed to process dispute round!");

            let dispute = bright_disputes
                .get_dispute(dispute_id)
                .expect("Failed to get dispute!");

            let judge = dispute.judge().expect("Judge was not assigned!");

            // Fail to confirm judge, invalid escrow
            set_caller::<DefaultEnvironment>(judge);
            set_value_transferred::<DefaultEnvironment>(5);
            let result = bright_disputes.confirm_judge_participation_in_dispute(dispute_id);
            assert_eq!(result, Err(BrightDisputesError::InvalidEscrowAmount));

            // Success, confirm judge
            set_caller::<DefaultEnvironment>(judge);
            set_value_transferred::<DefaultEnvironment>(10);
            let result = bright_disputes.confirm_judge_participation_in_dispute(dispute_id);
            assert_eq!(result, Ok(()));

            // Confirm juries
            for juror in &dispute.juries() {
                set_caller::<DefaultEnvironment>(*juror);

                // Fail to confirm juror, invalid escrow
                set_value_transferred::<DefaultEnvironment>(5);
                assert_eq!(
                    bright_disputes.confirm_juror_participation_in_dispute(dispute_id),
                    Err(BrightDisputesError::InvalidEscrowAmount)
                );

                // Success
                set_value_transferred::<DefaultEnvironment>(10);
                assert_eq!(
                    bright_disputes.confirm_juror_participation_in_dispute(dispute_id),
                    Ok(())
                );
            }

            // Failed to confirm twice
            let result = bright_disputes.confirm_juror_participation_in_dispute(dispute_id);
            assert_eq!(
                result,
                Err(BrightDisputesError::JurorAlreadyConfirmedDispute)
            );

            // Switch to "Voting" state.
            set_caller::<DefaultEnvironment>(accounts.alice);
            bright_disputes
                .process_dispute_round(dispute_id)
                .expect("Failed to process dispute round!");
        }

        // Check switching to next round when deadline appear.
        #[ink::test]
        fn vote() {
            let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
            set_caller::<DefaultEnvironment>(accounts.alice);

            let mut bright_disputes = create_test_bright_dispute_with_running_dispute();
            let dispute_id = 1;

            // Register charlie, eve, frank  and django as a juries.
            register_valid_juries(&mut bright_disputes);

            // Switch to "PickingJuriesAndJudge" state.
            set_caller::<DefaultEnvironment>(accounts.alice);
            bright_disputes
                .process_dispute_round(dispute_id)
                .expect("Failed to create a dispute!");

            let dispute = bright_disputes
                .get_dispute(dispute_id)
                .expect("Failed to get dispute!");

            // Assign all juries
            let assigned_juries = dispute.juries();

            let mut juror_not_assigned = vec![
                accounts.charlie,
                accounts.eve,
                accounts.frank,
                accounts.django,
            ];
            juror_not_assigned.retain(|id| !assigned_juries.contains(id));

            // Confirm juror participation
            set_caller::<DefaultEnvironment>(assigned_juries[0]);
            bright_disputes
                .confirm_juror_participation_in_dispute(dispute_id)
                .expect("Failed confirm juries participation!");

            set_caller::<DefaultEnvironment>(assigned_juries[1]);
            bright_disputes
                .confirm_juror_participation_in_dispute(dispute_id)
                .expect("Failed confirm juries participation!");

            set_caller::<DefaultEnvironment>(assigned_juries[2]);
            bright_disputes
                .confirm_juror_participation_in_dispute(dispute_id)
                .expect("Failed confirm juries participation!");

            // Assign judge
            set_caller::<DefaultEnvironment>(juror_not_assigned[0]);
            bright_disputes
                .confirm_judge_participation_in_dispute(dispute_id)
                .expect("Failed to confirm judge participation!");

            // Switch state to "Voting" state
            set_caller::<DefaultEnvironment>(accounts.alice);
            let result = bright_disputes.process_dispute_round(dispute_id);
            assert_eq!(result, Ok(()));

            // Juries voting
            set_caller::<DefaultEnvironment>(assigned_juries[0]);
            bright_disputes.vote(dispute_id, 1).expect("Failed to vote");
            set_caller::<DefaultEnvironment>(assigned_juries[1]);
            bright_disputes.vote(dispute_id, 1).expect("Failed to vote");
            set_caller::<DefaultEnvironment>(assigned_juries[2]);
            bright_disputes.vote(dispute_id, 1).expect("Failed to vote");

            // Switch state to "CountingTheVotes" state
            set_caller::<DefaultEnvironment>(accounts.alice);
            let result = bright_disputes.process_dispute_round(dispute_id);
            assert_eq!(result, Ok(()));
        }

        // Check switching to next round can only be done by owner.
        #[ink::test]
        fn process_dispute_round_owner_call() {
            let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
            set_caller::<DefaultEnvironment>(accounts.alice);

            let mut bright_disputes = create_test_bright_dispute_with_running_dispute();
            let dispute_id = 1;

            // Register charlie, eve, frank  and django as a juries.
            register_valid_juries(&mut bright_disputes);

            // Failed, defendant can not switch the state.
            set_caller::<DefaultEnvironment>(accounts.bob);
            let result = bright_disputes.process_dispute_round(dispute_id);
            assert_eq!(result, Err(BrightDisputesError::NotAuthorized));

            // Success
            set_caller::<DefaultEnvironment>(accounts.alice);
            let result = bright_disputes.process_dispute_round(dispute_id);
            assert_eq!(result, Ok(()));
        }

        // Check dispute round progress
        #[ink::test]
        fn process_dispute_round() {
            let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
            set_caller::<DefaultEnvironment>(accounts.alice);

            let mut bright_disputes = create_test_bright_dispute_with_running_dispute();
            let dispute_id = 1;

            // Register charlie, eve, frank  and django as a juries.
            register_valid_juries(&mut bright_disputes);

            // Switch to "PickingJuriesAndJudge" state.
            set_caller::<DefaultEnvironment>(accounts.alice);
            bright_disputes
                .process_dispute_round(dispute_id)
                .expect("Failed to create a dispute!");

            let dispute = bright_disputes
                .get_dispute(dispute_id)
                .expect("Failed to get dispute!");

            // Assign all juries
            let assigned_juries = dispute.juries();

            let mut juror_not_assigned = vec![
                accounts.charlie,
                accounts.eve,
                accounts.frank,
                accounts.django,
            ];
            juror_not_assigned.retain(|id| !assigned_juries.contains(id));

            // Confirm juror participation
            set_caller::<DefaultEnvironment>(assigned_juries[0]);
            bright_disputes
                .confirm_juror_participation_in_dispute(dispute_id)
                .expect("Failed confirm juries participation!");

            set_caller::<DefaultEnvironment>(assigned_juries[1]);
            bright_disputes
                .confirm_juror_participation_in_dispute(dispute_id)
                .expect("Failed confirm juries participation!");

            set_caller::<DefaultEnvironment>(assigned_juries[2]);
            bright_disputes
                .confirm_juror_participation_in_dispute(dispute_id)
                .expect("Failed confirm juries participation!");

            // Assign judge
            set_caller::<DefaultEnvironment>(juror_not_assigned[0]);
            bright_disputes
                .confirm_judge_participation_in_dispute(dispute_id)
                .expect("Failed to confirm judge participation!");

            // Switch state to "Voting" state
            set_caller::<DefaultEnvironment>(accounts.alice);
            let result = bright_disputes.process_dispute_round(dispute_id);
            assert_eq!(result, Ok(()));

            // Juries voting
            set_caller::<DefaultEnvironment>(assigned_juries[0]);
            bright_disputes.vote(dispute_id, 1).expect("Failed to vote");
            set_caller::<DefaultEnvironment>(assigned_juries[1]);
            bright_disputes.vote(dispute_id, 1).expect("Failed to vote");
            set_caller::<DefaultEnvironment>(assigned_juries[2]);
            bright_disputes.vote(dispute_id, 1).expect("Failed to vote");

            // Switch state to CountingTheVotes
            set_caller::<DefaultEnvironment>(accounts.alice);
            let result = bright_disputes.process_dispute_round(dispute_id);
            assert_eq!(result, Ok(()));

            // Failed to count the votes, only judge can do it.
            set_caller::<DefaultEnvironment>(accounts.alice);
            let result = bright_disputes.process_dispute_round(dispute_id);
            assert_eq!(result, Err(BrightDisputesError::NotAuthorized));

            // Count the votes, dispute ends
            set_caller::<DefaultEnvironment>(juror_not_assigned[0]);
            let result = bright_disputes.process_dispute_round(dispute_id);
            assert_eq!(result, Ok(()));
        }

        // Check switching to next rounds.
        #[ink::test]
        fn process_dispute_round_next_round() {
            let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
            set_caller::<DefaultEnvironment>(accounts.alice);

            let mut bright_disputes = create_test_bright_dispute_with_running_dispute();
            let dispute_id = 1;

            // Register charlie, eve, frank  and django as a juries.
            register_valid_juries(&mut bright_disputes);

            // Switch to "PickingJuriesAndJudge" state.
            set_caller::<DefaultEnvironment>(accounts.alice);
            bright_disputes
                .process_dispute_round(dispute_id)
                .expect("Failed to create a dispute!");

            let dispute = bright_disputes
                .get_dispute(dispute_id)
                .expect("Failed to get dispute!");

            // Assign all juries
            let assigned_juries = dispute.juries();

            let mut juror_not_assigned = vec![
                accounts.charlie,
                accounts.eve,
                accounts.frank,
                accounts.django,
            ];
            juror_not_assigned.retain(|id| !assigned_juries.contains(id));

            // Confirm juror participation
            set_caller::<DefaultEnvironment>(assigned_juries[0]);
            bright_disputes
                .confirm_juror_participation_in_dispute(dispute_id)
                .expect("Failed confirm juries participation!");

            set_caller::<DefaultEnvironment>(assigned_juries[1]);
            bright_disputes
                .confirm_juror_participation_in_dispute(dispute_id)
                .expect("Failed confirm juries participation!");

            set_caller::<DefaultEnvironment>(assigned_juries[2]);
            bright_disputes
                .confirm_juror_participation_in_dispute(dispute_id)
                .expect("Failed confirm juries participation!");

            // Assign judge
            set_caller::<DefaultEnvironment>(juror_not_assigned[0]);
            bright_disputes
                .confirm_judge_participation_in_dispute(dispute_id)
                .expect("Failed to confirm judge participation!");

            // Switch state to "Voting" state
            set_caller::<DefaultEnvironment>(accounts.alice);
            bright_disputes
                .process_dispute_round(dispute_id)
                .expect("Failed to process dispute round!");

            // Juries voting
            set_caller::<DefaultEnvironment>(assigned_juries[0]);
            bright_disputes.vote(dispute_id, 1).expect("Failed to vote");
            set_caller::<DefaultEnvironment>(assigned_juries[1]);
            bright_disputes.vote(dispute_id, 1).expect("Failed to vote");
            set_caller::<DefaultEnvironment>(assigned_juries[2]);
            bright_disputes.vote(dispute_id, 0).expect("Failed to vote");

            // Switch state to "CountingTheVotes" state
            set_caller::<DefaultEnvironment>(accounts.alice);
            let result = bright_disputes.process_dispute_round(dispute_id);
            assert_eq!(result, Ok(()));

            // Majority of votes not reached, new round.
            set_caller::<DefaultEnvironment>(juror_not_assigned[0]);
            let result = bright_disputes.process_dispute_round(dispute_id);
            assert_eq!(result, Ok(()));

            // New round
            set_caller::<DefaultEnvironment>(accounts.alice);
            let result = bright_disputes.process_dispute_round(dispute_id);
            assert_eq!(result, Err(BrightDisputesError::JuriesPoolIsToSmall));
        }

        // Check deposit distribution
        #[ink::test]
        fn distribute_deposit() {
            let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
            set_caller::<DefaultEnvironment>(accounts.alice);

            let mut bright_disputes = create_test_bright_dispute_with_running_dispute();
            let dispute_id = 1;

            // Register charlie, eve, frank  and django as a juries.
            register_valid_juries(&mut bright_disputes);

            // Switch to "PickingJuriesAndJudge" state.
            set_caller::<DefaultEnvironment>(accounts.alice);
            bright_disputes
                .process_dispute_round(dispute_id)
                .expect("Failed to create a dispute!");

            let dispute = bright_disputes
                .get_dispute(dispute_id)
                .expect("Failed to get dispute!");

            // Assign all juries
            let assigned_juries = dispute.juries();

            let mut juror_not_assigned = vec![
                accounts.charlie,
                accounts.eve,
                accounts.frank,
                accounts.django,
            ];
            juror_not_assigned.retain(|id| !assigned_juries.contains(id));

            // Confirm juror participation
            set_caller::<DefaultEnvironment>(assigned_juries[0]);
            bright_disputes
                .confirm_juror_participation_in_dispute(dispute_id)
                .expect("Failed confirm juries participation!");

            set_caller::<DefaultEnvironment>(assigned_juries[1]);
            bright_disputes
                .confirm_juror_participation_in_dispute(dispute_id)
                .expect("Failed confirm juries participation!");

            set_caller::<DefaultEnvironment>(assigned_juries[2]);
            bright_disputes
                .confirm_juror_participation_in_dispute(dispute_id)
                .expect("Failed confirm juries participation!");

            // Assign judge
            set_caller::<DefaultEnvironment>(juror_not_assigned[0]);
            bright_disputes
                .confirm_judge_participation_in_dispute(dispute_id)
                .expect("Failed to confirm judge participation!");

            // Switch state to "Voting" state
            set_caller::<DefaultEnvironment>(accounts.alice);
            bright_disputes
                .process_dispute_round(dispute_id)
                .expect("Failed process dispute round!");

            // Juries voting
            set_caller::<DefaultEnvironment>(assigned_juries[0]);
            bright_disputes.vote(dispute_id, 1).expect("Failed to vote");
            set_caller::<DefaultEnvironment>(assigned_juries[1]);
            bright_disputes.vote(dispute_id, 1).expect("Failed to vote");
            set_caller::<DefaultEnvironment>(assigned_juries[2]);
            bright_disputes.vote(dispute_id, 1).expect("Failed to vote");

            // Switch state to CountingTheVotes
            set_caller::<DefaultEnvironment>(accounts.alice);
            bright_disputes
                .process_dispute_round(dispute_id)
                .expect("Failed process dispute round!");

            // Count the votes, dispute ends
            set_caller::<DefaultEnvironment>(juror_not_assigned[0]);
            bright_disputes
                .process_dispute_round(dispute_id)
                .expect("Failed process dispute round!");

            set_caller::<DefaultEnvironment>(accounts.bob);
            let result = bright_disputes.distribute_deposit(dispute_id);
            assert_eq!(result, Ok(()));
        }
    }
}
