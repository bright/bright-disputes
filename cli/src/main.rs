use aleph_client::{
    account_from_keypair, keypair_from_string, AccountId, Connection, SignedConnection,
};
use anyhow::{anyhow, Result};
use bright_disputes::{application::Application, bright_disputes::BrightDisputes};
use clap::Parser;
use std::str::FromStr;
use std::{env, io};
extern crate bright_disputes;
use inquire::Text;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::config::{Command::SetNode, ContractCmd};

mod config;
mod helpers;
use crate::{
    config::{
        Command, Config,
        ContractCmd::{
            ConfirmDefendant, ConfirmJudgeParticipation, ConfirmJurorParticipation, CountTheVotes,
            CreateDispute, DistributeDeposit, GetDispute, GetDisputeFull, ProcessDisputeRound,
            RegisterAsAnActiveJuror, UnregisterAsAnActiveJuror, UpdateDefendantDescription,
            UpdateOwnerDescription, Vote,
        },
    },
    helpers::to_ink_account_id,
    Command::{Contract, SetContract},
};

async fn handle_contract_command(
    app: &mut Application,
    cmd: ContractCmd,
) -> Result<(), Box<dyn std::error::Error>> {
    let contract_address = match app.contract_address.clone() {
        Some(contract_address) => contract_address,
        None => {
            let address = Text::new("Contract address:").prompt()?;
            app.contract_address = Some(address.clone());
            address
        }
    };

    let connection = Connection::new(&app.node_address).await;

    let contract_address = match AccountId::from_str(&contract_address) {
        Ok(address) => address,
        _ => return Err("Invalid contract address!".into()),
    };

    let bright_dispute = BrightDisputes::new(&contract_address, &app.metadata_path)?;

    match cmd {
        CreateDispute {
            caller_account,
            defendant_seed,
            owner_link,
            escrow,
        } => {
            let account = keypair_from_string(&caller_account);
            let signed_connection = SignedConnection::from_connection(connection, account.clone());

            let defendant_key = keypair_from_string(&defendant_seed);
            let defendant_account = account_from_keypair(defendant_key.signer());

            let dispute_id = bright_dispute
                .create_dispute(
                    &signed_connection,
                    owner_link,
                    to_ink_account_id(&defendant_account),
                    escrow,
                )
                .await?;
            info!("New dispute created, id: {dispute_id}");
        }
        ConfirmDefendant {
            caller_account,
            dispute_id,
            defendant_link,
        } => {
            let account = keypair_from_string(&caller_account);
            let signed_connection = SignedConnection::from_connection(connection, account.clone());

            let dispute = bright_dispute
                .get_dispute(&signed_connection, dispute_id)
                .await?;

            bright_dispute
                .confirm_defendant(
                    &signed_connection,
                    dispute_id,
                    defendant_link,
                    dispute.escrow,
                )
                .await?;
            info!(
                "Defendant confirmed his participation in the dispute: {}!",
                dispute_id
            );
        }
        CountTheVotes {
            caller_account,
            dispute_id,
            private_key,
        } => {
            let account = keypair_from_string(&caller_account);
            let signed_connection = SignedConnection::from_connection(connection, account.clone());

            bright_dispute
                .count_the_votes(
                    &signed_connection,
                    dispute_id,
                    private_key,
                    &app.verdict_none_pk,
                    &app.verdict_negative_pk,
                    &app.verdict_positive_pk,
                )
                .await?;
            info!("Votes counted: {}!", dispute_id);
        }
        GetDispute {
            caller_account,
            dispute_id,
        } => {
            let account = keypair_from_string(&caller_account);
            let signed_connection = SignedConnection::from_connection(connection, account.clone());

            let dispute = bright_dispute
                .get_dispute(&signed_connection, dispute_id)
                .await?;
            info!("{}", dispute);
        }
        GetDisputeFull {
            caller_account,
            dispute_id,
        } => {
            let account = keypair_from_string(&caller_account);
            let signed_connection = SignedConnection::from_connection(connection, account.clone());

            let dispute = bright_dispute
                .get_dispute(&signed_connection, dispute_id)
                .await?;
            info!(?dispute);
        }
        UpdateOwnerDescription {
            caller_account,
            dispute_id,
            owner_link,
        } => {
            let account = keypair_from_string(&caller_account);
            let signed_connection = SignedConnection::from_connection(connection, account.clone());

            bright_dispute
                .update_owner_description(&signed_connection, dispute_id, owner_link)
                .await?;
            info!("Updated Owner description link!");
        }
        UpdateDefendantDescription {
            caller_account,
            dispute_id,
            defendant_link,
        } => {
            let account = keypair_from_string(&caller_account);
            let signed_connection = SignedConnection::from_connection(connection, account.clone());

            bright_dispute
                .update_defendant_description(&signed_connection, dispute_id, defendant_link)
                .await?;
            info!("Updated Defendant description link!");
        }
        Vote {
            caller_account,
            dispute_id,
            vote,
            private_key,
        } => {
            let account = keypair_from_string(&caller_account);
            let signed_connection = SignedConnection::from_connection(connection, account.clone());

            bright_dispute
                .vote(
                    &signed_connection,
                    dispute_id,
                    private_key,
                    vote,
                    &app.vote_pk,
                )
                .await?;
            info!("Voting succeed!");
        }
        RegisterAsAnActiveJuror { caller_account } => {
            let account = keypair_from_string(&caller_account);
            let signed_connection = SignedConnection::from_connection(connection, account.clone());

            bright_dispute
                .register_as_an_active_juror(&signed_connection)
                .await?;
            info!(
                "\"{}\" [{}] has registered for the Active Jurors Pool!",
                caller_account,
                account.account_id()
            );
        }
        UnregisterAsAnActiveJuror { caller_account } => {
            let account = keypair_from_string(&caller_account);
            let signed_connection = SignedConnection::from_connection(connection, account.clone());

            bright_dispute
                .unregister_as_an_active_juror(&signed_connection)
                .await?;
            info!(
                "\"{}\" has unregistered from the Active Jurors Pool!",
                caller_account
            );
        }
        ConfirmJurorParticipation {
            caller_account,
            dispute_id,
        } => {
            let account = keypair_from_string(&caller_account);
            let signed_connection = SignedConnection::from_connection(connection, account.clone());

            let dispute = bright_dispute
                .get_dispute(&signed_connection, dispute_id)
                .await?;
            bright_dispute
                .confirm_juror_participation_in_dispute(
                    &signed_connection,
                    dispute_id,
                    dispute.escrow,
                )
                .await?;
            info!(
                "Juror \"{}\", has confirmed his participation in the dispute: {}!",
                caller_account, dispute_id
            );
        }
        ConfirmJudgeParticipation {
            caller_account,
            dispute_id,
        } => {
            let account = keypair_from_string(&caller_account);
            let signed_connection = SignedConnection::from_connection(connection, account.clone());

            let dispute = bright_dispute
                .get_dispute(&signed_connection, dispute_id)
                .await?;

            bright_dispute
                .confirm_judge_participation_in_dispute(
                    &signed_connection,
                    dispute_id,
                    dispute.escrow,
                )
                .await?;
            info!(
                "Judge \"{}\", has confirmed his participation in the dispute: {}!",
                caller_account, dispute_id
            );
        }
        ProcessDisputeRound {
            caller_account,
            dispute_id,
        } => {
            let account = keypair_from_string(&caller_account);
            let signed_connection = SignedConnection::from_connection(connection, account.clone());

            bright_dispute
                .process_dispute_round(&signed_connection, dispute_id)
                .await?;

            let dispute = bright_dispute
                .get_dispute(&signed_connection, dispute_id)
                .await?;
            let dispute_state = dispute.dispute_round.unwrap().state;
            info!(
                "Successfully switched to the next dispute state: {:?}",
                dispute_state
            );
        }
        DistributeDeposit {
            caller_account,
            dispute_id,
        } => {
            let account = keypair_from_string(&caller_account);
            let signed_connection = SignedConnection::from_connection(connection, account.clone());

            bright_dispute
                .distribute_deposit(&signed_connection, dispute_id)
                .await?;
            info!("Deposit distributed successfully!");
        }
    }
    Ok(())
}

fn setup_logging() -> Result<()> {
    let filter = EnvFilter::new(
        env::var("RUST_LOG")
            .as_deref()
            .unwrap_or("warn,bright_disputes_cli=info"),
    );

    let subscriber = tracing_subscriber::fmt()
        .with_writer(io::stdout)
        .with_target(false)
        .with_env_filter(filter);
    subscriber.try_init().map_err(|err| anyhow!(err))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logging()?;

    let config = Config::parse();
    let mut app = Application::load_or_create(&config.config_path)?;

    match config.command.clone() {
        SetNode { node_address } => {
            app.node_address = node_address.clone();
            info!("Node address set to: {:?}", node_address);
        }
        SetContract { contract_address } => {
            app.contract_address = Some(contract_address.clone());
            info!("Contract address set to: {:?}", contract_address);
        }
        Contract(cmd) => {
            handle_contract_command(&mut app, cmd).await?;
        }
    }

    Application::save(&config.config_path, &app)?;
    Ok(())
}
