#[ink::contract(env = baby_liminal_extension::ink::BabyLiminalEnvironment)]
mod bright_disputes {

    use ink::prelude::string::String;

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
    pub struct DefendantConfirmDispute {
        id: DisputeId,
        defendant_id: AccountId,
    }

    /// Main contract storage
    #[ink(storage)]
    pub struct BrightDisputes {
        last_dispute_id: DisputeId,
        dispute: Option<Dispute>,
    }

    impl BrightDisputes {
        /// Constructor
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                last_dispute_id: 0,
                dispute: None,
            }
        }

        /// Create new dispute
        #[ink(message)]
        pub fn create_dispute(
            &mut self,
            owner_link: String,
            defendant_id: AccountId,
            escrow: Balance,
        ) -> Result<()> {
            if self.dispute.is_some() {
                return Err(BrightDisputesError::DisputeAlreadyCreated);
            }
            let owner_id = ink::env::caller::<ink::env::DefaultEnvironment>();
            self.last_dispute_id = self.generate_dispute_id()?;
            self.dispute = Some(Dispute::create(
                self.last_dispute_id,
                owner_link,
                defendant_id,
                escrow,
            ));

            self.env().emit_event(DisputeRaised {
                id: self.last_dispute_id,
                owner_id,
                defendant_id,
            });

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
            dispute.confirm_defendant(defendant_link, escrow)?;

            self.env().emit_event(DefendantConfirmDispute {
                id: dispute.id(),
                defendant_id: ink::env::caller::<ink::env::DefaultEnvironment>(),
            });

            self.dispute = Some(dispute);

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
            Ok(())
        }

        fn generate_dispute_id(&self) -> Result<DisputeId> {
            if let Some(id) = self.last_dispute_id.checked_add(1) {
                Ok(id)
            } else {
                Err(BrightDisputesError::InvalidAction)
            }
        }

        fn get_dispute_or_assert(&self, dispute_id: DisputeId) -> Result<Dispute> {
            if let Some(d) = &self.dispute {
                if d.id() == dispute_id {
                    return Ok(d.clone());
                }
            }
            return Err(BrightDisputesError::DisputeNotExist);
        }
    }

    #[cfg(test)]
    mod tests {
        use ink::env::{test::set_caller, DefaultEnvironment};

        use super::*;

        /// We test if we can create only one single dispute.
        #[ink::test]
        fn create_single_dispute() {
            let mut bright_disputes = BrightDisputes::new();
            assert_eq!(bright_disputes.dispute, None);

            let accounts = ink::env::test::default_accounts::<DefaultEnvironment>();
            let owner_link = "https://brightinventions.pl/";
            let escrow_amount: Balance = 15;
            set_caller::<DefaultEnvironment>(accounts.alice);
            let result =
                bright_disputes.create_dispute(owner_link.into(), accounts.bob, escrow_amount);
            assert_eq!(result, Ok(()));

            let result =
                bright_disputes.create_dispute(owner_link.into(), accounts.bob, escrow_amount);
            assert_eq!(result, Err(BrightDisputesError::DisputeAlreadyCreated));
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
