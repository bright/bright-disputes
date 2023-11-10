use std::fmt::{Display, Formatter};
use std::path::Path;

use aleph_client::{contract::ContractInstance, AccountId, SignedConnection};
use anyhow::{anyhow, Result};
use ark_ed_on_bls12_381::EdwardsAffine as JubJubAffine;
use ark_ed_on_bls12_381::EdwardsProjective as JubJub;
use ark_std::{vec::Vec, One, Zero};
use ink_wrapper_types::{Connection as _, SignedConnection as _};
use liminal_ark_relations::environment::CircuitField;
use liminal_ark_relations::disputes::{
    ecdh::{Ecdh, EcdhScheme},
    field_to_vote, hash_to_field, make_shared_key_hash, make_two_to_one_hash, vote_to_filed,
    VerdictNegativeRelationWithFullInput, VerdictNoneRelationWithFullInput,
    VerdictPositiveRelationWithFullInput, VerdictRelation, VoteRelationWithFullInput,
    MAX_VOTES_LEN,
};
use tracing::info;

use crate::{
    bright_disputes_ink::{Dispute, Instance, Verdict},
    generate_proof,
    helpers::{account_id_to_string, to_ink_account_id},
};

impl From<&ContractInstance> for Instance {
    fn from(contract: &ContractInstance) -> Self {
        let account_id = contract.address();
        let ink_account_id = to_ink_account_id(account_id);
        ink_account_id.into()
    }
}

impl Display for Dispute {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let judge = {
            if self.judge.is_none() {
                "".to_string()
            } else {
                account_id_to_string(&self.judge.unwrap())
            }
        };
        write!(
            f,
            "{{ \n Dispute: {} \n Owner: {} \n Defendant: {} \n Judge: {} \n Jurors: {:?} \n}}",
            self.id,
            account_id_to_string(&self.owner),
            account_id_to_string(&self.defendant),
            judge,
            self.juries
                .iter()
                .map(account_id_to_string)
                .collect::<Vec<_>>()
        )
    }
}

/// Wraps the bright disputes contract and allows to call it messages.
pub struct BrightDisputes {
    contract: ContractInstance,
}

impl BrightDisputes {
    pub fn new(address: &AccountId, metadata_path: &Path) -> Result<Self> {
        Ok(Self {
            contract: ContractInstance::new(address.clone(), metadata_path.to_str().unwrap())?,
        })
    }

    /// Calls 'get_dispute' of the contract and returns dispute.
    pub async fn get_dispute(
        &self,
        connection: &SignedConnection,
        dispute_id: u32,
    ) -> Result<Dispute> {
        let ink_contract: Instance = (&self.contract).into();

        let res = connection
            .read(ink_contract.get_dispute(dispute_id))
            .await??;

        match res {
            Ok(dispute) => Ok(dispute),
            _ => Err(anyhow!("Unable to get dispute!")),
        }
    }

    /// Calls 'juror_public_key' of the contract and returns dispute.
    pub async fn get_juror_public_key(
        &self,
        connection: &SignedConnection,
        dispute_id: u32,
        juror_id: ink_primitives::AccountId,
    ) -> Result<Vec<u8>> {
        let ink_contract: Instance = (&self.contract).into();

        let res = connection
            .read(ink_contract.juror_public_key(dispute_id, juror_id))
            .await??;

        match res {
            Ok(key) => Ok(key),
            _ => Err(anyhow!("Unable to get public key for the juror!")),
        }
    }

    /// Calls 'create_dispute' of the contract. If success, return dispute id of newly created dispute.
    pub async fn create_dispute(
        &self,
        connection: &SignedConnection,
        owner_link: String,
        defendant_id: ink_primitives::AccountId,
        escrow: u128,
    ) -> Result<u32> {
        let ink_contract: Instance = (&self.contract).into();

        connection
            .exec(
                ink_contract
                    .create_dispute(owner_link, defendant_id, escrow)
                    .with_value(escrow),
            )
            .await?;

        let res = connection
            .read(ink_contract.get_last_dispute_id())
            .await??;
        Ok(res)
    }

    /// Calls 'confirm_defendant' of the contract.
    pub async fn confirm_defendant(
        &self,
        connection: &SignedConnection,
        dispute_id: u32,
        defendant_link: String,
        escrow: u128,
    ) -> Result<()> {
        let ink_contract: Instance = (&self.contract).into();

        connection
            .exec(
                ink_contract
                    .confirm_defendant(dispute_id, defendant_link)
                    .with_value(escrow),
            )
            .await?;

        Ok(())
    }

    /// Calls 'update_owner_description' of the contract.
    pub async fn update_owner_description(
        &self,
        connection: &SignedConnection,
        dispute_id: u32,
        owner_link: String,
    ) -> Result<()> {
        let ink_contract: Instance = (&self.contract).into();

        connection
            .exec(ink_contract.update_owner_description(dispute_id, owner_link))
            .await?;

        Ok(())
    }

