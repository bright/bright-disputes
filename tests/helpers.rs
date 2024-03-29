use std::sync::Mutex;

use aleph_client::{
    keypair_from_string, pallets::balances::BalanceUserApi, Connection, KeyPair, SignedConnection,
    TxStatus,
};
use anyhow::Result;
use once_cell::sync::Lazy;
use rand::RngCore as _;

static AUTHORITY_MUTEX: Lazy<Mutex<KeyPair>> =
    Lazy::new(|| Mutex::new(keypair_from_string("//Alice")));

/// Generates a random test account.
pub fn random_account() -> KeyPair {
    keypair_from_string(&format!("//TestAccount/{}", rand::thread_rng().next_u64()))
}

/// Connects to the local node and transfers some funds to it. Returns a connection signed by that account.
pub async fn create_new_connection() -> Result<SignedConnection> {
    let authority = AUTHORITY_MUTEX.lock().unwrap();
    let conn = Connection::new("ws://localhost:9944").await;

    let account = random_account();
    SignedConnection::from_connection(conn.clone(), authority.clone())
        .transfer(account.account_id().clone(), alephs(100), TxStatus::InBlock)
        .await?;

    Ok(SignedConnection::from_connection(conn, account))
}

pub async fn create_new_connections(num_of_new_connections: u8) -> Result<Vec<SignedConnection>> {
    let mut connections: Vec<SignedConnection> = Vec::new();
    for _ in 0..num_of_new_connections {
        let conn = create_new_connection().await?;
        connections.push(conn);
    }
    Ok(connections)
}

pub fn alephs(n: u128) -> aleph_client::Balance {
    n * 1_000_000_000_000
}
