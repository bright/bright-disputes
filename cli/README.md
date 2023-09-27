# Bright Disputes - CLI
A command line tool for interacting with the Bright Disputes smart contract.

## Build
Please note, in case of changing Bright Disputes smart contract, it is necessary to generate a safe code: 
```
ink-wrapper -m ../contract/target/ink/bright_disputes.json | rustfmt --edition 2021 > src/bright_disputes_ink.rs
```

To build a CLI we need to run:
```
cargo +nightly-2023-04-19 build --release
```
we need to use `nightly` version, because `liminal-ark-relations v0.4.0` requires it. 

## Run
By running:
```
./cli/target/release/bright_disputes_cli 
```
we will get all possible commands:
```
Usage: bright_disputes_cli [OPTIONS] <COMMAND>

Commands:
  set-node                       Set node address
  set-contract                   Set smart contract address
  create-dispute                 Create new dispute
  confirm-defendant              Confirms defendant
  get-dispute                    Get dispute
  get-dispute-full               Get dispute
  update-owner-description       Update owner description of the dispute
  update-defendant-description   Update defendant description of the dispute
  vote                           Make a vote (call by juror)
  register-as-an-active-juror    Register as an active juror in bright disputes
  unregister-as-an-active-juror  Unregister from being an active juror in bright disputes
  confirm-juror-participation    Confirms juror participation in the dispute
  confirm-judge-participation    Confirms judge participation in the dispute
  process-dispute-round          Process dispute round
  distribute-deposit             Distribute dispute deposit
  help                           Print this message or the help of the given subcommand(s)

Options:
      --config-path <CONFIG_PATH>            [default: .bright_disputes_config.json]
      --node-address <NODE_ADDRESS>          [default: ws://127.0.0.1:9944]
      --contract-address <CONTRACT_ADDRESS>  
  -h, --help                                 Print help
```
More details of how to use a CLI, can be found in the [showcase](https://github.com/bright/bright-disputes/blob/main/doc/README_CLI.md).
