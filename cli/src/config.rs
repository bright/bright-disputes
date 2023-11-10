use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Config {
    #[clap(long, default_value = ".bright_disputes_config.json")]
    pub config_path: PathBuf,

    #[clap(long, default_value = "ws://127.0.0.1:9944")]
    pub node_address: String,

    #[clap(long)]
    pub contract_address: Option<String>,

    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Clone, Subcommand)]
pub enum Command {
    /// Set node address
    SetNode { node_address: String },
    /// Set smart contract address
    SetContract { contract_address: String },
    /// Contract command
    #[clap(flatten)]
    Contract(ContractCmd),
}

#[derive(Clone, Subcommand)]
pub enum ContractCmd {
    /// Create new dispute
    CreateDispute {
        caller_account: String,
        defendant_seed: String,
        owner_link: String,
        escrow: u128,
    },
    /// Confirms defendant
    ConfirmDefendant {
        caller_account: String,
        dispute_id: u32,
        defendant_link: String,
    },
    /// Counts the votes
    CountTheVotes {
        caller_account: String,
        dispute_id: u32,
        #[clap(value_parser, num_args = 1.., value_delimiter = ',')]
        private_key: Vec<u8>,
    },
    /// Get dispute
    GetDispute {
        caller_account: String,
        dispute_id: u32,
    },
    /// Get dispute
    GetDisputeFull {
        caller_account: String,
        dispute_id: u32,
    },
    /// Update owner description of the dispute
    UpdateOwnerDescription {
        caller_account: String,
        dispute_id: u32,
        owner_link: String,
    },
    /// Update defendant description of the dispute
    UpdateDefendantDescription {
        caller_account: String,
        dispute_id: u32,
        defendant_link: String,
    },
    /// Make a vote (call by juror)
    Vote {
        caller_account: String,
        dispute_id: u32,
        vote: u8,
        #[clap(value_parser, num_args = 1.., value_delimiter = ',')]
        private_key: Vec<u8>,
    },
    /// Register as an active juror in bright disputes
    RegisterAsAnActiveJuror { caller_account: String },
    /// Unregister from being an active juror in bright disputes
    UnregisterAsAnActiveJuror { caller_account: String },
    /// Confirms juror participation in the dispute
    ConfirmJurorParticipation {
        caller_account: String,
        dispute_id: u32,
    },
    /// Confirms judge participation in the dispute
    ConfirmJudgeParticipation {
        caller_account: String,
        dispute_id: u32,
    },
    /// Process dispute round
    ProcessDisputeRound {
        caller_account: String,
        dispute_id: u32,
    },
    /// Distribute dispute deposit
    DistributeDeposit {
        caller_account: String,
        dispute_id: u32,
    },
}
