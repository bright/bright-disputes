use std::{fs, path::Path};

// use aleph_client::AccountId;
use anyhow::Result;
use ark_ed_on_bls12_381::{EdwardsAffine as JubJubAffine, EdwardsProjective as JubJub};
use ark_serialize::CanonicalDeserialize;
use ark_std::{vec::Vec, One, Zero};
use ink_primitives::AccountId;
use liminal_ark_relations::{
    disputes::{
        ecdh::{Ecdh, EcdhScheme},
        field_to_vote, hash_to_field, make_shared_key_hash, make_two_to_one_hash, vote_to_filed,
        VerdictNegativeRelationWithFullInput, VerdictNoneRelationWithFullInput,
        VerdictPositiveRelationWithFullInput, VerdictRelation, VoteRelationWithFullInput,
        MAX_VOTES_LEN,
    },
    environment::{CircuitField, Groth16, ProvingSystem},
    serialization::serialize,
    ConstraintSynthesizer,
};

pub mod helpers;

pub const MAJORITY_OF_VOTES: f32 = 0.70; // 70%

pub struct PublicVote {
    pub id: AccountId,
    pub pub_key: Vec<u8>,
    pub hashed_vote: [u64; 4],
}

pub fn generate_proof(
    circuit: impl ConstraintSynthesizer<CircuitField>,
    proving_key_file: &Path,
) -> Result<Vec<u8>> {
    let pk_bytes = fs::read(proving_key_file)?;
    let pk = <<Groth16 as ProvingSystem>::ProvingKey>::deserialize(&*pk_bytes)?;
    Ok(serialize(&Groth16::prove(&pk, circuit)))
}

pub fn prepare_voting_inputs(
    vote: u8,
    votes_hash: [u64; 4],
    judge_public_key: Vec<u8>,
    juror_private_key: Vec<u8>,
    vote_pk_file: &Path,
) -> Result<([u64; 4], [u64; 4], Vec<u8>)> {
    let judge_pub_key = Ecdh::<JubJub>::deserialize_public_key(judge_public_key);
    let juror_priv_key = Ecdh::<JubJub>::deserialize_private_key(juror_private_key);
    let shared_key = Ecdh::<JubJub>::make_shared_key(judge_pub_key, juror_priv_key);
    let hashed_shared_key = make_shared_key_hash(shared_key);
    let encrypted_vote = vote_to_filed(vote) + hashed_shared_key;
    let new_encrypted_all_votes = make_two_to_one_hash(encrypted_vote, hash_to_field(votes_hash));

    let circuit = VoteRelationWithFullInput::new(
        encrypted_vote.0 .0,
        votes_hash,
        new_encrypted_all_votes.0 .0,
        vote,
        hashed_shared_key.0 .0,
    );
    let proof = generate_proof(circuit, vote_pk_file)?;
    Ok((encrypted_vote.0 .0, new_encrypted_all_votes.0 .0, proof))
}

pub fn prepare_counting_inputs(
    judge_private_key: Vec<u8>,
    votes: Vec<PublicVote>,
    verdict_none_pk: &Path,
    verdict_negative_pk: &Path,
    verdict_positive_pk: &Path,
) -> Result<(u8, u8, VerdictRelation, [u64; 4], Vec<AccountId>, Vec<u8>)> {
    let judge_priv_key = Ecdh::<JubJub>::deserialize_private_key(judge_private_key.clone());

    let mut jurors_banned = Vec::<AccountId>::new();
    let mut decoded_votes = Vec::<u8>::new();
    let mut shared_keys = Vec::<[u64; 4]>::new();
    let mut all_votes_hashed = hash_to_field([1u64; 4]);
    let mut hashed_votes = CircuitField::one();
    let mut sum_votes: u8 = 0;
    for i in 0..MAX_VOTES_LEN as usize {
        if let Some(vote) = votes.get(i) {
            let juror_pub_key = Ecdh::<JubJub>::deserialize_public_key(vote.pub_key.clone());
            let shared_key = Ecdh::<JubJub>::make_shared_key(juror_pub_key, judge_priv_key);
            let hashed_shared_key = make_shared_key_hash(shared_key);
            shared_keys.push(hashed_shared_key.0 .0);

            let decode_vote =
                field_to_vote(hash_to_field(vote.hashed_vote) - hashed_shared_key.clone());
            hashed_votes = make_two_to_one_hash(hash_to_field(vote.hashed_vote), hashed_votes);
            all_votes_hashed =
                make_two_to_one_hash(hash_to_field(vote.hashed_vote), all_votes_hashed);
            sum_votes += decode_vote;
            jurors_banned.push(vote.id.clone());
            decoded_votes.push(decode_vote);
        } else {
            let shared_key = Ecdh::<JubJub>::make_shared_key(JubJubAffine::zero(), judge_priv_key);
            let hashed_shared_key = make_shared_key_hash(shared_key);

            decoded_votes.push(0u8);
            shared_keys.push(hashed_shared_key.0 .0);
            hashed_votes =
                make_two_to_one_hash(vote_to_filed(0u8) + hashed_shared_key.clone(), hashed_votes);
        }
    }

    let votes_minimum: u8 = (MAJORITY_OF_VOTES * votes.len() as f32).ceil() as u8;
    let votes_maximum: u8 = votes.len() as u8 - votes_minimum;
    let verdict = if sum_votes >= votes_minimum {
        jurors_banned = jurors_banned
            .iter()
            .enumerate()
            .filter(|&(index, _)| decoded_votes[index] == 0)
            .map(|(_, &ref value)| value.clone())
            .collect();
        VerdictRelation::Positive
    } else if sum_votes <= votes_maximum {
        jurors_banned = jurors_banned
            .iter()
            .enumerate()
            .filter(|&(index, _)| decoded_votes[index] == 1)
            .map(|(_, &ref value)| value.clone())
            .collect();
        VerdictRelation::Negative
    } else {
        VerdictRelation::None
    };

    let proof = match verdict {
        VerdictRelation::Positive => generate_proof(
            VerdictPositiveRelationWithFullInput::new(
                votes_minimum,
                VerdictRelation::Positive as u8,
                hashed_votes.0 .0,
                decoded_votes,
                shared_keys,
            ),
            verdict_positive_pk,
        )?,
        VerdictRelation::Negative => generate_proof(
            VerdictNegativeRelationWithFullInput::new(
                votes_maximum,
                VerdictRelation::Negative as u8,
                hashed_votes.0 .0,
                decoded_votes,
                shared_keys,
            ),
            verdict_negative_pk,
        )?,
        VerdictRelation::None => generate_proof(
            VerdictNoneRelationWithFullInput::new(
                votes_minimum,
                votes_maximum,
                VerdictRelation::None as u8,
                hashed_votes.0 .0,
                decoded_votes,
                shared_keys,
            ),
            verdict_none_pk,
        )?,
    };

    Ok((
        votes_maximum,
        votes_minimum,
        verdict,
        hashed_votes.0 .0,
        jurors_banned,
        proof,
    ))
}
