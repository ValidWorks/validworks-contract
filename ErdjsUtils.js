import {
  ContractCallPayloadBuilder,
  ProxyProvider,
  Address,
  HWProvider,
  Account,
  Balance,
  Transaction,
  U64Value,
  BigUIntValue,
  ContractFunction,
  AddressValue,
  GasLimit,
} from "@elrondnetwork/erdjs";

import BigNumber from "bignumber.js";

// Change network (test/dev/main)
const proxyProvider = new ProxyProvider(
  "https://testnet-gateway.elrond.com",
  10000
);
// Ledger integration
const hwWalletP = new HWProvider(proxyProvider);
const smartContractAddress = new Address(
  "erd1qqqqqqqqqqqqqpgqqr37qsc5lyue3ketjjh90jnwwsaypx9md8ss4w9n7k"
);

const connectLedger = async () => {
  hwWalletP
    .init()
    .then((success) => {
      if (!success) {
        console.warn(
          "could not initialise ledger app, make sure Elrond app is open"
        );
        return;
      }

      hwWalletP
        .login()
        .then((address) => {
          console.log(address);
        })
        .catch((err) => {
          console.warn(err);
        });
    })
    .catch((error) => {
      console.error("error ", error);
    });
};

/* 
@param caller_address: String
@param gig_id: number
@param deadline: number
@param price (in eGLD): number 
*/
const sellerList = async (caller_address, gig_id, deadline, price) => {
  // SYNC NONCE
  let caller = new Account(new Address(caller_address));
  await caller.sync(proxyProvider);
  // LOAD PAYLOAD
  let payload_builder = new ContractCallPayloadBuilder();
  payload_builder.setFunction(new ContractFunction("list")); // function
  payload_builder.addArg(new U64Value(new BigNumber(gig_id))); // gig-id
  payload_builder.addArg(new U64Value(new BigNumber(deadline))); // deadline
  payload_builder.addArg(
    new BigUIntValue(new BigNumber(price * 1000000000000000000))
  ); // price
  // BUIDL
  let payload = payload_builder.build();
  // MAKE TRANSACTION
  let tx = new Transaction({
    receiver: smartContractAddress,
    nonce: caller.nonce,
    gasLimit: new GasLimit(50000000),
    data: payload,
  });
  // SEND IT
  let reply = await hwWalletP.sendTransaction(tx);
  // Get the transaction object first
  await reply.awaitExecuted(proxyProvider);
  // Then wait for it to get executed
  return reply;
  // return the transaction
};

/* 
@param caller_address: String
@param gig_id: number
*/
const sellerUnlist = async (caller_address, gig_id) => {
  // SYNC NONCE
  let caller = new Account(new Address(caller_address));
  await caller.sync(proxyProvider);
  // LOAD PAYLOAD
  let payload_builder = new ContractCallPayloadBuilder();
  payload_builder.setFunction(new ContractFunction("unlist")); // function
  payload_builder.addArg(new U64Value(new BigNumber(gig_id))); // gig-id
  // BUIDL
  let payload = payload_builder.build();
  // MAKE TRANSACTION
  let tx = new Transaction({
    receiver: smartContractAddress,
    nonce: caller.nonce,
    gasLimit: new GasLimit(50000000),
    data: payload,
  });
  // SEND IT
  let reply = await hwWalletP.sendTransaction(tx);
  // Get the transaction object first
  await reply.awaitExecuted(proxyProvider);
  // Then wait for it to get executed
  return reply;
  // return the transaction
};

/* 
@param caller_address: String
@param gig_id: number
*/
const sellerDeliver = async (caller_address, gig_id) => {
  // SYNC NONCE
  let caller = new Account(new Address(caller_address));
  await caller.sync(proxyProvider);
  // LOAD PAYLOAD
  let payload_builder = new ContractCallPayloadBuilder();
  payload_builder.setFunction(new ContractFunction("deliver")); // function
  payload_builder.addArg(new U64Value(new BigNumber(gig_id))); // gig-id
  // BUIDL
  let payload = payload_builder.build();
  // MAKE TRANSACTION
  let tx = new Transaction({
    receiver: smartContractAddress,
    nonce: caller.nonce,
    gasLimit: new GasLimit(50000000),
    data: payload,
  });
  // SEND IT
  let reply = await hwWalletP.sendTransaction(tx);
  // Get the transaction object first
  await reply.awaitExecuted(proxyProvider);
  // Then wait for it to get executed
  return reply;
  // return the transaction
};

/* 
@param caller_address: String
@param gig_id: number
*/
const sellerClaim = async (caller_address, gig_id) => {
  // SYNC NONCE
  let caller = new Account(new Address(caller_address));
  await caller.sync(proxyProvider);
  // LOAD PAYLOAD
  let payload_builder = new ContractCallPayloadBuilder();
  payload_builder.setFunction(new ContractFunction("claim")); // function
  payload_builder.addArg(new U64Value(new BigNumber(gig_id))); // gig-id
  // BUIDL
  let payload = payload_builder.build();
  // MAKE TRANSACTION
  let tx = new Transaction({
    receiver: smartContractAddress,
    nonce: caller.nonce,
    gasLimit: new GasLimit(50000000),
    data: payload,
  });
  // SEND IT
  let reply = await hwWalletP.sendTransaction(tx);
  // Get the transaction object first
  await reply.awaitExecuted(proxyProvider);
  // Then wait for it to get executed
  return reply;
  // return the transaction
};

