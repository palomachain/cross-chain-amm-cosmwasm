# Cross-Chain AMM

This is a template (factory) modeled after the Uniswap approach that allows anyone to create and deploy new
cross-chain AMM pools on any chains which have a Compass contract deployed.

## Overview of pool logic

### Pool participants

Liquidity providers: liquidity providers join the pool by providing liquidity either single-sided or double-sided. They
earn fees from liquidity takers.

- single-sided LP: LPs initially deposit tokens only on one chain, say `chain1.token1`.
  See [Single-sided liquidity adding](#single-sided-liquidity-adding)
  below
- double-sided LP: LPs whose initial deposits contain both `chain1.token1` and `chain2.token2`. Since they can’t deposit
  on both chains in one transaction, the logic is the same as [Single-sided liquidity adding](#single-sided-liquidity-adding).

Liquidity takers: users who initially own tokens on one chain and would like to move the liquidity to another chain.
They pay fees to LPs for the liquidity transfer.

### Assumptions

1. Each pair consists of `token1` on `chain1` and `token2` on `chain2` will therefore have a contract on two chains plus a
   Cosmwasm contract on Paloma. The Paloma Cosmwasm contract controls the pool logic.
2. Liquidity providers (LPs) can provide liquidity by depositing `token1`, `token2`, or both.
3. LPs who want to withdraw their position may not be able to withdraw the token on the same chain that they made their
   initial deposit.
4. Liquidity takers can swap `token1` to `token2` or `token2` to `token1` as long as they have a wallet on `chain1` and `chain2`
5. Liquidity providers and takers need a valid wallet addresses on `chain1` and `chain2`.

### Single-sided liquidity adding

By default, the AMMs let liquidity providers add tokens double-sided---they need to deposit `x chain1.token1` and 
`y chain2.token2` in order to add liquidity, where `x / y` is determined by the pool’s token balance. In addition, we will
have a single-sided liquidity adding option. If users only have tokens on one chain, they can use this option to add
liquidity.

#### How single-sided liquidity adding works:

- each pool has two wallets: `chain1.wallet1` which contains all `chain1.token1` reserve; `chain2.wallet2` which contains all
  `chain2.token2` reserve
- $X$ amount of swappable `chain1.token1` in `chain1.wallet1`, $Y$ amount of swappable `chain2.token2` in `chain2.wallet2` (some
  of the tokens might be temporarily un-swappable before they are matched with tokens on the other chain)
- if user Alice adds $X_0$ `chain1.token1` with no `chain2.token2`, her liquidity is not immediately available in the pool---
  equivalently, she doesn’t get LP tokens right away. Rather, her liquidity waits in the queue. In this case the amount
  of swappable `chain1.token1` is still $X$.
- the user Bob adds $Y_0$ `chain2.token2` with no `chain1.token1`, his liquidity will be matched with Alice’s in the
  following way:
    - if $X_0 < Y_0$, $X / Y$: Alice’s liquidity is completely added into the pool while Bob’s $X_0$ $Y / X$ added
      to his remaining $Y_0 - X_0$, and $Y / X$ is still in the queue.
    - if $X_0 > Y_0$, $X / Y$: Bob’s liquidity is completely added into the pool while Alice’s $Y_0$ $X / Y$ added
      to her remaining $X_0 - Y_0$, and $X / Y$ is still in the queue.
- Once the LP’s liquidity is added, they receive LP tokens, which means their liquidity now is in a mixed state,
  containing both `chain1.token1` and `chain2.token2`.
- If an LP adds liquidity single-sided, they might wait in the queue in-definitely---this is a drawback to the
  single-sided LP addition.
- While their liquidity waiting in queue, they can withdraw them in the original token.
- Once their liquidity is matched and added to the pool, their liquidity is represented by the LP tokens they hold.
- At that point if they withdraw, they will end up with a combination of tokens, determined by their LP shares.

### A sample process

A series of events taking place in a Paloma cross chain AMM factory pool in chronological order:

1. A Ethereum.WETH <> Polygon.WETH pool is set up and deployed with a number of pool parameters {fees, etc}
2. LP Alice deposits 10 Polygon.WETH in the pool → since there is not Ethereum.WETH, her tokens are un-swappable for
   swapping at this moment
3. LP Bob deposits 10 Ethereum.WETH in the pool → this liquidity addition updates the pool’s supply and demand and now
   there’s an exchange rate, say 0.95, namely 1 Polygon.WETH = 0.95 Ethereum.WETH.
    1. Alice and Bob’s LP shares are updated.
    2. Automatically their liquidity becomes mixed:
        1. Alice owns (10 Polygon.WETH + 10 Ethereum.WETH) * 0.95 / (1+ 0.95)
        2. Bob owns (10 Polygon.WETH + 10 Ethereum.WETH) * 1 / (1+ 0.95)
    3. This means if they’d like to remove liquidity they both will end up with two tokens
4. LP Chris has both Ethereum.WETH and Polygon.WETH and would like to add liquidity on both sides. At the exchange rate
   in the moment, he deposits 20 Polygon.WETH and 19 Ethereum.WETH. Alice, Bob and Chris’ LP shares are updated
5. Now the pool contains 30 Polygon.WETH and 29 Ethereum.WETH, with exchange rate at 0.95
6. Liquidity taker Derek puts 0.5 Polygon.WETH in the pool to swap for Ethereum.WETH. He should get around 0.5 * 0.95 =
   0.475 Ethereum.WETH. After accounting for fees, he gets 0.45 Ethereum.WETH and the rest of his liquidity consists of
   0.01 Polygon.WETH and 0.01 Ethereum.WETH, which becomes the rewards for LPs
7. The exchange rate gets updated. Now at 0.93.
8. Alice decides to remove liquidity, she claims reward and gets back Ethereum.WETH and Polygon.WETH at the exchange
   rate of 0.93
