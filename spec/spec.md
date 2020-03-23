## Specification: Consensus algorithm of Off-Chain trading of data access control rights 

日本語版はこちら(https://github.com/thashimoto1998/keeper-layer2/blob/master/spec/specjp.md)

**System Architecture**
Celer Channel is conditional payment.  There are three key components in the system: CelerPay, CelerApp, and CelerNode.

![1](image/Architecture.png)
**CelerPay** is a generalized payment network that supports efficient off-chain token transfer with the capability to resolve arbitrary conditional dependency on on-chain verifiable states. It consists of a set of on-chain smart contracts and off-chain communication protocols. The shared smart contracts maintain the minimum required on-chain states for each pair of channel peers. The off-chain protocols specify how peers update and exchange off-chain states, and when to make the rare on-chain function calls. CelerPay channels are the edges connecting the state channel network. 

**CelerApp** are generic state channels that can express any application logic. They expose the standard query APIs required by CelerPay, so that payment conditions can be based on CelerApp outcomes. Dashed lines in the figure above indicate CelerApp could be virtual modules. An app contract can be either initially deployed once by the developer and shared by all the future players.

**CelerNodes** are the endpoints that run the state channel protocol of CelerPay and CelerApps. A node can join the state channel network by setting up a CelerPay channel with another node in the network. Once the CelerNode joins the network, it can send off-chain payments to any other nodes in the network. 

＊CelerPay and CelerApp are loosely connected through the simple conditional dependency interface. This allow CelerPay use cases to go significantly beyond state channel applications, because off-chain conditional payment sent through the CelerPay network can be resolved as long as there is an on-chain verifiable conditional state.  Sending a conditional payment with dependency on an outcome from an on-chain oracle.

**Protobuf Messages**
Celer components at different platforms need to support the same set of protobuf messages, which can be categorized into four groups:
[chain.proto](https://github.com/celer-network/cChannel-eth/blob/master/contracts/lib/data/proto/chain.proto) is used only for interactions with CelerPay on-chain smart contracts.
[entity.proto](https://github.com/celer-network/cChannel-eth/blob/master/contracts/lib/data/proto/entity.proto) has CelerPay core data structures for both on-chain and off-chain communications.
[app.proto](https://github.com/celer-network/cApps-eth/blob/master/contracts/lib/proto/app.proto) is used for CelerApp on-chain and off-chain communications.

**Contracts Architecture**
![1](image/contract.png)

White dashed modules at the boatman are user-offchain components. Each colored rectangle is an individual on-chain contract. Blue modules are CelerPay contracts (ones with dashed border are upgradable); green modules are external arbitrary condition contracts; orange arrows are external function calls (with single word functionality summaries) among contracts; black arrows are external function calls from CelerNodes (off-chain users).

**CelerWallet**
The CelerWallet contract keeps the multi-owner and multi-token wallets for all the payment channels. CelerWallet only holds tokens for the channel peers without any complicated payment channel logics, which are programmed in the CelerLedger contract. Is is extremely robust and safe due to its simplicity. Payment channel peers (CelerNodes) do not directly interact with the CelerWallet contract to operate their funds, but the wallet operator: the CelerLedger contract, which we describe below.

**CelerLedger**
CelerLedger is central of all CelerPay contracts, and the entry point of most of the [on chain user operations](https://www.celer.network/docs/celercore/channel/pay_contracts.html#channel-operations). 
It defines the CelerPay on-chain state machine, maintains the core logic of a payment channel, acts as the operator of CelerWallet to operate on the token assets, and expose a rich set of [APIs](https://github.com/celer-network/cChannel-eth/blob/master/contracts/lib/interface/ICelerLedger.sol) for users (channel peers) to manage the payment channels. CelerLedger calls the external functions of three contracts when executing its logic:
* To CelerWallet: operation on CelerWallet to deposit/withdraw funds, or transfer operatorship.
* To EthPool: transfer ETH to CelerWallet, enable the single-transaction channel opening.
* To PayRegistry: query about the resolved payment amount when settling a channel.

**PayResolver**
PayResolver defines the payment resolving logic. It exposes two [APIs](https://github.com/celer-network/cChannel-eth/blob/master/contracts/lib/interface/IPayResolver.sol) to let a CelerNode resolve a payment on-chain if it cannot clear the payment off-chain with its channel peer cooperatively. PayResolver call external functions of other contracts when executing its logic:
* To PayRegistry: set the resolved payment amount in the global payment information registry.
* To Conditions: query the condition outcomes when computing the payment finalized amount.

**PayRegistry**
PayRegistry is the global registry to store the resolved amount of all payments. It exposes simple [APIs](https://github.com/celer-network/cChannel-eth/blob/master/contracts/lib/interface/IPayRegistry.sol) for anyone to set a payment result indexed by the payment ID. PayRegistry calculates the payment ID as `payID = Hash(Hash(pay), setterAddress)`, where setter is usually the PayResolver. In this way, each payment’s result can only be set int the registry by its self-specified resolver contract (field 8 of the [ConditionalPay message](https://www.celer.network/docs/celercore/channel/pay_contracts.html#conditional-payment). A payment result becomes immutable and publicly available once it is finalized on the PayRegistry. Then all channels that have the payment in pending status can use the result from the registry to clear the payment either off-chain or on-chain.

**EthPool**
EthPool is a simple ETH wallet contract that provides ERC-20-like APIs for ETH, and an additional API to make ETH deposit into CelerPay more flexible and efficient. EthPool enables the single-transaction channel opening feature of CelerPay.

**Conditions**
Conditions are not part of the CelerPay contracts, but external [CelerApp](https://www.celer.network/docs/celercore/channel/app.html) contracts for the PayResolver to query through `isFinalized()` and `getOutcome()` APIs when resolving  payments. A condition contract can be an initially on-chain deployed-contract.


## Flows
This section describes detail flow. 

## When PUBLISHERS and CONSUMERS contract for the first time and CONSUMERS and PUBLISHERS is cooperative.
	
![for-the-first-time](image/for-the-first-time-3.png)
	
**Deploy**
DID PUBLISHERS deploy ![AccessSecretRegistry.sol](https://github.com/thashimoto1998/keeper-layer2/blob/master/contracts/keeper-layer2/registry/AccessSecretRegistry.sol). This contract is used for on-chain oracle. Sending a conditional payment with dependency on an outcome from this smart contract. Outcome is `isFinalized()` and `getOutcome()`. When `IsFinalized()` and `getOutcome()` is true, PUBLISHERS can get token. Access agreement outcome is `checkPermissions()`. When `checkPermissions()` is true, CONSUMERS can access document . 


**Open Channel**
The CelerLedger contract expose an `openChannel()` API which allows a funded payment channel to be open a single transaction. The API takes a single input, which is the channel peer co-signed payment channel initializer message. Once the CelerLedger contract receives a valid open channel request, it will execute the following operations in a single transaction:
1. Create a wallet in the CelerWallet contract and use the returned wallet ID as the channel ID, which is computed as 
    `Hash(walletAddress, ledgerAddress, Hash(channelInitializer))`.
2. Initialize the channel state in the CelerLedger contract.
3. Accept the blockchain native tokens (ETH) sent along with the transaction request, and transfer tokens from the peer’s approved token pools (e.g.EthPool or ERC20 contracts) to the CelerWallet according to the requested initial distribution Amounts.

**Send Conditional Payment**
Sending a conditional payment is essentially creating a new co-signed [simplex channel state](https://www.celer.network/docs/celercore/channel/pay_contracts.html#simplex-channel-state) to add a new entry in the pending payId list (field 5) and update other related fields. Two off-chain messages(`CondPayRequest` and `CondPayResponse`) in one round trip are involved during the process. `CondPayRequest` is the single-hop message sent by the peer who wants to send or forward the conditional payment. It mainly consists of the following information:
1. Payment data: the immutable [conditional payment](https://www.celer.network/docs/celercore/channel/pay_contracts.html#conditional-payment) message set by the payment source.
2. New one-sig state: the new [simplex state](https://www.celer.network/docs/celercore/channel/pay_contracts.html#simplex-channel-state) with the signature of peer_from. The new state should have a higher sequence number, new pending payId list (field 5) that includes the  new conditional payment ID, and updated channel metadata(field 6 and field 7).
3. Base seq: the sequence number of the previous simplex state on which this new state is based.
4. Pay note: a payment note with `google.protobuf.Any` type that can describe any information which might be useful for off-chain communication.

**`CondPayResponse`** is the replied message from receiving peer after checking the validity of every data field in the request. The response consists of two fields:
1. Co-Signed state: the latest co-signed [simplex state](https://www.celer.network/docs/celercore/channel/pay_contracts.html#simplex-channel-state). This sate should be the same as the state in the `CondPayRequest` if the request is valid. Otherwise (e.g. invalid sequence number due to packet loss), the latest co-signed state stored by the receiving peer is replied to the peer_from to help failure recovery (e.g., resending lost previous request).
2. Error: an optional error message with the error reason and the sequence number of the errored request. The peer_from sender is responsible for remembering and funding out the sent request based on the NACked sequence number.

**Send State Proof Request(state is key of did)**
Sending a [StateProof](https://github.com/celer-network/cApps-eth/blob/master/contracts/lib/proto/app.proto) is essentially creating a new consigned state.
It mainly consists of the following information:
1. New one-sig state: the new state with the signature of peer_from. The new state should have a higher sequence number. state is key of `didList (did(bytes32)=> key(uint8))`.
2. seq: the sequence number of the previous state on which this new state is based.

**`StateProofResponse`** is the replied message from receiving peer after checking the validity of every data field in the request. The response consists of two fields:
1. Co-Signed state: the latest co-signed state.
2. Error: an optional error message with the error reason or the sequence number of the errors request. 

**intendSettle(state is key of did)**
Submit and settle off-chain state to update state according to an off-chain state proof. Outcome `isFinalized()`, `getOutcome()` and `checkPermissions()` will be true after checking validity (co-signed, state is valid).

**Settle Conditional Payment**
After a conditional payment is successfully setup, two peers can cooperatively settle the payment off-chain once the condition outcomes are finalized. Settling a conditional payment is essentially creating a new co-signed [simplex channel state](https://www.celer.network/docs/celercore/channel/pay_contracts.html#simplex-channel-state) to remove an entry from the pending payId list (field 5) and update the transferred amount(field 4) and other related fields. 

`PaymentSettleRequest` is the single-hop message sent by peer_from side of the channel to clear a payment. It mainly consists of the following information:
1. Payments to be settled: a list if payment IDs to be settled, their settle reasons(e.g. fully paid, expired, rejected, on-chain resolved), and the settled amounts.
2. New one-sig state: the new [simplex state](https://www.celer.network/docs/celercore/channel/pay_contracts.html#simplex-channel-state) with the signature of peer_from. The new state should have a higher sequence number (field 3), new pending payId list  (field 5) that removes the IDs of settled payments, and updated transferred amount (field 4) and total pending amount (field 7).
3. Base seq: the sequence number of the previous simplex state on which this new state is based.

**`PaymentSettleResponse`** is the replied message from the receiving peer after checking the validity of the request. It has the two fields with the `CondPayResponse` described above: a co-signed simplex state, and an optional error message.

## When PUBLISHER are motivated to dispute the payment in case of uncooperative behaviors of CONSUMER.
**When CONSUMER doesn’t send `PaymentSettleRequest` after `intendSettle()`to AccessSecretRegistry.sol or `PaymentSettleRequest` is not expected.**

![2](image/dispute-2.png)
**Resolve Payment by Condition**
If not receive the settlement as expected, PUBLISHER can choose to submit an on-chain transaction to resolve the payment by conditions once conditions of a payment are finalized on-chain. `resolvePaymentByConditions()` API input consists of two pieces of information: 1) the full conditional payment data and 2) all hash preimgaes to the hash locks associated with the payment. Then the PayResolver will verify the hash preimages, query the conditions outcomes, then compute and set the payment result in the PayRegistry. PUBLISHER should send the `PaymentSettleProof` message to the CONSUMER to ask for the settlement. `PaymentSettleProof` is used by the receiving peer to initiating a settlement process. After payment is resolved on-chain and CONSUMER will be cooperative, CONSUMER send valid `PaymentSettleRequest` and PUBLISHER return `PaymentSettleResponse`.

**When CONSUMER or PUBLISHER is uncooperative after payment is resolved on-chain.**

![3](image/consumer-uncooperative-2.png)

**Settle/Close the payment channel**
If cooperative settling is not possible, PUBLISHERS can initiate a unilateral settling by calling the `intendSettle()` API, which takes the co-signed off-chain simplex states as input. The CelerLedger contract will compute the settled balance distributions based on the simplex states and the results of pending payments queried from the PayRegistry.
A challenge time window is opened after the unilateral settlement request, for the other peer to submit simplex channel states with higher sequence numbers if exists. After the challenge window is closed, one can call the `confirmSettle()` API to finish the operation and close the channel.

## When PUBLISHER and CONSUMER want to contract another data.

![3](image/another-did-2.png)

REQUIREMENT: The security assumption of the applications on which conditional payments depend so we should not update `isFinalized()` and `getOutcome()` unintentionally.

Set another DID.
PUBLISHER call `setDID()` to AccessSecretRegistry to set another DID.

Send State Proof Request (state is -2)
When CONSUMER `intendSettle()`(state is -2) to AccessSecretRegistry.sol, `AppStatus.FINALIZED -> ApStatus.IDLE`

## When PUBLISHER and CONSUMER want to swap positions.

![5](image/swap-2.png)

REQUIREMENT: The security assumption of the applications on which conditional payments depend so we should not update `isFinalized()` and `getOutcome()` unintentionally.

Send State Proof Request (state is -1)
When CONSUMER `intendSettle()` (state is -1) to AccessSecretRegistry.sol, owner <-swap-> grantee.


Reference: celercore: (https://www.celer.network/docs/celercore/index.html)
