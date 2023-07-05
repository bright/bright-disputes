use aleph_client::{SignedConnection, SignedConnectionApi};
use anyhow::Result;
use ink_wrapper_types::util::ToAccountId;
use rand::RngCore as _;

use crate::{
    bright_disputes,
    bright_disputes::{DisputeResult, DisputeState, RoundState},
    helpers::{alephs, create_new_connection, create_new_connections},
};

async fn connect_and_deploy() -> Result<(SignedConnection, bright_disputes::Instance)> {
    let conn = create_new_connection().await?;
    let mut salt = vec![0; 32];
    rand::thread_rng().fill_bytes(&mut salt);
    let contract = bright_disputes::Instance::new(&conn, salt).await?;

    Ok((conn, contract))
}

#[tokio::test]
async fn test_dispute_success() -> Result<()> {
    // Create dispute owner, deploy on establish connection.
    let (owner_conn, contract) = connect_and_deploy().await?;

    // Create a dispute defendant
    let defendant_conn = create_new_connection().await?;
    let defendant = defendant_conn.signer().account_id().to_account_id();

    // Create a dispute
    contract
        .create_dispute(&owner_conn, "".into(), defendant, alephs(0))
        .await?;
    let dispute_id = contract.get_last_dispute_id(&owner_conn).await??;
    assert!(dispute_id == 1u32);

    // Defendant confirm dispute
    contract
        .confirm_defendant(
            &defendant_conn,
            dispute_id,
            "https://brightinventions.pl/".into(),
        )
        .await?;

    // Create juries
    let all_juries_conn: Vec<SignedConnection> = create_new_connections(4).await?;

    // Register as a jure
    for conn in &all_juries_conn {
        contract.register_as_an_active_jure(conn).await?;
    }

    // Process dispute round, assign juries and judge to dispute
    contract
        .process_dispute_round(&owner_conn, dispute_id)
        .await?;

    // Get information about dispute, judge and juries
    let dispute = contract
        .get_dispute(&owner_conn, dispute_id)
        .await??
        .expect("Unable to get dispute!");
    let juries_conn: Vec<SignedConnection> = all_juries_conn
        .iter()
        .filter(|&conn| dispute.juries.contains(&conn.account_id().to_account_id()))
        .map(|conn| conn.clone())
        .collect();
    let judge_conn = all_juries_conn
        .iter()
        .find(|&conn| {
            conn.account_id().to_account_id() == dispute.judge.expect("Jude not assigned!")
        })
        .expect("Failed to find judge!");
    assert_eq!(juries_conn.len(), 3);

    // Confirm all juries participation
    for conn in &juries_conn {
        contract
            .confirm_jure_participation_in_dispute(conn, dispute_id)
            .await?;
    }

    // Confirm judge participation
    contract
        .confirm_judge_participation_in_dispute(judge_conn, dispute_id)
        .await?;

    // Process dispute round, at this stage we check if all juries and judge confirmed their
    // participation in the dispute, move to voting phase.
    contract
        .process_dispute_round(&owner_conn, dispute_id)
        .await?;

    // Voting
    for conn in &juries_conn {
        contract.vote(conn, dispute_id, 1).await?;
    }

    // Process dispute round, check if all juries votes and move to counting the votes
    contract
        .process_dispute_round(&owner_conn, dispute_id)
        .await?;

    // Process dispute round, count the votes and move to ending phase
    contract
        .process_dispute_round(judge_conn, dispute_id)
        .await?;

    // Get dispute and check verdict
    let dispute = contract
        .get_dispute(&owner_conn, dispute_id)
        .await??
        .expect("Unable to get dispute!");
    assert!(dispute.dispute_result.unwrap() == DisputeResult::Owner());

    Ok(())
}

