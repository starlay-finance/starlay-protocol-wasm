import { ReturnNumber } from '@727-ventures/typechain-types'
import type { ApiPromise } from '@polkadot/api'
import type { KeyringPair } from '@polkadot/keyring/types'
import { BN } from '@polkadot/util'
import {
  deployController,
  deployDefaultInterestRateModel,
  deployPoolFromAsset,
  deployPriceOracle,
  deployPSP22Token,
} from '../scripts/helper/deploy_helper'
import { hexToUtf8, ZERO_ADDRESS } from '../scripts/helper/utils'
import { ONE_ETHER } from '../scripts/tokens'
import Controller from '../types/contracts/controller'
import DefaultInterestRateModel from '../types/contracts/default_interest_rate_model'
import Pool from '../types/contracts/pool'
import PSP22Token from '../types/contracts/psp22_token'
import { Mint, Redeem } from '../types/event-types/pool'
import { Transfer } from '../types/event-types/psp22_token'
import { expectToEmit, shouldNotRevert } from './testHelpers'

const TOKENS = ['dai', 'usdc', 'usdt'] as const
const METADATAS: {
  [key in (typeof TOKENS)[number]]: {
    name: string
    symbol: string
    decimals: number
  }
} = {
  dai: {
    name: 'Dai Stablecoin',
    symbol: 'DAI',
    decimals: 18,
  },
  usdc: {
    name: 'USD Coin',
    symbol: 'USDC',
    decimals: 6,
  },
  usdt: {
    name: 'USD Tether',
    symbol: 'USDT',
    decimals: 6,
  },
} as const
const preparePoolWithMockToken = async ({
  api,
  metadata,
  controller,
  rateModel,
  manager,
}: {
  api: ApiPromise
  metadata: {
    name: string
    symbol: string
    decimals: number
  }
  controller: Controller
  rateModel: DefaultInterestRateModel
  manager: KeyringPair
}): Promise<{
  token: PSP22Token
  pool: Pool
}> => {
  const token = await deployPSP22Token({
    api,
    signer: manager,
    args: [
      0,
      metadata.name as unknown as string[],
      metadata.symbol as unknown as string[],
      metadata.decimals,
    ],
  })

  const pool = await deployPoolFromAsset({
    api,
    signer: manager,
    args: [
      token.address,
      controller.address,
      rateModel.address,
      [ONE_ETHER.toString()],
    ],
    token,
  })

  return { token, pool }
}

type Pools = {
  [key in (typeof TOKENS)[number]]: {
    token: PSP22Token
    pool: Pool
  }
}
const preparePoolsWithPreparedTokens = async ({
  api,
  controller,
  rateModel,
  manager,
}: {
  api: ApiPromise
  controller: Controller
  rateModel: DefaultInterestRateModel
  manager: KeyringPair
}): Promise<Pools> => {
  const dai = await preparePoolWithMockToken({
    api,
    controller,
    rateModel,
    manager: manager,
    metadata: METADATAS.dai,
  })
  const usdc = await preparePoolWithMockToken({
    api,
    controller,
    rateModel,
    manager: manager,
    metadata: METADATAS.usdc,
  })
  const usdt = await preparePoolWithMockToken({
    api,
    controller,
    rateModel,
    manager: manager,
    metadata: METADATAS.usdt,
  })
  return { dai, usdc, usdt }
}

const pow10 = (exponent: number) => new BN(10).pow(new BN(exponent))
const mantissa = () => pow10(18)
const to_dec18 = (value: number) => new BN(value).mul(pow10(18))
const to_dec6 = (value: number) => new BN(value).mul(pow10(6))

