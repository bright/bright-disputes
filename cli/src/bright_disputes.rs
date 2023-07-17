use std::fmt::{Display, Formatter};
use std::path::Path;

use aleph_client::{contract::ContractInstance, AccountId, SignedConnection};
use anyhow::{anyhow, Result};
use ink_wrapper_types::{Connection as _, SignedConnection as _};

use crate::{
    bright_disputes_ink::{Dispute, Instance},
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
                .map(|juror| account_id_to_string(&juror))
                .collect::<Vec<_>>()
        ) // account_id_to_string(&self))
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

    pub async fn vote(
        &self,
        connection: &SignedConnection,
        dispute_id: u32,
        vote: u8,
    ) -> Result<()> {
        let ink_contract: Instance = (&self.contract).into();

        connection.exec(ink_contract.vote(dispute_id, vote)).await?;

        Ok(())
    }

    pub async fn register_as_an_active_juror(&self, connection: &SignedConnection) -> Result<()> {
        let ink_contract: Instance = (&self.contract).into();

        connection
            .exec(ink_contract.register_as_an_active_juror())
            .await?;

        Ok(())
    }

    pub async fn unregister_as_an_active_juror(&self, connection: &SignedConnection) -> Result<()> {
        let ink_contract: Instance = (&self.contract).into();

        connection
            .exec(ink_contract.unregister_as_an_active_juror())
            .await?;

        Ok(())
    }

    pub async fn confirm_juror_participation_in_dispute(
        &self,
        connection: &SignedConnection,
        dispute_id: u32,
        escrow: u128,
    ) -> Result<()> {
        let ink_contract: Instance = (&self.contract).into();

        connection
            .exec(
                ink_contract
                    .confirm_juror_participation_in_dispute(dispute_id)
                    .with_value(escrow),
            )
            .await?;

        Ok(())
    }

    pub async fn confirm_judge_participation_in_dispute(
        &self,
        connection: &SignedConnection,
        dispute_id: u32,
        escrow: u128,
    ) -> Result<()> {
        let ink_contract: Instance = (&self.contract).into();

        connection
            .exec(
                ink_contract
                    .confirm_judge_participation_in_dispute(dispute_id)
                    .with_value(escrow),
            )
            .await?;

        Ok(())
    }

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
