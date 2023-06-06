use ink::primitives::AccountId;
#[derive(Clone, Debug, PartialEq, scale::Decode, scale::Encode)]
#[cfg_attr(
    feature = "std",
    derive(ink::storage::traits::StorageLayout, scale_info::TypeInfo)
)]
pub struct Jure {
    id: AccountId,
}

impl Jure {
    #[allow(dead_code)]
    pub fn create(id: AccountId) -> Self {
        Jure { id }
    }
    pub fn id(&self) -> AccountId {
        self.id
    }
}
