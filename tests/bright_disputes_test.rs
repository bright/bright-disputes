use aleph_client::{SignedConnection, SignedConnectionApi};
use anyhow::Result;
use core::iter::zip;
use ink_wrapper_types::{util::ToAccountId, Connection as _, SignedConnection as _};
use rand::RngCore as _;

use bright_disputes_lib::{prepare_counting_inputs, prepare_voting_inputs, PublicVote};
use liminal_ark_relations::disputes::VerdictRelation;
use std::path::PathBuf;

use crate::{
    bright_disputes::{DisputeResult, DisputeState, Instance, RoundState, Verdict},
    helpers::{alephs, create_new_connection, create_new_connections},
};

async fn connect_and_deploy() -> Result<(SignedConnection, Instance)> {
    let conn = create_new_connection().await?;
    let mut salt = vec![0; 32];
    rand::thread_rng().fill_bytes(&mut salt);

    let contract = conn.instantiate(Instance::new().with_salt(salt)).await?;

    Ok((conn, contract))
}

#[derive(Clone)]
struct KeyPair {
    pub public: Vec<u8>,
    pub private: Vec<u8>,
}

struct Setup {
    vote_pk: PathBuf,
    verdict_none_pk: PathBuf,
    verdict_negative_pk: PathBuf,
    verdict_positive_pk: PathBuf,
    judge: KeyPair,
    jurors: Vec<KeyPair>,
    jurors_extended: Vec<KeyPair>,
    jurors_large: Vec<KeyPair>,
}

impl Setup {
    fn new() -> Self {
        let juries_base = vec![
            KeyPair {
                public: vec![
                    143, 96, 146, 215, 67, 186, 237, 47, 231, 60, 4, 227, 180, 180, 227, 175, 139,
                    11, 9, 212, 45, 153, 174, 82, 61, 94, 185, 142, 229, 93, 248, 141,
                ],
                private: vec![
                    179, 168, 214, 171, 19, 120, 215, 166, 1, 175, 173, 235, 85, 161, 223, 244,
                    253, 121, 185, 141, 92, 32, 171, 52, 154, 20, 21, 152, 97, 250, 190, 6,
                ],
            },
            KeyPair {
                public: vec![
                    93, 66, 190, 16, 93, 13, 181, 112, 42, 68, 88, 90, 88, 65, 241, 30, 80, 202,
                    221, 3, 137, 104, 89, 40, 93, 2, 69, 100, 36, 104, 158, 72,
                ],
                private: vec![
                    250, 71, 107, 70, 4, 169, 48, 81, 216, 97, 222, 161, 213, 137, 52, 53, 250, 3,
                    165, 188, 184, 58, 181, 151, 160, 0, 153, 178, 252, 164, 62, 7,
                ],
            },
            KeyPair {
                public: vec![
                    199, 48, 32, 250, 139, 107, 224, 127, 96, 217, 223, 140, 130, 3, 111, 69, 146,
                    249, 47, 219, 36, 50, 38, 216, 154, 163, 197, 232, 65, 72, 57, 115,
                ],
                private: vec![
                    214, 199, 173, 149, 198, 9, 55, 208, 20, 165, 65, 187, 65, 103, 253, 211, 174,
                    91, 92, 193, 21, 244, 157, 43, 215, 163, 41, 161, 15, 65, 106, 4,
                ],
            },
        ];

        let juries_extended = vec![
            KeyPair {
                public: vec![
                    80, 178, 234, 242, 171, 209, 22, 130, 199, 254, 117, 68, 133, 250, 75, 85, 234,
                    102, 19, 27, 233, 156, 129, 188, 151, 189, 75, 102, 11, 7, 166, 10,
                ],
                private: vec![
                    129, 115, 160, 93, 168, 200, 122, 82, 174, 154, 170, 159, 113, 154, 16, 88,
                    236, 9, 196, 177, 244, 70, 44, 155, 75, 214, 69, 78, 100, 219, 245, 3,
                ],
            },
            KeyPair {
                public: vec![
                    16, 95, 167, 2, 64, 164, 179, 52, 113, 155, 196, 127, 52, 53, 166, 5, 162, 184,
                    107, 58, 249, 150, 181, 89, 156, 141, 228, 36, 170, 156, 2, 159,
                ],
                private: vec![
                    109, 73, 196, 180, 234, 25, 39, 144, 20, 144, 101, 198, 157, 48, 146, 204, 169,
                    91, 59, 90, 133, 23, 138, 171, 193, 194, 158, 189, 148, 150, 204, 1,
                ],
            },
        ];

        Self {
            vote_pk: "../scripts/docker/keys/vote.groth16.pk.bytes".into(),
            verdict_none_pk: "../scripts/docker/keys/verdict_none.groth16.pk.bytes".into(),
            verdict_negative_pk: "../scripts/docker/keys/verdict_negative.groth16.pk.bytes".into(),
            verdict_positive_pk: "../scripts/docker/keys/verdict_positive.groth16.pk.bytes".into(),
            judge: KeyPair {
                public: vec![
                    209, 186, 95, 203, 236, 84, 246, 136, 3, 232, 135, 235, 5, 218, 13, 168, 128,
                    89, 67, 143, 5, 125, 187, 223, 178, 40, 113, 238, 18, 97, 242, 81,
                ],
                private: vec![
                    25, 164, 133, 151, 251, 54, 205, 192, 212, 173, 218, 155, 210, 238, 98, 4, 36,
                    68, 162, 114, 94, 30, 134, 181, 187, 167, 219, 131, 227, 25, 202, 6,
                ],
            },
            jurors: juries_base.clone(),
            jurors_extended: juries_extended.clone(),
            jurors_large: juries_base
                .clone()
                .into_iter()
                .chain(juries_extended.clone())
                .collect(),
        }
    }
}

