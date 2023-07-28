use aleph_client::{SignedConnection, SignedConnectionApi};
use anyhow::Result;
use ink_wrapper_types::{util::ToAccountId, Connection as _, SignedConnection as _};
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

    let contract = conn
        .instantiate(bright_disputes::Instance::new().with_salt(salt))
        .await?;

    Ok((conn, contract))
}

#[tokio::test]
async fn test_dispute_success() -> Result<()> {
    // Create dispute owner, deploy on establish connection.
    let (owner_conn, contract) = connect_and_deploy().await?;

    // Define escrow
    let escrow = alephs(20);

    // Create a dispute defendant
    let defendant_conn = create_new_connection().await?;
    let defendant = defendant_conn.signer().account_id().to_account_id();

    // Create a dispute
    owner_conn
        .exec(
            contract
                .create_dispute("".into(), defendant, escrow)
                .with_value(escrow),
        )
        .await?;

    let dispute_id = owner_conn.read(contract.get_last_dispute_id()).await??;
    assert!(dispute_id == 1u32);

    // Defendant confirm dispute
    defendant_conn
        .exec(
            contract
                .confirm_defendant(dispute_id, "https://brightinventions.pl/".into())
                .with_value(escrow),
        )
        .await?;

    // Create juries
    let all_juries_conn: Vec<SignedConnection> = create_new_connections(4).await?;

    // Register as a juror
    for conn in &all_juries_conn {
        conn.exec(contract.register_as_an_active_juror()).await?;
    }

    // Process dispute round, assign juries and judge to dispute
    owner_conn
        .exec(contract.process_dispute_round(dispute_id))
        .await?;

    // Get information about dispute, judge and juries
    let dispute = owner_conn
        .read(contract.get_dispute(dispute_id))
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
        conn.exec(
            contract
                .confirm_juror_participation_in_dispute(dispute_id)
                .with_value(escrow),
        )
        .await?;
    }

    // Confirm judge participation
    judge_conn
        .exec(
            contract
                .confirm_judge_participation_in_dispute(dispute_id)
                .with_value(escrow),
        )
        .await?;

    // Process dispute round, at this stage we check if all juries and judge confirmed their
    // participation in the dispute, move to voting phase.
    owner_conn
        .exec(contract.process_dispute_round(dispute_id))
        .await?;

    // Voting
    for conn in &juries_conn {
        conn.exec(contract.vote(dispute_id, 1)).await?;
    }

    // Process dispute round, check if all juries votes and move to counting the votes
    owner_conn
        .exec(contract.process_dispute_round(dispute_id))
        .await?;

    // Process dispute round, count the votes and move to ending phase
    judge_conn
        .exec(contract.process_dispute_round(dispute_id))
        .await?;

    // Get dispute and check verdict
    let dispute = owner_conn
        .read(contract.get_dispute(dispute_id))
        .await??
        .expect("Unable to get dispute!");
    assert!(dispute.dispute_result.unwrap() == DisputeResult::Owner());

    Ok(())
}

