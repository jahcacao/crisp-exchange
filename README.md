# near-amm
How to use this contract via near-cli

Set env variables
```
export CONTRACT_ID=yourdevaccount.testnet
export NEAR_ENV=testnet
```
Build contract
```
./build.sh
```
Deploy contract
```
./deploy.sh
```
Initialize contract
```
near call $CONTRACT_ID new '{"owner_id": "'$CONTRACT_ID'"}" --accountId $CONTRACT_ID
```
Create pool
```
near call $CONTRACT_ID create_pool '{""}' --accountId $CONTRACT_ID
```