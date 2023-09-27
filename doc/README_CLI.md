# Bright Disputes - showcase (cli)

To build and deploy Bright Disputes smart contract we need to run:
```
bash ../scripts/deploy.sh
```
This script will do few things:
* run aleph-node
* pre-found accounts: `Owner`, `Defendant`, `Juror1`, `Juror2`, `Juror3`, `Juror4`, `Juror5`, `Juror6`, `Juror7`
* build and deploy `Bright Disputes` smart contract to aleph-node
* build cli

When the `deploy.sh` script succeed our smart contract will be deployed on the aleph-node. In the output we can find address of where we can find our smart contract:
```
[2023-08-03 13:21:42] [INFO] Contract address: 5F3bNYZMgeDvqVCnS4sMdWavuyWDwht6yuK8sL2gxc3PP2xg
```
or we can find it in the file `scripts/addresses.json`.

Cli allows us to set smart contract address by calling:
```
../cli/target/release/bright_disputes_cli set-contract 5F3bNYZMgeDvqVCnS4sMdWavuyWDwht6yuK8sL2gxc3PP2xg
```

We will start from creating a new dispute, and we are going to use accounts from our script:
```
../cli/target/release/bright_disputes_cli create-dispute //Owner //Defendant "https://brightinventions.pl/" 100
```
The next step is to confirm dispute by the defendant. In our case defendant account is `//Defendant`, so we can call:
```
../cli/target/release/bright_disputes_cli confirm-defendant //Defendant 1 "https://brightinventions.pl/"
```
Before we start our dispute, we will need first to register some jurors. Some of them are going to be assigned to our dispute:
```
../cli/target/release/bright_disputes_cli register-as-an-active-juror //Juror1
../cli/target/release/bright_disputes_cli register-as-an-active-juror //Juror2
../cli/target/release/bright_disputes_cli register-as-an-active-juror //Juror3
../cli/target/release/bright_disputes_cli register-as-an-active-juror //Juror4
../cli/target/release/bright_disputes_cli register-as-an-active-juror //Juror5
../cli/target/release/bright_disputes_cli register-as-an-active-juror //Juror6
../cli/target/release/bright_disputes_cli register-as-an-active-juror //Juror7
```
Now we can process with the dispute. As an owner of the dispute we need to call:
```
../cli/target/release/bright_disputes_cli process-dispute-round //Owner 1
```
This call will start a dispute round, which will assign Jurors and the Judge to the dispute. Now they need to confirm their participation in the dispute. We can check which Judge and Jurors were assigned, by calling:
```
../cli/target/release/bright_disputes_cli get-dispute //Owner 1
```
based on the output:
```
Dispute: 1 
Owner: 5FTyuyEQQZs8tCcPTUFqotkm2SYfDnpefn9FitRgmTHnFDBD 
Defendant: 5HpJbr84AqocNWyq4WNAQLNLSNNoXVmqAhvrk8Tq7YX23j6p 
Judge: 5GvG1edSDSrAG5HZ21N1BVGEgygpSujAAjuruyfyuCgsgEFr 
Jurors: ["5H4SHcV6XVFiGF3QGdKFLES3xwUmN9jFdt5KVNapJtfWPPtT", "5G492oT3GwqTpz4ebV15JHERucL96zEp54TZZSm3ZQHGe9AE","5GjNM6gLeYxeB9aQoPxVVa7H494ijFsHTXTNo9dkNuTyDCeD"] 
```
we can find out which Jurors where picked from the active pool. In our case it was: //Juror4, //Juror5, //Juror6, and //Juror1 as a Judge. Now they need to confirm their participation in the dispute, we can do it by calling:
```
../cli/target/release/bright_disputes_cli confirm-juror-participation //Juror4 1
../cli/target/release/bright_disputes_cli confirm-juror-participation //Juror5 1
../cli/target/release/bright_disputes_cli confirm-juror-participation //Juror6 1
../cli/target/release/bright_disputes_cli confirm-judge-participation //Juror1 1
```
When all Jurors and Judge confirms participation in the dispute, we can proceed with the dispute round:
```
../cli/target/release/bright_disputes_cli process-dispute-round //Owner 1
```
and start a `Voting` phase, where Jurors can vote against one of the parties:
```
../cli/target/release/bright_disputes_cli vote //Juror4 1 1
../cli/target/release/bright_disputes_cli vote //Juror5 1 1
../cli/target/release/bright_disputes_cli vote //Juror6 1 1
```
Once again we need to proceed with the dispute round:
```
../cli/target/release/bright_disputes_cli process-dispute-round //Owner 1
```
which moves us to the next phase which is `Counting the Votes`. Now the role of Judge came in, and he need to count the votes:
```
../cli/target/release/bright_disputes_cli process-dispute-round //Juror1 1
```
finally we can finish the dispute and distribute the deposit:
```
../cli/target/release/bright_disputes_cli distribute-deposit //Owner 1
```

