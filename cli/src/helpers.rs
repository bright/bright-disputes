use aleph_client::{sp_core::crypto::AccountId32, AccountId};

pub fn to_ink_account_id(account_id: &AccountId) -> ink_primitives::AccountId {
    let inner: [u8; 32] = *account_id.as_ref();
    inner.into()
}

pub fn account_id_to_string(account_id: &ink_primitives::AccountId) -> String {
    let inner: [u8; 32] = *account_id.as_ref();
    let account: AccountId32 = inner.into();
    account.to_string()
}
