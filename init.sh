#!/bin/sh

if [ $# -ne 2 ]; then
    echo "Usage: $0 <contract_account> <master_account>"
    exit 1
fi

CONTRACT=$1
MASTER_ACCOUNT=$2

echo ">> Preparing the account"
near delete $CONTRACT $MASTER_ACCOUNT
near create-account $CONTRACT --masterAccount $MASTER_ACCOUNT

set -e
export NEAR_ENV=testnet

echo ">> Deploying the contract"
./deploy.sh

echo ">> Initializing the contract"
near call $CONTRACT new '{"owner_id": "'$MASTER_ACCOUNT'"}' --accountId $MASTER_ACCOUNT

echo ">> Creating the pools"
near call $CONTRACT create_pool '{"token1": "usdn.testnet", "token2": "usdc.fakes.testnet", "initial_price": 0.000000000001, "protocol_fee": 1, "rewards": 1}' --accountId $CONTRACT
near call $CONTRACT create_pool '{"token1": "wrap.testnet", "token2": "usdc.fakes.testnet", "initial_price": 0.0000000000000001, "protocol_fee": 1, "rewards": 1}' --accountId $CONTRACT

echo ">> Pools created:"
near view $CONTRACT get_pools '{}'

echo ">> Creating reserves"
near call $CONTRACT create_reserve '{"reserve_token": "'usdn.testnet'"}' --accountId $CONTRACT
near call $CONTRACT create_reserve '{"reserve_token": "usdc.fakes.testnet"}' --accountId $CONTRACT
near call $CONTRACT create_reserve '{"reserve_token": "wrap.testnet"}' --accountId $CONTRACT
