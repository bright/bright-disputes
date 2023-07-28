# Bright Disputes - showcase
This document present a showcase for the Bright Disputes. We are going to use a [Substrate Contract UI](https://contracts-ui.substrate.io/) for interaction with our dApp.

Before starting interaction with smart contract, we will need first to build and deploy it on our local node. To do this please fallow the instruction from the [README](https://github.com/bright/bright-disputes/blob/main/README.md) file. We have provide a set of testing accounts:
[Owner](https://github.com/bright/bright-disputes/blob/main/doc/accounts/5ChhBGUJJLxPk2EJzDN6aeuA7yx7bBBGxgZx5iSr9rMhegrM.json), [Defendant](https://github.com/bright/bright-disputes/blob/main/doc/accounts/5Fhhzf8ZNH2mkP5YddoJ6kj6PfsnB49BxReRopc6CRvqVNrQ.json), [Juror1](https://github.com/bright/bright-disputes/blob/main/doc/accounts/5CFysjxm4tWyePnpELf4xG2o3ZvQV5WVdfvcETn552rYA8h9.json), [Juror2](https://github.com/bright/bright-disputes/blob/main/doc/accounts/5DfNSomECQZkpJJPi8CnBt3aFSAcbDJHy48xaqBkkAc5vVYJ.json), [Juror3](https://github.com/bright/bright-disputes/blob/main/doc/accounts/5CS8L2eS3sbYUcR6b5cvH93DZWiwCGXH4WJzSwTcHmAZekUj.json), [Juror4](https://github.com/bright/bright-disputes/blob/main/doc/accounts/5CSdvQ1mG1j6tsyMib46kFHpwdUqizvWs1NTHGLzQWpNRbrK.json), [Juror5](https://github.com/bright/bright-disputes/blob/main/doc/accounts/5CSvSo9vt1eu4d93EobfA6au8bheGLbkTdvATLb9RPVKgu9b.json), [Juror6](https://github.com/bright/bright-disputes/blob/main/doc/accounts/5CS1o2oMdptJ2owGABQd8Q2TJXSYnLiQjKMWRGnRnSw36RwP.json), [Juror7](https://github.com/bright/bright-disputes/blob/main/doc/accounts/5CSdKZuEYAbaH1nB8rbxqJU5PDtgTtCB5pj4abqQAhimdLU1.json), which are pre founded with some tokens. Password for all accounts is the same and it is: `123456` :)

### Create a dispute
When contract was successfully deployed to the aleph-node, we can create a dispute, by selecting `createDispute` extrinsic. We are going to use `Owner` account as a caller, and point `Defendant` account in the `defendantId`. We should also provide a link for the dispute description (`ownerLink`). Owner should also define an escrow amount, which will need to be deposit by all participates of the dispute. In our case we select `50`, and we also place the same amount in the `Value` filed. Those tokens are going to be transferred from our `Owner` account to the smart contract.

<center>
    
![Extrinsics](https://github.com/bright/bright-disputes/blob/main/doc/images/create_dispute.png)
    
</center> 

### Confirm Defendant
Second step in the dispute process is confirmation of the defendant. We are going to select a `Defendant` account, and chose `confirmDefendant` extrinsic. Inputs for this message are: `dispute_id` which is going to be `1` and the `defendantLink` which is going to be link to defendant description of the dispute.

<center>
    
![Extrinsics](https://github.com/bright/bright-disputes/blob/main/doc/images/confirm_defendant.png)
    
</center> 

### Juror / Judge registration
We are going to register accounts `Juror1`, `Juror2`, `Juror3`, `Juror4`, `Juror5`, `Juror6`, `Juror7` as an juries for the Bright Disputes. Some of them are going to be picked as Juries and one as Judge for our dispute case. First they need to register as an active Juror. We can do it by selecting their accounts as a Callers and picking a `registerAsAnActiveJuror` extrinsic.

<center>
    
![Extrinsics](https://github.com/bright/bright-disputes/blob/main/doc/images/register_juror.png)
    
</center> 

### Process Dispute: assign juries
Now `Owner` of the dispute can start dispute round by calling `processDisputeRound`, as an input passing `dispute_id` which is `1`.

<center>
    
![Extrinsics](https://github.com/bright/bright-disputes/blob/main/doc/images/process_dispute__assign_juries.png)
    
</center> 

Now we can check if Juries and Judge where truly assigned to the dispute. We can call `getDispute` extrinsic, and check accounts of the Juries and Judge:

<center>
    
![Extrinsics](https://github.com/bright/bright-disputes/blob/main/doc/images/get_dispute_picking_juries.png)
    
</center> 

From what we see accounts of `Juror1`, `Juror2`, `Juror3` are assigned as juror role, and the `Juror4` is a judge.

### Juries / Judge confirmation
Next step is to confirm participation of the juries and judge in the dispute. Juries need to call `confirmJurorParticipationInDispute` 

<center>
    
![Extrinsics](https://github.com/bright/bright-disputes/blob/main/doc/images/confirm_juror.png)
    
</center> 

and judge will call `confirmJudgeParticipationInDispute`

<center>
    
![Extrinsics](https://github.com/bright/bright-disputes/blob/main/doc/images/confirm_judge.png)
    
</center> 

Please note that as well juries and the judge will need to pay the same escrow as owner and defendant of the dispute.

### Process Dispute: voting phase
Now `Owner` of the dispute will once again call `processDisputeRound` extrinsic. At this stage, smart contract check if all juries and judge confirms their participation in the dispute. If they have not, than new dispute round will be started and once again juries and judge (whose haven not confirmed their participation) are going to be assigned. No we can move to the voting phase

<center>
    
![Extrinsics](https://github.com/bright/bright-disputes/blob/main/doc/images/get_dispute_start_voting.png)
    
</center> 

where juries are requested to vote. They can do it by calling `vote` extrinsic, and passing `dispute_id` as `1` and vote value which can be `0` - vote against `Defendant` or `1` - vote against `Owner`.

<center>
    
![Extrinsics](https://github.com/bright/bright-disputes/blob/main/doc/images/juror_vote.png)
    
</center> 

We can check if all juries have vote by getting the dispute:

<center>
    
![Extrinsics](https://github.com/bright/bright-disputes/blob/main/doc/images/get_dispute_voting_ends.png)
    
</center> 

### Process Dispute: count the votes
At this stage once again `Owner` of the dispute will call `processDisputeRound` extrinsic. Smart contract will check if all juries have voted and move to the count the votes phase.

<center>
    
![Extrinsics](https://github.com/bright/bright-disputes/blob/main/doc/images/process_dispute__voting.png)
    
</center> 

Now is the judge role to count the votes. In our case judge, who is `Juror4`, will call `processDisputeRound` extrinsic. 

<center>
    
![Extrinsics](https://github.com/bright/bright-disputes/blob/main/doc/images/process_dispute__count_votes.png)
    
</center> 

We can check if the dispute result was set:

<center>
    
![Extrinsics](https://github.com/bright/bright-disputes/blob/main/doc/images/get_dispute_ended.png)
    
</center> 

### Distribute the deposit
The last stage of the dispute process is to split the dispute deposit, by calling `distributeDeposit`. Deposit will be distributed to:
* owner
* defendant
* judge
* juries - who where in the majority of the votes
For all juries and judges who haven't fullfil their duties, their escrow will be lost and split to others. The same rule stands for the juries who were in the minority of the votes.

<center>
    
![Extrinsics](https://github.com/bright/bright-disputes/blob/main/doc/images/distribute_deposit.png)
    
</center> 