#[tokio::test]
async fn test_dispute_verdict_positive() -> Result<()> {
    let setup = Setup::new();

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
    for (juror, conn) in zip(&setup.jurors, &juries_conn) {
        conn.exec(
            contract
                .confirm_juror_participation_in_dispute(dispute_id, juror.public.clone())
                .with_value(escrow),
        )
        .await?;
    }

    // Confirm judge participation
    judge_conn
        .exec(
            contract
                .confirm_judge_participation_in_dispute(dispute_id, setup.judge.public.clone())
                .with_value(escrow),
        )
        .await?;

    // Process dispute round, at this stage we check if all juries and judge confirmed their
    // participation in the dispute, move to voting phase.
    owner_conn
        .exec(contract.process_dispute_round(dispute_id))
        .await?;

    // Voting
    let mut votes_hash = [1u64; 4];
    for (juror, conn) in zip(&setup.jurors, &juries_conn) {
        let vote: u8 = 1;
        let (encrypted_vote, new_encrypted_all_votes, proof) = prepare_voting_inputs(
            vote,
            votes_hash,
            setup.judge.public.clone(),
            juror.private.clone(),
            &setup.vote_pk,
        )?;

        votes_hash = new_encrypted_all_votes.clone();

        conn.exec(contract.vote(dispute_id, encrypted_vote, new_encrypted_all_votes, proof))
            .await?;
    }

    // Process dispute round, check if all juries votes and move to counting the votes
    owner_conn
        .exec(contract.process_dispute_round(dispute_id))
        .await?;

    // Get information about dispute, judge and juries
    let dispute = owner_conn
        .read(contract.get_dispute(dispute_id))
        .await??
        .expect("Unable to get dispute!");

    let votes: Vec<PublicVote> = dispute
        .votes
        .iter()
        .enumerate()
        .map(|(index, &ref vote)| {
            let key = setup.jurors.get(index).unwrap();
            PublicVote {
                id: vote.juror.clone(),
                pub_key: key.public.clone(),
                hashed_vote: vote.vote,
            }
        })
        .collect();

    let (votes_maximum, votes_minimum, verdict, hashed_votes, jurors_banned, proof) =
        prepare_counting_inputs(
            setup.judge.private.clone(),
            votes,
            &setup.verdict_none_pk,
            &setup.verdict_negative_pk,
            &setup.verdict_positive_pk,
        )?;

    let ink_verdict = match verdict {
        VerdictRelation::Positive => Verdict::Positive(),
        VerdictRelation::Negative => Verdict::Negative(),
        VerdictRelation::None => Verdict::None(),
    };

    judge_conn
        .exec(contract.issue_the_verdict(
            dispute_id,
            votes_maximum,
            votes_minimum,
            ink_verdict,
            hashed_votes,
            jurors_banned,
            proof,
        ))
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
async fn test_dispute_verdict_negative() -> Result<()> {
    let setup = Setup::new();

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
    for (juror, conn) in zip(&setup.jurors, &juries_conn) {
        conn.exec(
            contract
                .confirm_juror_participation_in_dispute(dispute_id, juror.public.clone())
                .with_value(escrow),
        )
        .await?;
    }

    // Confirm judge participation
    judge_conn
        .exec(
            contract
                .confirm_judge_participation_in_dispute(dispute_id, setup.judge.public.clone())
                .with_value(escrow),
        )
        .await?;

    // Process dispute round, at this stage we check if all juries and judge confirmed their
    // participation in the dispute, move to voting phase.
    owner_conn
        .exec(contract.process_dispute_round(dispute_id))
        .await?;

    // Voting
    let mut votes_hash = [1u64; 4];
    for (juror, conn) in zip(&setup.jurors, &juries_conn) {
        let vote: u8 = 0;
        let (encrypted_vote, new_encrypted_all_votes, proof) = prepare_voting_inputs(
            vote,
            votes_hash,
            setup.judge.public.clone(),
            juror.private.clone(),
            &setup.vote_pk,
        )?;

        votes_hash = new_encrypted_all_votes.clone();

        conn.exec(contract.vote(dispute_id, encrypted_vote, new_encrypted_all_votes, proof))
            .await?;
    }

    // Process dispute round, check if all juries votes and move to counting the votes
    owner_conn
        .exec(contract.process_dispute_round(dispute_id))
        .await?;

    // Get information about dispute, judge and juries
    let dispute = owner_conn
        .read(contract.get_dispute(dispute_id))
        .await??
        .expect("Unable to get dispute!");

    let votes: Vec<PublicVote> = dispute
        .votes
        .iter()
        .enumerate()
        .map(|(index, &ref vote)| {
            let key = setup.jurors.get(index).unwrap();
            PublicVote {
                id: vote.juror.clone(),
                pub_key: key.public.clone(),
                hashed_vote: vote.vote,
            }
        })
        .collect();

    let (votes_maximum, votes_minimum, verdict, hashed_votes, jurors_banned, proof) =
        prepare_counting_inputs(
            setup.judge.private.clone(),
            votes,
            &setup.verdict_none_pk,
            &setup.verdict_negative_pk,
            &setup.verdict_positive_pk,
        )?;

    let ink_verdict = match verdict {
        VerdictRelation::Positive => Verdict::Positive(),
        VerdictRelation::Negative => Verdict::Negative(),
        VerdictRelation::None => Verdict::None(),
    };

    judge_conn
        .exec(contract.issue_the_verdict(
            dispute_id,
            votes_maximum,
            votes_minimum,
            ink_verdict,
            hashed_votes,
            jurors_banned,
            proof,
        ))
        .await?;

    // Get dispute and check verdict
    let dispute = owner_conn
        .read(contract.get_dispute(dispute_id))
        .await??
        .expect("Unable to get dispute!");
    assert!(dispute.dispute_result.unwrap() == DisputeResult::Defendant());

    Ok(())
}

#[tokio::test]
async fn test_dispute_verdict_none() -> Result<()> {
    let setup = Setup::new();

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
    for (juror, conn) in zip(&setup.jurors, &juries_conn) {
        conn.exec(
            contract
                .confirm_juror_participation_in_dispute(dispute_id, juror.public.clone())
                .with_value(escrow),
        )
        .await?;
    }

    // Confirm judge participation
    judge_conn
        .exec(
            contract
                .confirm_judge_participation_in_dispute(dispute_id, setup.judge.public.clone())
                .with_value(escrow),
        )
        .await?;

    // Process dispute round, at this stage we check if all juries and judge confirmed their
    // participation in the dispute, move to voting phase.
    owner_conn
        .exec(contract.process_dispute_round(dispute_id))
        .await?;

    // Voting
    let mut votes_hash = [1u64; 4];
    let mut vote: u8 = 1;
    for (juror, conn) in zip(&setup.jurors, &juries_conn) {
        let (encrypted_vote, new_encrypted_all_votes, proof) = prepare_voting_inputs(
            vote,
            votes_hash,
            setup.judge.public.clone(),
            juror.private.clone(),
            &setup.vote_pk,
        )?;

        votes_hash = new_encrypted_all_votes.clone();

        conn.exec(contract.vote(dispute_id, encrypted_vote, new_encrypted_all_votes, proof))
            .await?;
        vote = (vote + 1) % 2;
    }

    // Process dispute round, check if all juries votes and move to counting the votes
    owner_conn
        .exec(contract.process_dispute_round(dispute_id))
        .await?;

    // Get information about dispute, judge and juries
    let dispute = owner_conn
        .read(contract.get_dispute(dispute_id))
        .await??
        .expect("Unable to get dispute!");

    let votes: Vec<PublicVote> = dispute
        .votes
        .iter()
        .enumerate()
        .map(|(index, &ref vote)| {
            let key = setup.jurors.get(index).unwrap();
            PublicVote {
                id: vote.juror.clone(),
                pub_key: key.public.clone(),
                hashed_vote: vote.vote,
            }
        })
        .collect();

    let (votes_maximum, votes_minimum, verdict, hashed_votes, jurors_banned, proof) =
        prepare_counting_inputs(
            setup.judge.private.clone(),
            votes,
            &setup.verdict_none_pk,
            &setup.verdict_negative_pk,
            &setup.verdict_positive_pk,
        )?;

    let ink_verdict = match verdict {
        VerdictRelation::Positive => Verdict::Positive(),
        VerdictRelation::Negative => Verdict::Negative(),
        VerdictRelation::None => Verdict::None(),
    };

    judge_conn
        .exec(contract.issue_the_verdict(
            dispute_id,
            votes_maximum,
            votes_minimum,
            ink_verdict,
            hashed_votes,
            jurors_banned,
            proof,
        ))
        .await?;

    // Get dispute and check verdict
    let dispute = owner_conn
        .read(contract.get_dispute(dispute_id))
        .await??
        .expect("Unable to get dispute!");
    assert!(dispute.dispute_result.is_none());

    Ok(())
}

#[tokio::test]
async fn test_dispute_rounds() -> Result<()> {
    let setup = Setup::new();

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
    let mut juries_conn: Vec<SignedConnection> = all_juries_conn
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
    assert_eq!(
        dispute.dispute_round.clone().unwrap().state,
        RoundState::PickingJuriesAndJudge()
    );

    // Confirm all juries participation
    for (juror, conn) in zip(&setup.jurors, &juries_conn) {
        conn.exec(
            contract
                .confirm_juror_participation_in_dispute(dispute_id, juror.public.clone())
                .with_value(escrow),
        )
        .await?;
    }

    // Confirm judge participation
    judge_conn
        .exec(
            contract
                .confirm_judge_participation_in_dispute(dispute_id, setup.judge.public.clone())
                .with_value(escrow),
        )
        .await?;

    // Process dispute round, at this stage we check if all juries and judge confirmed their
    // participation in the dispute, move to voting phase.
    owner_conn
        .exec(contract.process_dispute_round(dispute_id))
        .await?;

    // Voting, majority of votes is not reached
    let mut votes_hash = [1u64; 4];
    let mut vote: u8 = 1;
    for (juror, conn) in zip(&setup.jurors, &juries_conn) {
        let (encrypted_vote, new_encrypted_all_votes, proof) = prepare_voting_inputs(
            vote,
            votes_hash,
            setup.judge.public.clone(),
            juror.private.clone(),
            &setup.vote_pk,
        )?;

        votes_hash = new_encrypted_all_votes.clone();

        conn.exec(contract.vote(dispute_id, encrypted_vote, new_encrypted_all_votes, proof))
            .await?;
        vote = (vote + 1) % 2;
    }

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

    // Counting the votes and issue the verdict
    let votes: Vec<PublicVote> = dispute
        .votes
        .iter()
        .enumerate()
        .map(|(index, &ref vote)| {
            let key = setup.jurors.get(index).unwrap();
            PublicVote {
                id: vote.juror.clone(),
                pub_key: key.public.clone(),
                hashed_vote: vote.vote,
            }
        })
        .collect();

    let (votes_maximum, votes_minimum, verdict, hashed_votes, jurors_banned, proof) =
        prepare_counting_inputs(
            setup.judge.private.clone(),
            votes,
            &setup.verdict_none_pk,
            &setup.verdict_negative_pk,
            &setup.verdict_positive_pk,
        )?;

    let ink_verdict = match verdict {
        VerdictRelation::Positive => Verdict::Positive(),
        VerdictRelation::Negative => Verdict::Negative(),
        VerdictRelation::None => Verdict::None(),
    };

    judge_conn
        .exec(contract.issue_the_verdict(
            dispute_id,
            votes_maximum,
            votes_minimum,
            ink_verdict,
            hashed_votes,
            jurors_banned,
            proof,
        ))
        .await?;

    let dispute = owner_conn
        .read(contract.get_dispute(dispute_id))
        .await??
        .expect("Unable to get dispute!");
    assert_eq!(dispute.juries.len(), 3);
    assert_eq!(dispute.dispute_round.clone().unwrap().number_of_juries, 5);
    assert_eq!(dispute.dispute_round_counter, 1);
    assert_eq!(dispute.state, DisputeState::Running());
    assert!(dispute.dispute_result.is_none());
    assert!(dispute.judge.is_some());
    assert_eq!(
        dispute.dispute_round.clone().unwrap().state,
        RoundState::AssignJuriesAndJudge()
    );

    // Process dispute round, assign juries to dispute
    owner_conn
        .exec(contract.process_dispute_round(dispute_id))
        .await?;

    let dispute = owner_conn
        .read(contract.get_dispute(dispute_id))
        .await??
        .expect("Unable to get dispute!");
    assert_eq!(dispute.juries.len(), 5);
    assert_eq!(dispute.dispute_round.clone().unwrap().number_of_juries, 5);
    assert_eq!(
        dispute.dispute_round.clone().unwrap().state,
        RoundState::PickingJuriesAndJudge()
    );

    let new_juries_conn: Vec<SignedConnection> = all_juries_conn
        .iter()
        .filter(|&conn| {
            conn.account_id().to_account_id() == *dispute.juries.get(3).expect("Jude not assigned!")
                || conn.account_id().to_account_id()
                    == *dispute.juries.get(4).expect("Jude not assigned!")
        })
        .map(|conn| conn.clone())
        .collect();

    // Confirm juries participation in dispute.
    for (juror, conn) in zip(&setup.jurors_extended, &new_juries_conn) {
        conn.exec(
            contract
                .confirm_juror_participation_in_dispute(dispute_id, juror.public.clone())
                .with_value(escrow),
        )
        .await?;
    }
    juries_conn.extend(new_juries_conn);

    // Process dispute round, at this stage we check if all juries and judge confirmed their
    // participation in the dispute, move to voting phase.
    owner_conn
        .exec(contract.process_dispute_round(dispute_id))
        .await?;

    // Voting
    let mut votes_hash = [1u64; 4];
    for (juror, conn) in zip(&setup.jurors_large, &juries_conn) {
        let vote: u8 = 1;
        let (encrypted_vote, new_encrypted_all_votes, proof) = prepare_voting_inputs(
            vote,
            votes_hash,
            setup.judge.public.clone(),
            juror.private.clone(),
            &setup.vote_pk,
        )?;

        votes_hash = new_encrypted_all_votes.clone();

        conn.exec(contract.vote(dispute_id, encrypted_vote, new_encrypted_all_votes, proof))
            .await?;
    }

    // Process dispute round, check if all juries votes and move to counting the votes
    owner_conn
        .exec(contract.process_dispute_round(dispute_id))
        .await?;

    let dispute = owner_conn
        .read(contract.get_dispute(dispute_id))
        .await??
        .expect("Unable to get dispute!");
    assert_eq!(
        dispute.dispute_round.clone().unwrap().state,
        RoundState::CountingTheVotes()
    );

    // Counting the votes and issue the verdict
    let votes: Vec<PublicVote> = dispute
        .votes
        .iter()
        .enumerate()
        .map(|(index, &ref vote)| {
            let key = setup.jurors_large.get(index).unwrap();
            PublicVote {
                id: vote.juror.clone(),
                pub_key: key.public.clone(),
                hashed_vote: vote.vote,
            }
        })
        .collect();

    let (votes_maximum, votes_minimum, verdict, hashed_votes, jurors_banned, proof) =
        prepare_counting_inputs(
            setup.judge.private.clone(),
            votes,
            &setup.verdict_none_pk,
            &setup.verdict_negative_pk,
            &setup.verdict_positive_pk,
        )?;

    let ink_verdict = match verdict {
        VerdictRelation::Positive => Verdict::Positive(),
        VerdictRelation::Negative => Verdict::Negative(),
        VerdictRelation::None => Verdict::None(),
    };

    judge_conn
        .exec(contract.issue_the_verdict(
            dispute_id,
            votes_maximum,
            votes_minimum,
            ink_verdict,
            hashed_votes,
            jurors_banned,
            proof,
        ))
        .await?;

    // Get dispute and check verdict
    let dispute = owner_conn
        .read(contract.get_dispute(dispute_id))
        .await??
        .expect("Unable to get dispute!");
    assert!(dispute.dispute_result.unwrap() == DisputeResult::Owner());

    Ok(())
}