#[tokio::test]
async fn test_dispute_rounds() -> Result<()> {
    // Create dispute owner, deploy on establish connection.
    let (owner_conn, contract) = connect_and_deploy().await?;

    // Define escrow
    let escrow = alephs(30);

    // Create a dispute defendant
    let defendant_conn = create_new_connection().await?;
    let defendant = defendant_conn.signer().account_id().to_account_id();

    // Create a dispute
    owner_conn
        .exec(
            contract
                .create_dispute("".into(), defendant, escrow)
                .with_value(escrow),
        )
        .await?;

    let dispute_id = owner_conn.read(contract.get_last_dispute_id()).await??;
    assert!(dispute_id == 1u32);

    // Defendant confirm dispute
    defendant_conn
        .exec(
            contract
                .confirm_defendant(dispute_id, "https://brightinventions.pl/".into())
                .with_value(escrow),
        )
        .await?;

    // Create juries
    let all_juries_conn: Vec<SignedConnection> = create_new_connections(10).await?;

    // Register as a juror
    for conn in &all_juries_conn {
        conn.exec(contract.register_as_an_active_juror()).await?;
    }

    // Process dispute round, assign juries and judge to dispute
    owner_conn
        .exec(contract.process_dispute_round(dispute_id))
        .await?;

    // Get information about dispute, judge and juries
    let dispute = owner_conn
        .read(contract.get_dispute(dispute_id))
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
        conn.exec(
            contract
                .confirm_juror_participation_in_dispute(dispute_id)
                .with_value(escrow),
        )
        .await?;
    }

    // Confirm judge participation
    judge_conn
        .exec(
            contract
                .confirm_judge_participation_in_dispute(dispute_id)
                .with_value(escrow),
        )
        .await?;

    // Process dispute round, at this stage we check if all juries and judge confirmed their
    // participation in the dispute, move to voting phase.
    owner_conn
        .exec(contract.process_dispute_round(dispute_id))
        .await?;

    // Voting, majority of votes is not reached
    juries_conn[0].exec(contract.vote(dispute_id, 1)).await?;
    juries_conn[1].exec(contract.vote(dispute_id, 1)).await?;
    juries_conn[2].exec(contract.vote(dispute_id, 0)).await?;

    // Process dispute round, check if all juries votes and move to counting the votes
    owner_conn
        .exec(contract.process_dispute_round(dispute_id))
        .await?;

    let dispute = owner_conn
        .read(contract.get_dispute(dispute_id))
        .await??
        .expect("Unable to get dispute!");
    assert_eq!(dispute.juries.len(), 3);
    assert_eq!(dispute.dispute_round_counter, 0);
    assert_eq!(
        dispute.dispute_round.clone().unwrap().state,
        RoundState::CountingTheVotes()
    );

    // Process dispute round, count the votes and start new dispute round
    judge_conn
        .exec(contract.process_dispute_round(dispute_id))
        .await?;

    // Get information about dispute and juries
    let dispute = owner_conn
        .read(contract.get_dispute(dispute_id))
        .await??
        .expect("Unable to get dispute!");
    assert_eq!(dispute.juries.len(), 3);
    assert_eq!(dispute.dispute_round.clone().unwrap().number_of_juries, 5);
    assert_eq!(dispute.dispute_round_counter, 1);

    // Process dispute round, assign juries to dispute
    owner_conn
        .exec(contract.process_dispute_round(dispute_id))
        .await?;

    // Get information about dispute and juries
    let dispute = owner_conn
        .read(contract.get_dispute(dispute_id))
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
        conn.exec(
            contract
                .confirm_juror_participation_in_dispute(dispute_id)
                .with_value(escrow),
        )
        .await?;
    }

    // Process dispute round, at this stage we check if all juries and judge confirmed their
    // participation in the dispute, move to voting phase.
    owner_conn
        .exec(contract.process_dispute_round(dispute_id))
        .await?;

    // Voting, majority of votes is not reached
    juries_round_two_conn[0]
        .exec(contract.vote(dispute_id, 1))
        .await?;
    juries_round_two_conn[1]
        .exec(contract.vote(dispute_id, 1))
        .await?;
    juries_round_two_conn[2]
        .exec(contract.vote(dispute_id, 1))
        .await?;
    juries_round_two_conn[3]
        .exec(contract.vote(dispute_id, 1))
        .await?;
    juries_round_two_conn[4]
        .exec(contract.vote(dispute_id, 0))
        .await?;

    // Get information about dispute and juries
    let dispute = owner_conn
        .read(contract.get_dispute(dispute_id))
        .await??
        .expect("Unable to get dispute!");
    assert_eq!(dispute.votes.len(), 5);

    // Process dispute round, check if all juries votes and move to counting the votes
    owner_conn
        .exec(contract.process_dispute_round(dispute_id))
        .await?;

    // Process dispute round, count the votes and move to ending phase
    judge_conn
        .exec(contract.process_dispute_round(dispute_id))
        .await?;

    // Get dispute and check verdict
    let dispute = owner_conn
        .read(contract.get_dispute(dispute_id))
        .await??
        .expect("Unable to get dispute!");
    assert!(dispute.dispute_result.unwrap() == DisputeResult::Owner());

    Ok(())
}