describe('Pool spec', () => {
  const setup = async () => {
    const { api, alice: deployer, bob, charlie, django } = globalThis.setup

    const controller = await deployController({
      api,
      signer: deployer,
      args: [deployer.address],
    })
    const priceOracle = await deployPriceOracle({
      api,
      signer: deployer,
      args: [],
    })

    // temp: declare params for rate_model
    const toParam = (m: BN) => [m.toString()]
    const rateModelArg = new BN(100).mul(ONE_ETHER)
    const rateModel = await deployDefaultInterestRateModel({
      api,
      signer: deployer,
      args: [
        toParam(rateModelArg),
        toParam(rateModelArg),
        toParam(rateModelArg),
        toParam(rateModelArg),
      ],
    })

    const pools = await preparePoolsWithPreparedTokens({
      api,
      controller,
      rateModel,
      manager: deployer,
    })

    const users = [bob, charlie, django]

    // initialize
    await controller.tx.setPriceOracle(priceOracle.address)
    await controller.tx.setCloseFactorMantissa(toParam(ONE_ETHER))
    //// for pool
    for (const sym of [pools.dai, pools.usdc, pools.usdt]) {
      await priceOracle.tx.setFixedPrice(sym.token.address, ONE_ETHER)
      await controller.tx.supportMarketWithCollateralFactorMantissa(
        sym.pool.address,
        toParam(ONE_ETHER.mul(new BN(90)).div(new BN(100))),
      )
    }

    return {
      api,
      deployer,
      pools,
      rateModel,
      controller,
      priceOracle,
      users,
    }
  }

  it('instantiate', async () => {
    const { pools, controller } = await setup()
    const { pool, token } = pools.dai
    expect(pool.address).not.toBe(ZERO_ADDRESS)
    expect((await pool.query.underlying()).value.ok).toEqual(token.address)
    expect((await pool.query.controller()).value.ok).toEqual(controller.address)
    expect(hexToUtf8((await pool.query.tokenName()).value.ok)).toEqual(
      'Starlay Dai Stablecoin',
    )
    expect(hexToUtf8((await pool.query.tokenSymbol()).value.ok)).toEqual('sDAI')
    expect((await pool.query.tokenDecimals()).value.ok).toEqual(18)
  })

  describe('.mint', () => {
    let deployer: KeyringPair
    let token: PSP22Token
    let pool: Pool

    beforeAll(async () => {
      ;({
        deployer,
        pools: {
          dai: { token, pool },
        },
      } = await setup())
    })

    const balance = 10_000
    it('preparations', async () => {
      await shouldNotRevert(token, 'mint', [deployer.address, balance])
      expect(
        (await token.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toBe(balance)
    })

    it('execute', async () => {
      const depositAmount = 3_000
      const mintAmount = depositAmount
      await shouldNotRevert(token, 'approve', [pool.address, depositAmount])
      const { events } = await shouldNotRevert(pool, 'mint', [depositAmount])

      expect(
        (await token.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toBe(balance - depositAmount)
      expect(
        (await token.query.balanceOf(pool.address)).value.ok.toNumber(),
      ).toBe(depositAmount)
      expect(
        (await pool.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toBe(mintAmount)

      expect(events).toHaveLength(2)
      expectToEmit<Transfer>(events[0], 'Transfer', {
        from: null,
        to: deployer.address,
        value: depositAmount,
      })
      expectToEmit<Mint>(events[1], 'Mint', {
        minter: deployer.address,
        mintAmount,
        mintTokens: depositAmount,
      })
    })
  })

  describe('.redeem', () => {
    let deployer: KeyringPair
    let token: PSP22Token
    let pool: Pool

    beforeAll(async () => {
      ;({
        deployer,
        pools: {
          dai: { token, pool },
        },
      } = await setup())
    })

    const deposited = 10_000
    const minted = deposited
    it('preparations', async () => {
      await shouldNotRevert(token, 'mint', [deployer.address, deposited])

      await shouldNotRevert(token, 'approve', [pool.address, deposited])
      await shouldNotRevert(pool, 'mint', [deposited])
      expect(
        (await pool.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(minted)
      expect(
        (await pool.query.exchangeRateCurrent()).value.ok.ok.toString(),
      ).toBe(ONE_ETHER.toString())
    })

    it('execute', async () => {
      const redeemAmount = 3_000
      const { events } = await shouldNotRevert(pool, 'redeem', [redeemAmount])

      expect(
        (await token.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(redeemAmount)
      expect(
        (await token.query.balanceOf(pool.address)).value.ok.toNumber(),
      ).toEqual(deposited - redeemAmount)
      expect(
        (await pool.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(minted - redeemAmount)

      expect(events).toHaveLength(2)
      expectToEmit<Transfer>(events[0], 'Transfer', {
        from: deployer.address,
        to: null,
        value: redeemAmount,
      })
      expectToEmit<Redeem>(events[1], 'Redeem', {
        redeemer: deployer.address,
        redeemAmount,
        redeemTokens: redeemAmount,
      })
    })
  })

  describe('.redeem_underlying', () => {
    let deployer: KeyringPair
    let token: PSP22Token
    let pool: Pool

    beforeAll(async () => {
      ;({
        deployer,
        pools: {
          dai: { token, pool },
        },
      } = await setup())
    })

    const deposited = 10_000
    const minted = deposited
    it('setup', async () => {
      await shouldNotRevert(token, 'mint', [deployer.address, deposited])

      await shouldNotRevert(token, 'approve', [pool.address, deposited])
      await shouldNotRevert(pool, 'mint', [deposited])
      expect(
        (await pool.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toBe(minted)
      expect(
        (await pool.query.exchangeRateCurrent()).value.ok.ok.toString(),
      ).toBe(ONE_ETHER.toString())
    })
    it('execute', async () => {
      const redeemAmount = 3_000
      const { events } = await shouldNotRevert(pool, 'redeemUnderlying', [
        redeemAmount,
      ])

      expect(
        (await token.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toBe(redeemAmount)
      expect(
        (await token.query.balanceOf(pool.address)).value.ok.toNumber(),
      ).toBe(deposited - redeemAmount)
      expect(
        (await pool.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toBe(minted - redeemAmount)

      expect(events).toHaveLength(2)
      expectToEmit<Transfer>(events[0], 'Transfer', {
        from: deployer.address,
        to: null,
        value: redeemAmount,
      })
      expectToEmit<Redeem>(events[1], 'Redeem', {
        redeemer: deployer.address,
        redeemAmount,
        redeemTokens: redeemAmount,
      })
    })
  })

  describe('.redeem (fail case)', () => {
    it('when no cash in pool', async () => {
      const {
        deployer,
        pools: { usdc, usdt },
      } = await setup()

      for await (const { token, pool } of [usdc, usdt]) {
        await shouldNotRevert(token, 'mint', [
          deployer.address,
          to_dec6(10_000),
        ])
        await shouldNotRevert(token, 'approve', [pool.address, to_dec6(10_000)])
        await shouldNotRevert(pool, 'mint', [to_dec6(10_000)])
      }

      const {
        value: { ok: cash },
      } = await usdc.pool.query.getCashPrior()
      const { value } = await usdc.pool
        .withSigner(deployer)
        .query.redeemUnderlying(cash.toNumber() + 1)
      expect(value.ok.err).toHaveProperty('redeemTransferOutNotPossible')
    })
  })

  describe('.borrow', () => {
    let deployer: KeyringPair
    let token: PSP22Token
    let pool: Pool
    let users: KeyringPair[]

    beforeAll(async () => {
      ;({
        deployer,
        users,
        pools: {
          dai: { token, pool },
        },
      } = await setup())
    })

    it('preparations', async () => {
      const [user1, user2] = users
      await token.withSigner(deployer).tx.mint(user1.address, 5_000)
      await token.withSigner(deployer).tx.mint(user2.address, 5_000)
      await token.withSigner(user1).tx.approve(pool.address, 5_000)
      await pool.withSigner(user1).tx.mint(5_000)
      await token.withSigner(user2).tx.approve(pool.address, 5_000)
      await pool.withSigner(user2).tx.mint(5_000)
      expect((await pool.query.totalSupply()).value.ok.toNumber()).toEqual(
        10_000,
      )
      expect(
        (await pool.query.balanceOf(user1.address)).value.ok.toNumber(),
      ).toEqual(5_000)
      expect(
        (await pool.query.balanceOf(user2.address)).value.ok.toNumber(),
      ).toEqual(5_000)
    })

    it('execute', async () => {
      const [user1, user2] = users
      const { events: events1 } = await pool.withSigner(user1).tx.borrow(3_000)

      expect(
        (await token.query.balanceOf(user1.address)).value.ok.toNumber(),
      ).toEqual(3_000)
      expect(
        (await token.query.balanceOf(user2.address)).value.ok.toNumber(),
      ).toEqual(0)
      expect(
        (await token.query.balanceOf(pool.address)).value.ok.toNumber(),
      ).toEqual(7_000)
      const event1 = events1[0]
      expect(event1.name).toEqual('Borrow')
      expect(event1.args.borrower).toEqual(user1.address)
      expect(event1.args.borrowAmount.toNumber()).toEqual(3_000)
      expect(event1.args.accountBorrows.toNumber()).toEqual(3_000)
      expect(event1.args.totalBorrows.toNumber()).toEqual(3_000)

      const { events: events2 } = await pool.withSigner(user2).tx.borrow(2_500)

      expect(
        (await token.query.balanceOf(user1.address)).value.ok.toNumber(),
      ).toEqual(3_000)
      expect(
        (await token.query.balanceOf(user2.address)).value.ok.toNumber(),
      ).toEqual(2_500)
      expect(
        (await token.query.balanceOf(pool.address)).value.ok.toNumber(),
      ).toEqual(4_500)
      const event2 = events2[0]
      expect(event2.name).toEqual('Borrow')
      expect(event2.args.borrower).toEqual(user2.address)
      expect(event2.args.borrowAmount.toNumber()).toEqual(2_500)
      expect(event2.args.accountBorrows.toNumber()).toEqual(2_500)
      expect(event2.args.totalBorrows.toNumber()).toEqual(5_500)
    })
  })

  describe('.borrow (fail case)', () => {
    it('when no cash in pool', async () => {
      const {
        deployer,
        pools: { usdc, usdt },
      } = await setup()

      await usdc.token.tx.mint(deployer.address, to_dec6(5_000))
      await usdc.token.tx.approve(usdc.pool.address, to_dec6(5_000))
      await usdc.pool.tx.mint(to_dec6(5_000))

      const { value } = await usdt.pool.query.borrow(to_dec6(1_000))
      expect(value.ok.err).toStrictEqual({ borrowCashNotAvailable: null })
    })
  })

  describe('.repay_borrow', () => {
    let deployer: KeyringPair
    let pools: Pools
    let users: KeyringPair[]

    beforeAll(async () => {
      ;({ deployer, users, pools } = await setup())
    })

    it('preparations', async () => {
      const { dai, usdc } = pools

      // add liquidity to usdc pool
      await usdc.token.tx.mint(deployer.address, to_dec6(10_000))
      await usdc.token.tx.approve(usdc.pool.address, to_dec6(10_000))
      await usdc.pool.tx.mint(to_dec6(10_000))
      expect(
        BigInt(
          (
            await usdc.pool.query.balanceOf(deployer.address)
          ).value.ok.toString(),
        ).toString(),
      ).toEqual(to_dec6(10_000).toString())

      // mint to dai pool for collateral
      const [user1] = users
      await dai.token.tx.mint(user1.address, to_dec18(20_000))
      await dai.token
        .withSigner(user1)
        .tx.approve(dai.pool.address, to_dec18(20_000))
      await dai.pool.withSigner(user1).tx.mint(to_dec18(20_000))
      expect(
        BigInt(
          (await dai.pool.query.balanceOf(user1.address)).value.ok.toString(),
        ).toString(),
      ).toEqual(to_dec18(20_000).toString())

      // borrow usdc
      await usdc.pool.withSigner(user1).tx.borrow(to_dec6(10_000))
      expect(
        BigInt(
          (await usdc.token.query.balanceOf(user1.address)).value.ok.toString(),
        ).toString(),
      ).toEqual(to_dec6(10_000).toString())
    })

    it('execute', async () => {
      const { token, pool } = pools.usdc
      const [user1] = users
      await token.withSigner(user1).tx.approve(pool.address, to_dec6(4_500))
      const { events } = await pool
        .withSigner(user1)
        .tx.repayBorrow(to_dec6(4_500))

      expect(
        BigInt(
          (await token.query.balanceOf(user1.address)).value.ok.toString(),
        ).toString(),
      ).toEqual(to_dec6(5_500).toString())
      expect(
        BigInt(
          (await token.query.balanceOf(pool.address)).value.ok.toString(),
        ).toString(),
      ).toEqual(to_dec6(4_500).toString())

      const event = events[0]
      expect(event.name).toEqual('RepayBorrow')
      expect(event.args.payer).toEqual(user1.address)
      expect(event.args.borrower).toEqual(user1.address)
      expect(event.args.repayAmount.toString()).toEqual(
        to_dec6(4_500).toString(),
      )
      expect(event.args.accountBorrows.toString()).toEqual(
        to_dec6(5_500).toString(),
      )
      expect(event.args.totalBorrows.toNumber()).toBeGreaterThanOrEqual(
        to_dec6(5_500).toNumber(),
      )
    })
  })

  it('.repay_borrow_behalf', async () => {
    const {
      pools: {
        dai: { pool },
      },
      users,
    } = await setup()
    const { value } = await pool
      .withSigner(users[0])
      .query.repayBorrowBehalf(users[1].address, 0)
    expect(value.ok.err).toStrictEqual({ notImplemented: null })
  })

  describe('.liquidate_borrow', () => {
    let deployer: KeyringPair
    let controller: Controller
    let pools: Pools
    let users: KeyringPair[]

    beforeAll(async () => {
      ;({ deployer, controller, pools, users } = await setup())
    })

    it('preparations', async () => {
      const { dai, usdc } = pools

      // add liquidity to usdc pool
      await usdc.token.tx.mint(deployer.address, to_dec6(10_000))
      await usdc.token.tx.approve(usdc.pool.address, to_dec6(10_000))
      await usdc.pool.tx.mint(to_dec6(10_000))
      expect(
        (await usdc.pool.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(to_dec6(10_000).toNumber())

      // mint to dai pool for collateral
      const [borrower] = users
      await dai.token.tx.mint(borrower.address, to_dec18(20_000))
      await dai.token
        .withSigner(borrower)
        .tx.approve(dai.pool.address, to_dec18(20_000))
      await dai.pool.withSigner(borrower).tx.mint(to_dec18(20_000))
      expect(
        BigInt(
          (
            await dai.pool.query.balanceOf(borrower.address)
          ).value.ok.toString(),
        ).toString(),
      ).toEqual(to_dec18(20_000).toString())

      // borrow usdc
      await usdc.pool.withSigner(borrower).tx.borrow(to_dec6(10_000))
      expect(
        (
          await usdc.token.query.balanceOf(borrower.address)
        ).value.ok.toNumber(),
      ).toEqual(to_dec6(10_000).toNumber())

      // down collateral_factor for dai
      const toParam = (m: BN) => [m.toString()]
      await controller.tx.setCollateralFactorMantissa(
        dai.pool.address,
        toParam(new BN(1)),
      )
      const [collateralValue, shortfallValue] = (
        await controller.query.getHypotheticalAccountLiquidity(
          borrower.address,
          ZERO_ADDRESS,
          0,
          0,
          null,
        )
      ).value.ok.ok
      expect(collateralValue.toString()).toEqual('0')
      expect(BigInt(shortfallValue.toString())).toBeGreaterThanOrEqual(
        BigInt(9999) * BigInt(10) ** BigInt(18),
      )
    })

    // TODO: fix/check calculation seize token amount
    it('execute', async () => {
      const [borrower, repayer] = users
      const collateral = pools.dai
      const borrowing = pools.usdc
      await borrowing.token.tx.mint(repayer.address, to_dec6(5_000))
      await borrowing.token
        .withSigner(repayer)
        .tx.approve(borrowing.pool.address, to_dec6(5_000))

      const { events } = await borrowing.pool
        .withSigner(repayer)
        .tx.liquidateBorrow(
          borrower.address,
          to_dec6(5_000),
          collateral.pool.address,
        )

      expect(
        (
          await borrowing.token.query.balanceOf(repayer.address)
        ).value.ok.toNumber(),
      ).toEqual(0)
      expect(
        (
          await borrowing.token.query.balanceOf(borrower.address)
        ).value.ok.toNumber(),
      ).toEqual(to_dec6(10_000).toNumber())
      expect(
        (
          await borrowing.pool.query.borrowBalanceStored(borrower.address)
        ).value.ok.toNumber(),
      ).toEqual(to_dec6(5_000).toNumber())
      // TODO: check seized

      expect(events[0].name).toEqual('RepayBorrow')
      const event = events[1]
      expect(event.name).toEqual('LiquidateBorrow')
      expect(event.args.liquidator).toEqual(repayer.address)
      expect(event.args.borrower).toEqual(borrower.address)
      expect(event.args.repayAmount.toNumber()).toEqual(
        to_dec6(5_000).toNumber(),
      )
      expect(event.args.tokenCollateral).toEqual(collateral.pool.address)
      expect(event.args.seizeTokens.toNumber()).toEqual(0) // TODO: fix
    })
  })

  describe('.liquidate_borrow (fail case)', () => {
    const setupExtended = async () => {
      const args = await setup()

      const secondToken = await deployPSP22Token({
        api: args.api,
        signer: args.deployer,
        args: [
          0,
          'Dai Stablecoin' as unknown as string[],
          'DAI' as unknown as string[],
          8,
        ],
      })

      const secondPool = await deployPoolFromAsset({
        api: args.api,
        signer: args.deployer,
        args: [
          secondToken.address,
          args.controller.address,
          args.rateModel.address,
          [ONE_ETHER.toString()],
        ],
        token: secondToken,
      })

      // initialize for pool
      await args.controller.tx.supportMarket(secondPool.address)
      await args.priceOracle.tx.setFixedPrice(secondToken.address, ONE_ETHER)
      const toParam = (m: BN) => [m.toString()]
      await args.controller.tx.setCollateralFactorMantissa(
        secondPool.address,
        toParam(ONE_ETHER.mul(new BN(90)).div(new BN(100))),
      )
      return {
        ...args,
        secondToken,
        secondPool,
      }
    }

    it('when liquidator is equal to borrower', async () => {
      const {
        pools: {
          dai: { pool },
        },
        users,
        secondPool,
      } = await setupExtended()
      const [user1] = users
      const { value } = await pool
        .withSigner(user1)
        .query.liquidateBorrow(user1.address, 0, secondPool.address)
      expect(value.ok.err).toStrictEqual({
        liquidateLiquidatorIsBorrower: null,
      })
    })
    it('when repay_amount is zero', async () => {
      const {
        pools: {
          dai: { pool },
        },
        users,
        secondPool,
      } = await setupExtended()
      const [user1, user2] = users
      const { value } = await pool
        .withSigner(user1)
        .query.liquidateBorrow(user2.address, 0, secondPool.address)
      expect(value.ok.err).toStrictEqual({
        liquidateCloseAmountIsZero: null,
      })
    })
  })

  describe('.transfer', () => {
    const assertAccountLiqudity = (
      actual: [ReturnNumber, ReturnNumber],
      expected: { collateral: number; shortfall: number },
    ) => {
      const collateral = BigInt(actual[0].toString()).toString()
      const shortfall = BigInt(actual[1].toString()).toString()
      expect(collateral.toString()).toEqual(
        new BN(expected.collateral).mul(mantissa()).toString(),
      )
      expect(shortfall.toString()).toEqual(
        new BN(expected.shortfall).mul(mantissa()).toString(),
      )
    }

    it('success', async () => {
      const { api, deployer, controller, rateModel, priceOracle, users } =
        await setup()
      const { dai, usdc } = await preparePoolsWithPreparedTokens({
        api,
        controller,
        rateModel,
        manager: deployer,
      })
      const [userA, userB] = users

      // prerequisite
      //// initialize
      const toParam = (m: BN) => [m.toString()]
      for (const sym of [dai, usdc]) {
        await priceOracle.tx.setFixedPrice(sym.token.address, ONE_ETHER)
        await controller.tx.supportMarketWithCollateralFactorMantissa(
          sym.pool.address,
          toParam(ONE_ETHER.mul(new BN(90)).div(new BN(100))),
        )
      }
      //// use protocol
      for await (const { user, sym, amount } of [
        {
          user: userA,
          sym: dai,
          amount: to_dec18(500_000),
        },
        {
          user: userB,
          sym: usdc,
          amount: to_dec6(500_000),
        },
      ]) {
        const { pool, token } = sym
        await token.withSigner(deployer).tx.mint(user.address, amount)
        await token.withSigner(user).tx.approve(pool.address, amount)
        await pool.withSigner(user).tx.mint(amount)
      }
      expect(
        (await dai.pool.query.balanceOf(userA.address)).value.ok.toString(),
      ).toBe(to_dec18(500_000).toString())
      expect(
        (await usdc.pool.query.balanceOf(userB.address)).value.ok.toString(),
      ).toBe(to_dec6(500_000).toString())

      {
        const { events } = await shouldNotRevert(
          dai.pool.withSigner(userA),
          'transfer',
          [userB.address, to_dec18(100_000), []],
        )
        // assertions
        expect(
          (await dai.pool.query.balanceOf(userA.address)).value.ok.toString(),
        ).toBe(to_dec18(400_000).toString())
        expect(
          (await dai.pool.query.balanceOf(userB.address)).value.ok.toString(),
        ).toBe(to_dec18(100_000).toString())
        expect(events).toHaveLength(1)
        //// check event
        const event = events[0]
        expect(event.name).toEqual('Transfer')
        expect(event.args.from).toEqual(userA.address)
        expect(event.args.to).toEqual(userB.address)
        expect(event.args.value.toString()).toEqual(
          to_dec18(100_000).toString(),
        )
        //// check account_liquidity
        assertAccountLiqudity(
          (await controller.query.getAccountLiquidity(userA.address)).value.ok
            .ok,
          {
            collateral: (400_000 * 90) / 100,
            shortfall: 0,
          },
        )
        assertAccountLiqudity(
          (await controller.query.getAccountLiquidity(userB.address)).value.ok
            .ok,
          {
            collateral: ((100_000 + 500_000) * 90) / 100,
            shortfall: 0,
          },
        )
      }
      {
        const { events } = await shouldNotRevert(
          usdc.pool.withSigner(userB),
          'transfer',
          [userA.address, to_dec6(200_000), []],
        )
        // assertions
        expect(
          (await usdc.pool.query.balanceOf(userA.address)).value.ok.toString(),
        ).toBe(to_dec6(200_000).toString())
        expect(
          (await usdc.pool.query.balanceOf(userB.address)).value.ok.toString(),
        ).toBe(to_dec6(300_000).toString())
        expect(events).toHaveLength(1)
        //// check event
        const event = events[0]
        expect(event.name).toEqual('Transfer')
        expect(event.args.from).toEqual(userB.address)
        expect(event.args.to).toEqual(userA.address)
        expect(event.args.value.toString()).toEqual(to_dec6(200_000).toString())
        //// check account_liquidity
        assertAccountLiqudity(
          (await controller.query.getAccountLiquidity(userA.address)).value.ok
            .ok,
          {
            collateral: ((400_000 + 200_000) * 90) / 100,
            shortfall: 0,
          },
        )
        assertAccountLiqudity(
          (await controller.query.getAccountLiquidity(userB.address)).value.ok
            .ok,
          {
            collateral: ((100_000 + 300_000) * 90) / 100,
            shortfall: 0,
          },
        )
      }
    })

    it('failure', async () => {
      const { api, deployer, controller, rateModel, priceOracle, users } =
        await setup()
      const { dai, usdc } = await preparePoolsWithPreparedTokens({
        api,
        controller,
        rateModel,
        manager: deployer,
      })
      const [userA, userB] = users

      // prerequisite
      //// initialize
      const toParam = (m: BN) => [m.toString()]
      for (const sym of [dai, usdc]) {
        await priceOracle.tx.setFixedPrice(sym.token.address, ONE_ETHER)
        await controller.tx.supportMarketWithCollateralFactorMantissa(
          sym.pool.address,
          toParam(ONE_ETHER.mul(new BN(90)).div(new BN(100))),
        )
      }
      //// use protocol
      for await (const { user, sym, amount } of [
        {
          user: userA,
          sym: dai,
          amount: to_dec18(500_000),
        },
        {
          user: userB,
          sym: usdc,
          amount: to_dec6(500_000),
        },
      ]) {
        const { pool, token } = sym
        await token.withSigner(deployer).tx.mint(user.address, amount)
        await token.withSigner(user).tx.approve(pool.address, amount)
        await pool.withSigner(user).tx.mint(amount)
      }

      // case: src == dst
      {
        const res = await dai.pool
          .withSigner(userA)
          .query.transfer(userA.address, new BN(1), [])
        expect(hexToUtf8(res.value.ok.err['custom'])).toBe('TransferNotAllowed')
      }
      // case: shortfall of account_liquidity

      {
        await usdc.pool.withSigner(userA).tx.borrow(to_dec6(450_000))
        const res = await dai.pool
          .withSigner(userA)
          .query.transfer(userB.address, new BN(2), []) // temp: truncated if less than 1 by collateral_factor?
        expect(hexToUtf8(res.value.ok.err['custom'])).toBe(
          'InsufficientLiquidity',
        )
      }
      // case: paused
      await controller.tx.setTransferGuardianPaused(true)
      {
        const res = await dai.pool
          .withSigner(userA)
          .query.transfer(userB.address, new BN(1), [])
        expect(hexToUtf8(res.value.ok.err['custom'])).toBe('TransferIsPaused')
      }
    })
  })

  describe('.transfer_from', () => {
    it('success', async () => {
      const { api, deployer, controller, rateModel, priceOracle, users } =
        await setup()
      const { dai, usdc } = await preparePoolsWithPreparedTokens({
        api,
        controller,
        rateModel,
        manager: deployer,
      })
      const [userA, userB] = users
      const spender = deployer

      // prerequisite
      //// initialize
      const toParam = (m: BN) => [m.toString()]
      for (const sym of [dai, usdc]) {
        await priceOracle.tx.setFixedPrice(sym.token.address, ONE_ETHER)
        await controller.tx.supportMarketWithCollateralFactorMantissa(
          sym.pool.address,
          toParam(ONE_ETHER.mul(new BN(90)).div(new BN(100))),
        )
      }
      //// use protocol
      for await (const { user, sym, amount } of [
        {
          user: userA,
          sym: dai,
          amount: to_dec18(500_000),
        },
        {
          user: userB,
          sym: usdc,
          amount: to_dec6(500_000),
        },
      ]) {
        const { pool, token } = sym
        await token.withSigner(deployer).tx.mint(user.address, amount)
        await token.withSigner(user).tx.approve(pool.address, amount)
        await pool.withSigner(user).tx.mint(amount)
      }
      expect(
        (await dai.pool.query.balanceOf(userA.address)).value.ok.toString(),
      ).toBe(to_dec18(500_000).toString())
      expect(
        (await usdc.pool.query.balanceOf(userB.address)).value.ok.toString(),
      ).toBe(to_dec6(500_000).toString())

      {
        // approve
        const { events: approveEvents } = await shouldNotRevert(
          dai.pool.withSigner(userA),
          'approve',
          [spender.address, to_dec18(100_000)],
        )
        //// assertions
        expect(
          (
            await dai.pool.query.allowance(userA.address, spender.address)
          ).value.ok.toString(),
        ).toBe(to_dec18(100_000).toString())
        ////// check event
        expect(approveEvents).toHaveLength(1)
        const event = approveEvents[0]
        expect(event.name).toEqual('Approval')
        expect(event.args.owner).toEqual(userA.address)
        expect(event.args.spender).toEqual(spender.address)
        expect(event.args.value.toString()).toEqual(
          to_dec18(100_000).toString(),
        )

        // transfer_from
        const { events: transferFromEvents } = await shouldNotRevert(
          dai.pool.withSigner(spender),
          'transferFrom',
          [userA.address, userB.address, to_dec18(100_000), []],
        )
        //// assertions
        expect(
          (
            await dai.pool.query.allowance(userA.address, spender.address)
          ).value.ok.toString(),
        ).toBe('0')
        expect(
          (await dai.pool.query.balanceOf(userA.address)).value.ok.toString(),
        ).toBe(to_dec18(400_000).toString())
        expect(
          (await dai.pool.query.balanceOf(userB.address)).value.ok.toString(),
        ).toBe(to_dec18(100_000).toString())
        expect(transferFromEvents).toHaveLength(2)
        //// check event
        const approvalEvent = transferFromEvents[0]
        expect(approvalEvent.name).toEqual('Approval')
        expect(approvalEvent.args.owner).toEqual(userA.address)
        expect(approvalEvent.args.spender).toEqual(spender.address)
        expect(approvalEvent.args.value.toString()).toEqual('0')
        const transferEvent = transferFromEvents[1]
        expect(transferEvent.name).toEqual('Transfer')
        expect(transferEvent.args.from).toEqual(userA.address)
        expect(transferEvent.args.to).toEqual(userB.address)
        expect(transferEvent.args.value.toString()).toEqual(
          to_dec18(100_000).toString(),
        )
      }
    })
    it('failure', async () => {
      const { api, deployer, controller, rateModel, priceOracle, users } =
        await setup()
      const { dai, usdc } = await preparePoolsWithPreparedTokens({
        api,
        controller,
        rateModel,
        manager: deployer,
      })
      const [userA, userB] = users
      const spender = deployer

      // prerequisite
      //// initialize
      const toParam = (m: BN) => [m.toString()]
      for (const sym of [dai, usdc]) {
        await priceOracle.tx.setFixedPrice(sym.token.address, ONE_ETHER)
        await controller.tx.supportMarketWithCollateralFactorMantissa(
          sym.pool.address,
          toParam(ONE_ETHER.mul(new BN(90)).div(new BN(100))),
        )
      }
      //// use protocol
      for await (const { user, sym, amount } of [
        {
          user: userA,
          sym: dai,
          amount: to_dec18(500_000),
        },
        {
          user: userB,
          sym: usdc,
          amount: to_dec6(500_000),
        },
      ]) {
        const { pool, token } = sym
        await token.withSigner(deployer).tx.mint(user.address, amount)
        await token.withSigner(user).tx.approve(pool.address, amount)
        await pool.withSigner(user).tx.mint(amount)
      }
      expect(
        (await dai.pool.query.balanceOf(userA.address)).value.ok.toString(),
      ).toBe(to_dec18(500_000).toString())
      expect(
        (await usdc.pool.query.balanceOf(userB.address)).value.ok.toString(),
      ).toBe(to_dec6(500_000).toString())

      // case: shortage of approval
      {
        await shouldNotRevert(dai.pool.withSigner(userA), 'approve', [
          spender.address,
          to_dec18(99_999),
        ])
        const res = await dai.pool
          .withSigner(spender)
          .query.transferFrom(
            userA.address,
            userB.address,
            to_dec18(100_000),
            [],
          )
        expect(res.value.ok.err.insufficientAllowance).toBeTruthy
      }
      await shouldNotRevert(dai.pool.withSigner(userA), 'approve', [
        spender.address,
        to_dec18(100_000),
      ])
      // case: src == dst
      {
        const res = await dai.pool
          .withSigner(spender)
          .query.transferFrom(userA.address, userA.address, new BN(1), [])
        expect(hexToUtf8(res.value.ok.err['custom'])).toBe('TransferNotAllowed')
      }
      // case: shortfall of account_liquidity
      {
        await usdc.pool.withSigner(userA).tx.borrow(to_dec6(450_000))
        const res = await dai.pool
          .withSigner(spender)
          .query.transferFrom(userA.address, userB.address, new BN(2), []) // temp: truncated if less than 1 by collateral_factor?
        expect(hexToUtf8(res.value.ok.err['custom'])).toBe(
          'InsufficientLiquidity',
        )
      }
      // case: paused
      await controller.tx.setTransferGuardianPaused(true)
      {
        const res = await dai.pool
          .withSigner(spender)
          .query.transferFrom(userA.address, userB.address, new BN(1), [])
        expect(hexToUtf8(res.value.ok.err['custom'])).toBe('TransferIsPaused')
      }
    })
  })

  it.todo('.reduceReserves')

  describe('.exchange_rate_stored', () => {
    describe('success', () => {
      it('return initial_exchange_rate_stored if no total_supply', async () => {
        const { api, deployer, controller, rateModel } = await setup()

        const newToken = await deployPSP22Token({
          api: api,
          signer: deployer,
          args: [
            0,
            'Sample Coin' as unknown as string[],
            'COIN' as unknown as string[],
            8,
          ],
        })
        const initialExchangeRate = ONE_ETHER
        const newPool = await deployPoolFromAsset({
          api: api,
          signer: deployer,
          args: [
            newToken.address,
            controller.address,
            rateModel.address,
            [initialExchangeRate.toString()],
          ],
          token: newToken,
        })

        // prerequisite
        expect((await newPool.query.totalSupply()).value.ok.toString()).toBe(
          '0',
        )
        expect(
          (
            await newPool.query.initialExchangeRateMantissa()
          ).value.ok.toString(),
        ).toBe(initialExchangeRate.toString())
        // execution
        expect(
          (await newPool.query.exchageRateStored()).value.ok.toString(),
        ).toBe(initialExchangeRate.toString())
      })
    })
  })
})
