use std::fmt::{Display, Formatter};
use std::path::Path;

use aleph_client::{contract::ContractInstance, AccountId, SignedConnection};
use anyhow::{anyhow, Result};
use ark_ed_on_bls12_381::EdwardsProjective as JubJub;
use ark_std::vec::Vec;
use bright_disputes_lib::{
    helpers::{account_id_to_string, to_ink_account_id},
    prepare_counting_inputs, prepare_voting_inputs, PublicVote,
};
use ink_wrapper_types::{Connection as _, SignedConnection as _};
use liminal_ark_relations::disputes::{
    ecdh::{Ecdh, EcdhScheme},
    VerdictRelation,
};
use tracing::info;

use crate::bright_disputes_ink::{Dispute, Instance, Verdict};

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

        let (encrypted_vote, new_encrypted_all_votes, proof) = prepare_voting_inputs(
            vote,
            dispute.votes_hash,
            judge_pub_key,
            private_key,
            vote_pk_file,
        )?;

        info!(target: "bright_disputes_cli", "Proof generated");

        let ink_contract: Instance = (&self.contract).into();
        connection
            .exec(ink_contract.vote(dispute_id, encrypted_vote, new_encrypted_all_votes, proof))
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
        judge_private_key: Vec<u8>,
        verdict_none_pk: &Path,
        verdict_negative_pk: &Path,
        verdict_positive_pk: &Path,
    ) -> Result<()> {
        let dispute = self.get_dispute(connection, dispute_id).await?;

        let mut jurors_public_key = Vec::<Vec<u8>>::new();
        for vote in &dispute.votes {
            let key = self
                .get_juror_public_key(connection, dispute_id, vote.juror)
                .await?;
            jurors_public_key.push(key);
        }

        let votes: Vec<PublicVote> = dispute
            .votes
            .iter()
            .zip(jurors_public_key.iter())
            .map(|(&ref vote, &ref key)| PublicVote {
                id: vote.juror,
                pub_key: key.clone(),
                hashed_vote: vote.vote,
            })
            .collect();

        let (votes_maximum, votes_minimum, verdict, hashed_votes, jurors_banned, proof) =
            prepare_counting_inputs(
                judge_private_key,
                votes,
                verdict_none_pk,
                verdict_negative_pk,
                verdict_positive_pk,
            )?;

        let ink_verdict = match verdict {
            VerdictRelation::Positive => Verdict::Positive(),
            VerdictRelation::Negative => Verdict::Negative(),
            VerdictRelation::None => Verdict::None(),
        };

        info!(target: "bright_disputes_cli", "Proof generated");

        let ink_contract: Instance = (&self.contract).into();
        connection
            .exec(ink_contract.issue_the_verdict(
                dispute_id,
                votes_maximum,
                votes_minimum,
                ink_verdict,
                hashed_votes,
                jurors_banned,
                proof,
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
}