    /// Calls 'update_defendant_description' of the contract.
    pub async fn update_defendant_description(
        &self,
        connection: &SignedConnection,
        dispute_id: u32,
        defendant_link: String,
    ) -> Result<()> {
        let ink_contract: Instance = (&self.contract).into();

        connection
            .exec(ink_contract.update_defendant_description(dispute_id, defendant_link))
            .await?;

        Ok(())
    }

    /// Calls 'vote' of the contract.
    pub async fn vote(
        &self,
        connection: &SignedConnection,
        dispute_id: u32,
        private_key: Vec<u8>,
        vote: u8,
        vote_pk_file: &Path,
    ) -> Result<()> {
        let dispute = self.get_dispute(connection, dispute_id).await?;
        let judge_id = dispute
            .judge
            .ok_or(anyhow!("Failed to vote, Judge is not assigned!"))?;
        let judge_pub_key = self
            .get_juror_public_key(connection, dispute_id, judge_id)
            .await?;

        let judge_pub_key = Ecdh::<JubJub>::deserialize_public_key(judge_pub_key);
        let juror_priv_key = Ecdh::<JubJub>::deserialize_private_key(private_key);
        let shared_key = Ecdh::<JubJub>::make_shared_key(judge_pub_key, juror_priv_key);
        let hashed_shared_key = make_shared_key_hash(shared_key);
        let encrypted_vote = vote_to_filed(vote) + hashed_shared_key;
        let new_encrypted_all_votes =
            make_two_to_one_hash(encrypted_vote, hash_to_field(dispute.votes_hash));

        let circuit = VoteRelationWithFullInput::new(
            encrypted_vote.0 .0,
            dispute.votes_hash,
            new_encrypted_all_votes.0 .0,
            vote,
            hashed_shared_key.0 .0,
        );
        let proof = generate_proof(circuit, vote_pk_file)?;

        info!(target: "bright_disputes_cli", "Proof generated");

        let ink_contract: Instance = (&self.contract).into();
        connection
            .exec(ink_contract.vote(
                dispute_id,
                encrypted_vote.0 .0,
                new_encrypted_all_votes.0 .0,
                proof,
            ))
            .await?;

        Ok(())
    }

    /// Calls 'register_as_an_active_juror' of the contract.
    pub async fn register_as_an_active_juror(&self, connection: &SignedConnection) -> Result<()> {
        let ink_contract: Instance = (&self.contract).into();

        connection
            .exec(ink_contract.register_as_an_active_juror())
            .await?;

        Ok(())
    }

    /// Calls 'unregister_as_an_active_juror' of the contract.
    pub async fn unregister_as_an_active_juror(&self, connection: &SignedConnection) -> Result<()> {
        let ink_contract: Instance = (&self.contract).into();

        connection
            .exec(ink_contract.unregister_as_an_active_juror())
            .await?;

        Ok(())
    }

    /// Calls 'confirm_juror_participation_in_dispute' of the contract.
    pub async fn confirm_juror_participation_in_dispute(
        &self,
        connection: &SignedConnection,
        dispute_id: u32,
        escrow: u128,
    ) -> Result<()> {
        let ink_contract: Instance = (&self.contract).into();

        let rng = &mut rand::thread_rng();
        let (pub_alice, priv_alice) = Ecdh::<JubJub>::generate_keys(rng);

        let serialized_pub = Ecdh::<JubJub>::serialize_public_key(pub_alice);
        let serialized_priv = Ecdh::<JubJub>::serialize_private_key(priv_alice);

        info!(target: "bright_disputes_cli", "Public key: {:?} \n Private key: {:?}!", serialized_pub, serialized_priv);

        connection
            .exec(
                ink_contract
                    .confirm_juror_participation_in_dispute(dispute_id, serialized_pub)
                    .with_value(escrow),
            )
            .await?;

        Ok(())
    }

    /// Calls 'confirm_judge_participation_in_dispute' of the contract.
    pub async fn confirm_judge_participation_in_dispute(
        &self,
        connection: &SignedConnection,
        dispute_id: u32,
        escrow: u128,
    ) -> Result<()> {
        let ink_contract: Instance = (&self.contract).into();

        let rng = &mut rand::thread_rng();
        let (pub_alice, priv_alice) = Ecdh::<JubJub>::generate_keys(rng);

        let serialized_pub = Ecdh::<JubJub>::serialize_public_key(pub_alice);
        let serialized_priv = Ecdh::<JubJub>::serialize_private_key(priv_alice);

        info!(target: "bright_disputes_cli", "Judge Public key: {:?} \n Private key: {:?}!", serialized_pub, serialized_priv);
        connection
            .exec(
                ink_contract
                    .confirm_judge_participation_in_dispute(dispute_id, serialized_pub)
                    .with_value(escrow),
            )
            .await?;

        Ok(())
    }

