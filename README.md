# Zero Network
[![CI Check](https://github.com/zero-network/zero/actions/workflows/ci.yml/badge.svg)](https://github.com/zero-network/zero/actions/workflows/ci.yml) [![Repository](https://img.shields.io/badge/github-zero-blueviolet?logo=github)](https://github.com/zero-network/zero) [![GitHub license](https://img.shields.io/badge/license-GPL3%2FApache2-blue)](#LICENSE)  
Zero Network is a completely anonymous blockchain. This allows us the anonymous transfers and privacy preserving smart contracts. These functionalities are designed relying on only the cryptographic hardness assumptions instead `L2 technologies`, `TEE` and `centralized security assumption`.

## Abstract
Zero Network is the `substrate-based` blockchain and that transaction information are totally hided with cryptography. This is going to be deployed as [`Polkadot`](https://polkadot.network/) parachain. We are also implementing `plonk` which has compatible with `no_std` and [`parity-scale-codec`](https://github.com/paritytech/parity-scale-codec) and, is optimized by assembly and latest algorithm. We are going to support following functionalities.

- Confidential Transfers
- Privacy Preserving Smart Contracts
- Create Transaction Validity Proof
- Client Wallet

## Directory Structure
- node  
The anonymous blockchain implementation.
- pallets  
The `pallet` implementations which are used in blockchain.
- snarks  
The optimized `plonk` research and development.

## Test
```
makers test
```
