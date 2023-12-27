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
{
 Dispute: 1 
 Owner: 5FTyuyEQQZs8tCcPTUFqotkm2SYfDnpefn9FitRgmTHnFDBD 
 Defendant: 5HpJbr84AqocNWyq4WNAQLNLSNNoXVmqAhvrk8Tq7YX23j6p 
 Judge: 5H4SHcV6XVFiGF3QGdKFLES3xwUmN9jFdt5KVNapJtfWPPtT 
 Jurors: ["5GvG1edSDSrAG5HZ21N1BVGEgygpSujAAjuruyfyuCgsgEFr", "5DZyhVcMqnfg78WK8EsUyu3tpLb2peARqVEoieEunsgH2iQb", "5FjEKpjdvNe8SbZjgUaF8qQD3mL9T5k8oCtGozMYkHi2aVCi"] 
}
```
we can find out which Jurors where picked from the active pool. In our case it was: //Juror1, //Juror2, //Juror3, and //Juror4 as a Judge. Now they need to confirm their participation in the dispute, we can do it by calling:
```
../cli/target/release/bright_disputes_cli confirm-juror-participation //Juror1 1
../cli/target/release/bright_disputes_cli confirm-juror-participation //Juror2 1
../cli/target/release/bright_disputes_cli confirm-juror-participation //Juror3 1
../cli/target/release/bright_disputes_cli confirm-judge-participation //Juror4 1
```
When confirming their participation, each juror will receive a unique pair of keys, which will be used for the further data encryption. In our case, this present:

Juror1
```
Public key: [143,96,146,215,67,186,237,47,231,60,4,227,180,180,227,175,139,11,9,212,45,153,174,82,61,94,185,142,229,93,248,141]

Private key: [179,168,214,171,19,120,215,166,1,175,173,235,85,161,223,244,253,121,185,141,92,32,171,52,154,20,21,152,97,250,190,6]
```

Juror2
```
Public key: [93,66,190,16,93,13,181,112,42,68,88,90,88,65,241,30,80,202,221,3,137,104,89,40,93,2,69,100,36,104,158,72] 

Private key: [250,71,107,70,4,169,48,81,216,97,222,161,213,137,52,53,250,3,165,188,184,58,181,151,160,0,153,178,252,164,62,7]
```

Juror3
```
Public key: [199,48,32,250,139,107,224,127,96,217,223,140,130,3,111,69,146,249,47,219,36,50,38,216,154,163,197,232,65,72,57,115] 

Private key: [214,199,173,149,198,9,55,208,20,165,65,187,65,103,253,211,174,91,92,193,21,244,157,43,215,163,41,161,15,65,106,4]
```

Juror4
```
Public key: [209,186,95,203,236,84,246,136,3,232,135,235,5,218,13,168,128,89,67,143,5,125,187,223,178,40,113,238,18,97,242,81] 

Private key: [25,164,133,151,251,54,205,192,212,173,218,155,210,238,98,4,36,68,162,114,94,30,134,181,187,167,219,131,227,25,202,6]
```

When all Jurors and Judge confirms participation in the dispute, we can proceed with the dispute round:
```
../cli/target/release/bright_disputes_cli process-dispute-round //Owner 1
```
and start a `Voting` phase, where Jurors can vote against one of the parties. For this dispute all jurors will vote against the owner:
```
../cli/target/release/bright_disputes_cli vote //Juror1 1 1 179,168,214,171,19,120,215,166,1,175,173,235,85,161,223,244,253,121,185,141,92,32,171,52,154,20,21,152,97,250,190,6
```

```
../cli/target/release/bright_disputes_cli vote //Juror2 1 1 250,71,107,70,4,169,48,81,216,97,222,161,213,137,52,53,250,3,165,188,184,58,181,151,160,0,153,178,252,164,62,7
```

```
../cli/target/release/bright_disputes_cli vote //Juror3 1 1 214,199,173,149,198,9,55,208,20,165,65,187,65,103,253,211,174,91,92,193,21,244,157,43,215,163,41,161,15,65,106,4
```
Once again we need to proceed with the dispute round:
```
../cli/target/release/bright_disputes_cli process-dispute-round //Owner 1
```
which moves us to the next phase which is `Counting the Votes`. Now the role of Judge came in, and he need to count the votes:
```
../cli/target/release/bright_disputes_cli count-the-votes //Juror4 1 25,164,133,151,251,54,205,192,212,173,218,155,210,238,98,4,36,68,162,114,94,30,134,181,187,167,219,131,227,25,202,6
```
finally we can finish the dispute and distribute the deposit:
```
../cli/target/release/bright_disputes_cli distribute-deposit //Owner 1
```

