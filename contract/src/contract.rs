#[ink::contract(env = baby_liminal_extension::ink::BabyLiminalEnvironment)]
mod bright_disputes {

    use ink::{
        prelude::{string::String, vec::Vec},
        storage::Mapping,
    };

    use crate::{
        dispute::Dispute,
        error::BrightDisputesError,
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
        disputes: Mapping<DisputeId, Dispute>,
    }

    impl BrightDisputes {
        /// Constructor
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                last_dispute_id: 0,
                disputes: Mapping::default(),
            }
        }

        /// Create new dispute
        #[ink(message)]
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

            self.env().emit_event(DisputeRaised {
                id: self.last_dispute_id,
                owner_id,
                defendant_id,
            });

            Ok(self.last_dispute_id)
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

        /// Defendant confirms his participation in dispute.
        #[ink(message)]
        pub fn confirm_defendant(
            &mut self,
            dispute_id: DisputeId,
            defendant_link: String,
            escrow: Balance,
        ) -> Result<()> {
            let mut dispute = self.get_dispute_or_assert(dispute_id)?;
            let id = dispute.id();
            dispute.confirm_defendant(defendant_link, escrow)?;
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
    }

    #[cfg(test)]
    mod tests {
        use ink::env::{test::set_caller, DefaultEnvironment};

        use super::*;

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
            let result = bright_disputes.confirm_defendant(1, defendant_link.into(), 30);
            assert_eq!(result, Err(BrightDisputesError::DisputeNotExist));

            bright_disputes
                .create_dispute("".into(), accounts.bob, 10)
                .expect("Failed to create a dispute!");

            // Check when dispute exist, but there someone else try to assign
            let result = bright_disputes.confirm_defendant(1, defendant_link.into(), 30);
            assert_eq!(result, Err(BrightDisputesError::NotAuthorized));

            // Check when dispute exist, but call refers to wrong dispute.
            let result = bright_disputes.confirm_defendant(0, defendant_link.into(), 30);
            assert_eq!(result, Err(BrightDisputesError::DisputeNotExist));

            // Check when defendant assign.
            set_caller::<DefaultEnvironment>(accounts.bob);
            let result = bright_disputes.confirm_defendant(1, defendant_link.into(), 30);
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
    }
}
