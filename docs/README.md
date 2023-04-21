# near-amm
How to use this contract via near-cli

Set env variables:
```
export CONTRACT_ID=yourdevaccount.testnet
export NEAR_ENV=testnet
export TOKEN1=token1-ft.testnet
export TOKEN2=token2-ft.testnet
export USER_ID=someuser.testnet
```
Build contract:
```
./build.sh
```
Deploy contract:
```
./deploy.sh
```
Initialize contract:
```
near call $CONTRACT_ID new '{"owner_id": "'$CONTRACT_ID'"}" --accountId $CONTRACT_ID
```
Create pool:
```
near call $CONTRACT_ID create_pool '{"token1": "'$TOKEN1'", "token2": "'$TOKEN2'", "initial_price": 100.0, "protocol_fee": 10, "rewards": 20}' --accountId $CONTRACT_ID
```
Return - pool_id:
```
0
```
View a specific pool:
```
near view $CONTRACT_ID get_pool '{"pool_id": 0}'
```
Return - pool information:
```
[
  {
    token0: 'near-ft.testnet',
    token1: 'usn-ft.testnet',
    liquidity: 26528334.515969425,
    sqrt_price: 10,
    tick: 46054,
    positions: [
      {
        id: 0,
        owner_id: 'liquidity-provider.testnet',
        liquidity: 26528334.515969425,
        token0_real_liquidity: 123456,
        token1_real_liquidity: 13613466.3557227,
        tick_lower_bound_price: 45000,
        tick_upper_bound_price: 47007,
        sqrt_lower_bound_price: 9.48666859725659,
        sqrt_upper_bound_price: 10.487483440671777,
        is_active: false
      }
    ],
    protocol_fee: 0,
    rewards: 0
  }
]

```
View all the pools:
```
near view $CONTRACT_ID get_pools '{}'
```
Returns list of pools.

