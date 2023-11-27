# Crisp: Functional Overview

Crisp provides the following functionality:
- [Swapping](#swapping)
- [Liquidity](#liquidity) provision (LP) with leverage
- [Lending](#lending)

## Swapping

As Crisp is a DEX, it allows swapping your tokens, e.g. USDC to NEAR. It is very much industry standard:
- choose the token you want to pay with
- choose the token you want to buy
- enter the amount
- see the price and check the slippage
- sign the transaction

It also implements multi-hop swap routing where if the pool with chosen tokens A and B doesn't exist then a route between token A and token B is found among existing pools.

## Liquidity

Liquidity for swaps is provided in liquidity pools for each token pair and work as a concentrated liquidity AMM. Liquidity provider (LP) creates a position in a chosen price range and earns a portion of swap fees proportional to their liquidity share at the current price level, as long as the price is within that price range.

In addition to that, LPs can apply [leverage](#leverage) on their liquidity positions. In this case, a liquidity position acts as collateral to borrow more tokens and multiply the position size.

LP can:
- create a liquidity position in a chosen pool
- choose their price range, position size, leverage
- see their position amounts, revenue generated, liquidation prices
- add liquidity to the position
- remove liquidity from the position
- add leverage to an existing unleveraged position
- unleverage a leveraged position
- close the position

### Leverage

Because every liquidity position has an easily and reliably determined value at any moment, every liquidity position can act as collateral for a loan, which is used to multiply the position size. The amount of leverage possible is limited by:
- the position price range
- $LTV_{max}$

With $LTV_{max}$ (loan-to-value) at its current 0.8, which means that every loan must be overcollateralized by at least 125%, the highest possible leverage gets close to 5x.

Leveraged positions hence provide up to 5x more liquidity than the LP's initial liquidity by employing the capital deposited by lenders. This generates them proportionally more revenue while also exposes the LP to higher volatility risks and potential [liquidation](#liquidation).

### Liquidation

Leveraged positions can be liquidated if the price goes far enough in either direction. As leverage is provided by borrowing fixed amounts of both tokens, collateralized by a position subject to changes of token amounts (known as "impermanent loss"), every leveraged position has two liquidation prices: below and above its price range. The higher the leverage of a position, the closer its liquidation prices will be to its active price range.

Every user can be a liquidator, finding undercollateralized positions and liquidating them. To do that they repay the outstanding loan in both tokens and acquire the position, which can then be immediately closed to take the profits or kept (it is now unleveraged, as the loan has been repaid).

Keepers are provided as protection against liquidations, they monitor user positions and close them as soon as a given price condition is met to avoid liquidation.

## Lending

Lending allows users to deposit their tokens and earn interest risk-free. These deposits fill up the reserves that are used by the leveraged liquidity positions. LPs act as borrowers when they use leverage and pay lenders the interest for using their capital.