#[tokio::test]
async fn test_dispute_rounds() -> Result<()> {
    // Create dispute owner, deploy on establish connection.
    let (owner_conn, contract) = connect_and_deploy().await?;

    // Create a dispute defendant
    let defendant_conn = create_new_connection().await?;
    let defendant = defendant_conn.signer().account_id().to_account_id();

    // Create a dispute
    contract
        .create_dispute(&owner_conn, "".into(), defendant, alephs(0))
        .await?;
    let dispute_id = contract.get_last_dispute_id(&owner_conn).await??;
    assert!(dispute_id == 1u32);

    // Defendant confirm dispute
    contract
        .confirm_defendant(
            &defendant_conn,
            dispute_id,
            "https://brightinventions.pl/".into(),
        )
        .await?;

    // Create juries
    let all_juries_conn: Vec<SignedConnection> = create_new_connections(10).await?;

    // Register as a jure
    for conn in &all_juries_conn {
        contract.register_as_an_active_jure(conn).await?;
    }

    // Process dispute round, assign juries and judge to dispute
    contract
        .process_dispute_round(&owner_conn, dispute_id)
        .await?;

    // Get information about dispute, judge and juries
    let dispute = contract
        .get_dispute(&owner_conn, dispute_id)
        .await??
        .expect("Unable to get dispute!");
    let juries_conn: Vec<SignedConnection> = all_juries_conn
        .iter()
        .filter(|&conn| dispute.juries.contains(&conn.account_id().to_account_id()))
        .map(|conn| conn.clone())
        .collect();
    let judge_conn = all_juries_conn
        .iter()
        .find(|&conn| {
            conn.account_id().to_account_id() == dispute.judge.expect("Jude not assigned!")
        })
        .expect("Failed to find judge!");
    assert_eq!(juries_conn.len(), 3);
    assert_eq!(dispute.juries.len(), 3);
    assert_eq!(dispute.dispute_round_counter, 0);

    // Confirm all juries participation
    for conn in &juries_conn {
        contract
            .confirm_jure_participation_in_dispute(conn, dispute_id)
            .await?;
    }

    // Confirm judge participation
    contract
        .confirm_judge_participation_in_dispute(judge_conn, dispute_id)
        .await?;

    // Process dispute round, at this stage we check if all juries and judge confirmed their
    // participation in the dispute, move to voting phase.
    contract
        .process_dispute_round(&owner_conn, dispute_id)
        .await?;

    // Voting, majority of votes is not reached
    contract.vote(&juries_conn[0], dispute_id, 1).await?;
    contract.vote(&juries_conn[1], dispute_id, 1).await?;
    contract.vote(&juries_conn[2], dispute_id, 0).await?;

    // Process dispute round, check if all juries votes and move to counting the votes
    contract
        .process_dispute_round(&owner_conn, dispute_id)
        .await?;

    let dispute = contract
        .get_dispute(&owner_conn, dispute_id)
        .await??
        .expect("Unable to get dispute!");
    assert_eq!(dispute.juries.len(), 3);
    assert_eq!(dispute.dispute_round_counter, 0);
    assert_eq!(
        dispute.dispute_round.clone().unwrap().state,
        RoundState::CountingTheVotes()
    );

    // Process dispute round, count the votes and start new dispute round
    contract
        .process_dispute_round(judge_conn, dispute_id)
        .await?;

    // Get information about dispute and juries
    let dispute = contract
        .get_dispute(&owner_conn, dispute_id)
        .await??
        .expect("Unable to get dispute!");
    assert_eq!(dispute.juries.len(), 3);
    assert_eq!(dispute.dispute_round.clone().unwrap().number_of_juries, 5);
    assert_eq!(dispute.dispute_round_counter, 1);

    // Process dispute round, assign juries to dispute
    contract
        .process_dispute_round(&owner_conn, dispute_id)
        .await?;

    // Get information about dispute and juries
    let dispute = contract
        .get_dispute(&owner_conn, dispute_id)
        .await??
        .expect("Unable to get dispute!");
    let juries_round_two_conn: Vec<SignedConnection> = all_juries_conn
        .iter()
        .filter(|&conn| dispute.juries.contains(&conn.account_id().to_account_id()))
        .map(|conn| conn.clone())
        .collect();
    assert_eq!(dispute.juries.len(), 5);
    assert_eq!(juries_round_two_conn.len(), 5);

    // New juries need to confirm participation in dispute
    let diff_juries_conn: Vec<SignedConnection> = juries_round_two_conn
        .iter()
        .filter(|conn| {
            juries_conn
                .iter()
                .find(|c| c.account_id() == conn.account_id())
                .is_none()
        })
        .map(|conn| conn.clone())
        .collect();
    assert_eq!(diff_juries_conn.len(), 2);
    assert_eq!(
        dispute.dispute_round.clone().unwrap().state,
        RoundState::PickingJuriesAndJudge()
    );
    assert!(dispute.judge.is_some());
    assert_eq!(dispute.state, DisputeState::Running());

    // Confirm juries participation in dispute.
    for conn in &diff_juries_conn {
        contract
            .confirm_jure_participation_in_dispute(conn, dispute_id)
            .await?;
    }

    // Process dispute round, at this stage we check if all juries and judge confirmed their
    // participation in the dispute, move to voting phase.
    contract
        .process_dispute_round(&owner_conn, dispute_id)
        .await?;

    // Voting, majority of votes is not reached
    contract
        .vote(&juries_round_two_conn[0], dispute_id, 1)
        .await?;
    contract
        .vote(&juries_round_two_conn[1], dispute_id, 1)
        .await?;
    contract
        .vote(&juries_round_two_conn[2], dispute_id, 1)
        .await?;
    contract
        .vote(&juries_round_two_conn[3], dispute_id, 1)
        .await?;
    contract
        .vote(&juries_round_two_conn[4], dispute_id, 0)
        .await?;

    // Get information about dispute and juries
    let dispute = contract
        .get_dispute(&owner_conn, dispute_id)
        .await??
        .expect("Unable to get dispute!");
    assert_eq!(dispute.votes.len(), 5);

    // Process dispute round, check if all juries votes and move to counting the votes
    contract
        .process_dispute_round(&owner_conn, dispute_id)
        .await?;

    // Process dispute round, count the votes and move to ending phase
    contract
        .process_dispute_round(judge_conn, dispute_id)
        .await?;

    // Get dispute and check verdict
    let dispute = contract
        .get_dispute(&owner_conn, dispute_id)
        .await??
        .expect("Unable to get dispute!");
    assert!(dispute.dispute_result.unwrap() == DisputeResult::Owner());

    Ok(())
}
