
# Requirements 

Install ink-wrapper
```
cargo install ink-wrapper --locked --force --version 0.4.1
```

and use it with the smart contract

```
ink-wrapper -m ../contract/target/ink/bright_disputes.json | rustfmt --edition 2021 > bright_disputes.rs
```