/* 
@param caller_address: String
@param gig_id: number
@param seller_address: String
@param payment: String
*/
const buyerOrder = async (caller_address, gig_id, seller_address, payment) => {
  // SYNC NONCE
  let caller = new Account(new Address(caller_address));
  await caller.sync(proxyProvider);
  // LOAD PAYLOAD
  let payload_builder = new ContractCallPayloadBuilder();
  payload_builder.setFunction(new ContractFunction("order")); // function
  payload_builder.addArg(new U64Value(new BigNumber(gig_id))); // gig-id
  payload_builder.addArg(new AddressValue(new Address(seller_address))); // seller_address
  // BUIDL
  let payload = payload_builder.build();
  // MAKE TRANSACTION
  let tx = new Transaction({
    value: new Balance(payment * 1000000000000000000),
    receiver: smartContractAddress,
    nonce: caller.nonce,
    gasLimit: new GasLimit(50000000),
    data: payload,
  });
  // SEND IT
  let reply = await hwWalletP.sendTransaction(tx);
  // Get the transaction object first
  await reply.awaitExecuted(proxyProvider);
  // Then wait for it to get executed
  return reply;
  // return the transaction
};

/* 
@param caller_address: String
@param gig_id: number
@param seller_address: String
*/
const buyerRefund = async (caller_address, gig_id, seller_address) => {
  // SYNC NONCE
  let caller = new Account(new Address(caller_address));
  await caller.sync(proxyProvider);
  // LOAD PAYLOAD
  let payload_builder = new ContractCallPayloadBuilder();
  payload_builder.setFunction(new ContractFunction("refund")); // function
  payload_builder.addArg(new U64Value(new BigNumber(gig_id))); // gig-id
  payload_builder.addArg(new AddressValue(new Address(seller_address))); // seller_address
  // BUIDL
  let payload = payload_builder.build();
  // MAKE TRANSACTION
  let tx = new Transaction({
    receiver: smartContractAddress,
    nonce: caller.nonce,
    gasLimit: new GasLimit(50000000),
    data: payload,
  });
  // SEND IT
  let reply = await hwWalletP.sendTransaction(tx);
  // Get the transaction object first
  await reply.awaitExecuted(proxyProvider);
  // Then wait for it to get executed
  return reply;
  // return the transaction
};

/* 
@param caller_address: String
@param gig_id: number
@param seller_address: String
*/
const buyerDispute = async (caller_address, gig_id, seller_address) => {
  // SYNC NONCE
  let caller = new Account(new Address(caller_address));
  await caller.sync(proxyProvider);
  // LOAD PAYLOAD
  let payload_builder = new ContractCallPayloadBuilder();
  payload_builder.setFunction(new ContractFunction("dispute")); // function
  payload_builder.addArg(new U64Value(new BigNumber(gig_id))); // gig-id
  payload_builder.addArg(new AddressValue(new Address(seller_address))); // seller_address
  // BUIDL
  let payload = payload_builder.build();
  // MAKE TRANSACTION
  let tx = new Transaction({
    receiver: smartContractAddress,
    nonce: caller.nonce,
    gasLimit: new GasLimit(50000000),
    data: payload,
  });
  // SEND IT
  let reply = await hwWalletP.sendTransaction(tx);
  // Get the transaction object first
  await reply.awaitExecuted(proxyProvider);
  // Then wait for it to get executed
  return reply;
  // return the transaction
};

/* 
@param caller_address: String
@param gig_id: number
@param seller_address: String
*/
const buyerAccept = async (caller_address, gig_id, seller_address) => {
  // SYNC NONCE
  let caller = new Account(new Address(caller_address));
  await caller.sync(proxyProvider);
  // LOAD PAYLOAD
  let payload_builder = new ContractCallPayloadBuilder();
  payload_builder.setFunction(new ContractFunction("accept")); // function
  payload_builder.addArg(new U64Value(new BigNumber(gig_id))); // gig-id
  payload_builder.addArg(new AddressValue(new Address(seller_address))); // seller_address
  // BUIDL
  let payload = payload_builder.build();
  // MAKE TRANSACTION
  let tx = new Transaction({
    receiver: smartContractAddress,
    nonce: caller.nonce,
    gasLimit: new GasLimit(50000000),
    data: payload,
  });
  // SEND IT
  let reply = await hwWalletP.sendTransaction(tx);
  // Get the transaction object first
  await reply.awaitExecuted(proxyProvider);
  // Then wait for it to get executed
  return reply;
  // return the transaction
};

export {
  connectLedger,
  sellerList,
  sellerUnlist,
  sellerClaim,
  sellerDeliver,
  buyerAccept,
  buyerDispute,
  buyerOrder,
  buyerRefund,
};