View balance of a specific account:
```
near view $CONTRACT_ID get_balance '{"account_id": "'$USER_ID'", "token": "'$TOKEN1'"}'
```
Returns balance:
```
'near-ft.testnet: 1000000000000'
```
View balance of a specific account (all the tokens):
```
near view $CONTRACT_ID get_balance_all_tokens '{"account_id": "'$USER_ID'"}'
```
Returns string containing balances:
```
'near-ft.testnet: 1000000000000, usn-ft.testnet: 100000,'
```
Deposit tokens (We have to interact with fungible token smart-contract. You should already have tokens):
```
near call $TOKEN1 storage_deposit '{"account_id": "'$CONTRACT_ID'"}' --accountId $USER_ID --amount 0.0125
near call $TOKEN1 ft_transfer_call '{"receiver_id": "'$CONTRACT_ID'", "amount": "10000", "msg": ""}' --accountId $USER_ID --depositYocto 1
```
Withdraw tokens:
```
near call $CONTRACT_ID withdraw '{"token": "'$TOKEN1'", "amount": "12345"}' --accountId $USER_ID
```
Get return (how much tokens I get if I send `amount_in` tokens to the pool):
```
near view $CONTRACT_ID get_return '{"pool_id": 0, "token_in": "'$TOKEN1'", "amount_in": "9876"}'
```
Returns amount I will get:
```
"1342"
```
Get expense (how much tokens should I send to get `amount_out` tokens from the pool):
```
near view $CONTRACT_ID get_expense '{"pool_id": 0, "token_out": "'$TOKEN1'", "amount_out": "2345"}'
```
Returns amount I will send:
```
"2453"
```
Get pool`s price:
```
near view $CONTRACT_ID get_price '{"pool_id": 0}'
```
Returns float price:
```
99.83752
```
Swap in the pool (If I know how much I want to send):
```
near call $CONTRACT_ID swap '{"pool_id": 0, "token_in": "'$TOKEN1'", "amount_in": "1357984", "token_out": "'$TOKEN2'"}' --accountId $USER_ID
```
Returns given amount I get:
```
"13562"
```
Open position (Choose only one token, amount of another token will be calculated automatically):
```
near call $CONTRACT_ID open_position '{"pool_id": 0, token0_liquidity: "100000", "lower_bound_price": 90.0, "upper_bound_price": 110.0}' --accountId $USER_ID
```
Returns position id:
```
0
```
Close position:
```
near call $CONTRACT_ID close_position '{"pool_id": 0, "position_id": 12}' --accountId $USER_ID
```
Returns bool (true if positions was actually closed and false otherwise)
```
true
```
Add tokens to the position:
```
near call $CONTRACT_ID add_liquidity '{"pool_id": 0, "position_id": 12, "token0_liquidity": "1000"}' --accountId $USER_ID
```
Returns bool (true if liquidity was actually added to the position and false otherwise)

Add tokens to the position:
```
near call $CONTRACT_ID remove_liquidity '{"pool_id": 0, "position_id": 12, "token0_liquidity": "1000"}' --accountId $USER_ID
```
Returns bool (true if liquidity was actually removed from the position and false otherwise)

Swap on exchange using multihope:
```
near call $CONTRACT_ID swap_multihope '{"token_in": "'$TOKEN1'", "amount_in": "100000", "token_out": "'$TOKEN2'"}' --accountId $USER_ID --gas 300000000000000
```
Returns given amount I get:
```
"13562"
```

Create reserve:
```
near call $CONTRACT_ID create_reserve '{"reserve_token": "'$TOKEN1'"}' --accountId $CONTRACT_ID
```

Create deposit:
```
near call $CONTRACT_ID create_deposit '{"asset": "'$TOKEN1'", "amount": "100000"}' --accountId $USER_ID
```

Close deposit:
```
near call $CONTRACT_ID close_deposit '{"deposit_id": "0"}' --accountId $USER_ID
```

Refresh deposits growth:
```
near call $CONTRACT_ID refresh_deposits_growth '{}' --accountId $CONTRACT_ID
```

Take deposit growth:
```
near call $CONTRACT_ID take_deposit_growth '{"deposit_id": "0", "amount": "100"}' --accountId $USER_ID
```
Returns amount taken
```
50
```

Get deposits for a certain account:
```
near call $CONTRACT_ID get_account_deposits '{"account_id": "'$USER_ID'"}' --accountId $USER_ID
```
Returns all the deposits made from USER_ID account

Supply collateral and borrow without leverage:
```
near call $CONTRACT_ID supply_collateral_and_borrow_simple '{"pool_id": 0, "position_id": 0}' --accountId $USER_ID
```
Returns the amount borrowed
```
567
```

Supply collateral and borrow with leverage:
```
near call $CONTRACT_ID supply_collateral_and_borrow_leveraged '{"pool_id": 0, "position_id": 0, "leverage": 2}' --accountId $USER_ID
```

Return collateral and repay:
```
near call $CONTRACT_ID return_collateral_and_repay '{"borrow_id": 0}' --accountId $USER_ID
```

Get borrows which could be liquidated:
```
near call $CONTRACT_ID get_liquidation_list '{}' --accountId $USER_ID
```
Return vector of ids:
```
[1, 3, 5, 6]
```

Get borrow`s health factor:
```
near view $CONTRACT_ID get_borrow_health_factor '{"borrow_id": 0}'
```
Returns float
```
1.25
```

Liquidate a borrow:
```
near call $CONTRACT_ID liquidate '{"borrow_id": 0}' --accountId $USER_ID
```

How to make a deposit for lending:
1. call method ```create_deposit``` with arguments ```asset = example.near```, ```amount = 10```
2. as a default ARP for deposits is 5%
3. to close deposit use method ```close_deposit``` with argument ```deposit_id = 1```

How to borrow assets:
1. to calculate how much we can take from position use formula ```p * x + y``` (this is for second token)
2. to borrow use method ```supply_collateral_and_borrow_simple``` with arguments ```position_id = 2``` ```pool_id = 1```
3. to check the health factor of a borrow use method ```get_borrow_health_factor``` with argument ```borrow_id = 0```
4. to return borrow use ```return_collateral_and_repay``` with argument ```borrow_id = 0```

How to borrow assets with leverage:
1. to calculate how much we can take from position use formula ```p * x + y``` (this is for second token)
2. to borrow use method ```supply_collateral_and_borrow_leveraged``` with arguments ```position_id = 2``` ```pool_id = 1``` ```leverage = 4```
3. to check the health factor of a borrow use method ```get_borrow_health_factor``` with argument ```borrow_id = 0```
4. to return borrow use ```return_collateral_and_repay``` with argument ```borrow_id = 0```
