

## Build

```
ink-wrapper -m ../contract/target/ink/bright_disputes.json | rustfmt --edition 2021 > src/bright_disputes_ink.rs
```

```
cargo +nightly build --release
```
we need to use `nightly` version, because `liminal-ark-relations v0.4.0` requires it. 

## Run
By running:
```
./cli/target/release/bright_disputes_cli 
```
we will get all possible commands.   