    /// Calls 'count_the_votes' of the contract.
    pub async fn count_the_votes(
        &self,
        connection: &SignedConnection,
        dispute_id: u32,
        private_key: Vec<u8>,
        verdict_none_pk: &Path,
        verdict_negative_pk: &Path,
        verdict_positive_pk: &Path,
    ) -> Result<()> {
        let dispute = self.get_dispute(connection, dispute_id).await?;
        let judge_priv_key = Ecdh::<JubJub>::deserialize_private_key(private_key.clone());

        let mut jurors_banned = Vec::<ink_primitives::AccountId>::new();
        let mut decoded_votes = Vec::<u8>::new();
        let mut shared_keys = Vec::<[u64; 4]>::new();
        let mut all_votes_hashed = hash_to_field([1u64; 4]);
        let mut hashed_votes = CircuitField::one();
        let mut sum_votes: u8 = 0;
        for i in 0..MAX_VOTES_LEN as usize {
            if let Some(vote) = dispute.votes.get(i) {
                let juror_pub_key = self
                    .get_juror_public_key(connection, dispute_id, vote.juror)
                    .await?;
                let juror_pub_key = Ecdh::<JubJub>::deserialize_public_key(juror_pub_key);
                let shared_key = Ecdh::<JubJub>::make_shared_key(juror_pub_key, judge_priv_key);
                let hashed_shared_key = make_shared_key_hash(shared_key);
                shared_keys.push(hashed_shared_key.0 .0);

                let decode_vote =
                    field_to_vote(hash_to_field(vote.vote) - hashed_shared_key.clone());
                hashed_votes = make_two_to_one_hash(hash_to_field(vote.vote), hashed_votes);
                all_votes_hashed = make_two_to_one_hash(hash_to_field(vote.vote), all_votes_hashed);
                if decode_vote > 0 {
                    jurors_banned.push(vote.juror);
                }
                sum_votes += decode_vote;
                decoded_votes.push(decode_vote);
            } else {
                let shared_key =
                    Ecdh::<JubJub>::make_shared_key(JubJubAffine::zero(), judge_priv_key);
                let hashed_shared_key = make_shared_key_hash(shared_key);

                decoded_votes.push(0u8);
                shared_keys.push(hashed_shared_key.0 .0);
                hashed_votes = make_two_to_one_hash(
                    vote_to_filed(0u8) + hashed_shared_key.clone(),
                    hashed_votes,
                );
            }
        }

        let votes_minimum: u8 = (0.75 * dispute.juries.len() as f32).ceil() as u8;
        let votes_maximum: u8 = dispute.juries.len() as u8 - votes_minimum;
        let verdict = if sum_votes >= votes_maximum {
            Verdict::Positive()
        } else if sum_votes <= votes_minimum {
            Verdict::Negative()
        } else {
            Verdict::None()
        };

        let proof = match verdict {
            Verdict::Positive() => generate_proof(
                VerdictPositiveRelationWithFullInput::new(
                    votes_minimum,
                    VerdictRelation::Positive as u8,
                    hashed_votes.0 .0,
                    decoded_votes,
                    shared_keys,
                ),
                verdict_positive_pk,
            ),
            Verdict::Negative() => generate_proof(
                VerdictNegativeRelationWithFullInput::new(
                    votes_maximum,
                    VerdictRelation::Negative as u8,
                    hashed_votes.0 .0,
                    decoded_votes,
                    shared_keys,
                ),
                verdict_negative_pk,
            ),
            Verdict::None() => generate_proof(
                VerdictNoneRelationWithFullInput::new(
                    votes_maximum,
                    votes_minimum,
                    VerdictRelation::None as u8,
                    hashed_votes.0 .0,
                    decoded_votes,
                    shared_keys,
                ),
                verdict_none_pk,
            ),
        };

        info!(target: "bright_disputes_cli", "Proof generated");

        let ink_contract: Instance = (&self.contract).into();
        connection
            .exec(ink_contract.issue_the_verdict(
                dispute_id,
                votes_maximum,
                votes_minimum,
                verdict,
                hashed_votes.0 .0,
                jurors_banned,
                proof.unwrap(),
            ))
            .await?;

        Ok(())
    }

    /// Calls 'process_dispute_round' of the contract.
    pub async fn process_dispute_round(
        &self,
        connection: &SignedConnection,
        dispute_id: u32,
    ) -> Result<()> {
        let ink_contract: Instance = (&self.contract).into();

        connection
            .exec(ink_contract.process_dispute_round(dispute_id))
            .await?;

        Ok(())
    }

    /// Calls 'distribute_deposit' of the contract.
    pub async fn distribute_deposit(
        &self,
        connection: &SignedConnection,
        dispute_id: u32,
    ) -> Result<()> {
        let ink_contract: Instance = (&self.contract).into();

        connection
            .exec(ink_contract.distribute_deposit(dispute_id))
            .await?;

        Ok(())
    }
}
