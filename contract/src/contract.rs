#[ink::contract(env = baby_liminal_extension::ink::BabyLiminalEnvironment)]
mod bright_disputes {

    /// Main contract storage
    #[ink(storage)]
    #[derive(Default)]
    pub struct BrightDisputes {
        value: bool,
    }

    impl BrightDisputes {
        /// Constructor
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        /// A message that can be called on instantiated contracts.
        /// This one flips the value of the stored `bool` from `true`
        /// to `false` and vice versa.
        #[ink(message, selector = 1)]
        pub fn flip(&mut self) {
            self.value = !self.value;
        }

        /// Simply returns the current value of our `bool`.
        #[ink(message, selector = 2)]
        pub fn get(&self) -> bool {
            self.value
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let bright_disputes = BrightDisputes::default();
            assert_eq!(bright_disputes.get(), false);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let mut bright_disputes = BrightDisputes::new(false);
            assert_eq!(bright_disputes.get(), false);
            bright_disputes.flip();
            assert_eq!(bright_disputes.get(), true);
        }
    }
}
