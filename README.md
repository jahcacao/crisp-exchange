# Crisp

[Crisp](https://crisp.exchange/) is an open-source structured liquidity protocol on the NEAR blockchain.

This repository contains the Rust smart contract.

There is also a [frontend repository](https://github.com/Mycelium-Lab/crisp-frontend).

## How it works

The smart contract implements an advanced DEX where you can:
- trade with existing liquidity
- place concentrated liquidity positions in a chosen price range to earn trading fees
- leverage your positions up to 5x
- lend your tokens for others to borrow for leverage and earn interest
- liquidate underwater leveraged positions to earn a premium

## Requirements
- NEAR 3.4.2
- Rust 1.64.0

## Setup

1. Install near-cli using instructions found [here](https://docs.near.org/tools/near-cli). 

2. Install rust using [this](https://www.rust-lang.org/tools/install).

3. Clone the repository and open it.

## Deploy

`./deploy.sh` will build and deploy the smart contract

## Usage

Interactions with the contract are documented [here](https://github.com/Mycelium-Lab/crisp-exchange/tree/main/docs).

## Tests

Run `cargo test` to run Rust tests

# License

[Apache 2.0](https://choosealicense.com/licenses/apache-2.0/)
