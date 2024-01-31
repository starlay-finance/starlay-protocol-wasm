/* eslint-disable dot-notation */
import type { KeyringPair } from '@polkadot/keyring/types'
import { WeightV2 } from '@polkadot/types/interfaces'
import { BN, BN_ONE, BN_TEN } from '@polkadot/util'
import { ONE_ETHER, ZERO_ADDRESS } from '../scripts/helper/constants'
import {
  deployController,
  deployDefaultInterestRateModel,
  deployIncentivesController,
  deployPSP22Token,
  deployPoolFromAsset,
  deployPriceOracle,
} from '../scripts/helper/deploy_helper'
import { getGasLimit } from '../scripts/helper/utils'
import { RATE_MODELS } from '../scripts/interest_rates'
import DefaultInterestRateModel from '../types/contracts/default_interest_rate_model'
import Pool from '../types/contracts/pool'
import PSP22Token from '../types/contracts/psp22_token'
import {
  Mint,
  Redeem,
  ReserveUsedAsCollateralEnabled,
} from '../types/event-types/pool'
import { Transfer } from '../types/event-types/psp22_token'
import { Pools, preparePoolsWithPreparedTokens } from './testContractHelper'
import {
  expectToEmit,
  mantissa,
  shouldNotRevert,
  toDec18,
  toDec6,
} from './testHelpers'

const MAX_CALL_WEIGHT = new BN(125_000_000_000).isub(BN_ONE).mul(BN_TEN)
const PROOFSIZE = new BN(2_000_000)
describe('Pool spec 1', () => {
  const setup = async (model?: DefaultInterestRateModel) => {
    const { api, alice: deployer, bob, charlie, django } = globalThis.setup

    const gasLimit = getGasLimit(api, MAX_CALL_WEIGHT, PROOFSIZE)
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

    const rateModel = model
      ? model
      : await deployDefaultInterestRateModel({
          api,
          signer: deployer,
          args: [[0], [0], [0], [0]],
        })

    const incentivesController = await deployIncentivesController({
      api,
      signer: deployer,
      args: [],
    })

    const pools = await preparePoolsWithPreparedTokens({
      api,
      controller,
      rateModel,
      manager: deployer,
      incentivesController,
    })

    const users = [bob, charlie, django]

    // initialize
    await controller.tx.setPriceOracle(priceOracle.address)
    await controller.tx.setCloseFactorMantissa([ONE_ETHER])
    //// for pool
    for (const sym of [pools.dai, pools.usdc, pools.usdt]) {
      await priceOracle.tx.setFixedPrice(sym.token.address, ONE_ETHER)
      await controller.tx.supportMarketWithCollateralFactorMantissa(
        sym.pool.address,
        sym.token.address,
        [ONE_ETHER.mul(new BN(90)).div(new BN(100))],
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
      incentivesController,
      gasLimit,
    }
  }

  it('instantiate', async () => {
    const { pools, controller } = await setup()
    const { pool, token } = pools.dai
    expect(pool.address).not.toBe(ZERO_ADDRESS)
    expect((await pool.query.underlying()).value.ok).toEqual(token.address)
    expect((await pool.query.controller()).value.ok).toEqual(controller.address)
    expect((await pool.query.tokenName()).value.ok).toEqual(
      'Starlay Dai Stablecoin',
    )
    expect((await pool.query.tokenSymbol()).value.ok).toEqual('sDAI')
    expect((await pool.query.tokenDecimals()).value.ok).toEqual(18)
    expect(
      (await pool.query.liquidationThreshold()).value.ok.toString(),
    ).toEqual('10000')
  })

  describe('.mint', () => {
    let deployer: KeyringPair
    let token: PSP22Token
    let pool: Pool

    beforeAll(async () => {
      let api
      let rateModel
      let controller
      let priceOracle
      let incentivesController
      ;({
        api,
        deployer,
        rateModel,
        controller,
        priceOracle,
        incentivesController,
      } = await setup())

      token = await deployPSP22Token({
        api,
        signer: deployer,
        args: [0, 'Sample', 'SAMPLE', 6],
      })

      pool = await deployPoolFromAsset({
        api,
        signer: deployer,
        args: [
          incentivesController.address,
          token.address,
          controller.address,
          rateModel.address,
          [ONE_ETHER.div(new BN(2)).toString()], // pool = underlying * 2
          10000,
        ],
        token,
      })

      await priceOracle.tx.setFixedPrice(token.address, ONE_ETHER)
      await controller.tx.supportMarketWithCollateralFactorMantissa(
        pool.address,
        token.address,
        [ONE_ETHER.mul(new BN(90)).div(new BN(100))],
      )
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
      const mintAmount = depositAmount * 2
      await shouldNotRevert(token, 'approve', [pool.address, depositAmount])
      const { events } = await shouldNotRevert(pool, 'mint', [depositAmount])

      // const dec18 = BigInt(10) ** BigInt(18)
      expect(
        (await token.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toBe(balance - depositAmount)
      expect(
        (await token.query.balanceOf(pool.address)).value.ok.toNumber(),
      ).toBe(depositAmount)
      expect(
        (await pool.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toBe(depositAmount) // NOTE: because balanceOf is converted to underlying value
      expect(
        (
          await pool.query.principalBalanceOf(deployer.address)
        ).value.ok.toNumber(),
      ).toBe(mintAmount)

      expect(events).toHaveLength(3)
      expectToEmit<ReserveUsedAsCollateralEnabled>(
        events[0],
        'ReserveUsedAsCollateralEnabled',
        {
          user: deployer.address,
        },
      )
      expectToEmit<Transfer>(events[1], 'Transfer', {
        from: null,
        to: deployer.address,
        value: mintAmount,
      })
      expectToEmit<Mint>(events[2], 'Mint', {
        minter: deployer.address,
        mintAmount: depositAmount,
        mintTokens: mintAmount,
      })
    })
  })

  describe('.redeem_underlying', () => {
    let deployer: KeyringPair
    let token: PSP22Token
    let pool: Pool
    let gasLimit: WeightV2

    beforeAll(async () => {
      ;({
        deployer,
        pools: {
          dai: { token, pool },
        },
        gasLimit,
      } = await setup())
    })

    const deposited = 10_000
    const minted = deposited
    it('setup', async () => {
      await shouldNotRevert(token, 'mint', [deployer.address, deposited])

      await shouldNotRevert(token, 'approve', [pool.address, deposited])
      await shouldNotRevert(pool, 'mint', [deposited, { gasLimit }])
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
        { gasLimit },
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
      })
    })
  })

  describe('.redeem (fail case)', () => {
    it('when no cash in pool', async () => {
      const {
        deployer,
        pools: { usdc, usdt },
        gasLimit,
      } = await setup()

      for await (const { token, pool } of [usdc, usdt]) {
        await shouldNotRevert(token, 'mint', [deployer.address, toDec6(10_000)])
        await shouldNotRevert(token, 'approve', [pool.address, toDec6(10_000)])
        await shouldNotRevert(pool, 'mint', [toDec6(10_000), { gasLimit }])
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

  describe('.redeem_all', () => {
    it('success', async () => {
      const { api, alice: deployer } = globalThis.setup
      const daiRateModel = RATE_MODELS.dai
      const rateModel = await deployDefaultInterestRateModel({
        api,
        signer: deployer,
        args: [
          [daiRateModel.baseRatePerYear()],
          [daiRateModel.multiplierPerYearSlope1()],
          [daiRateModel.multiplierPerYearSlope2()],
          [daiRateModel.kink()],
        ],
      })
      const {
        pools: {
          usdc: { token, pool },
        },
        gasLimit,
      } = await setup(rateModel)

      await shouldNotRevert(token, 'mint', [deployer.address, toDec6(60_000)])
      await shouldNotRevert(token, 'approve', [pool.address, toDec6(60_000)])
      await shouldNotRevert(pool, 'mint', [toDec6(10_000), { gasLimit }])
      await shouldNotRevert(pool, 'mint', [toDec6(20_000), { gasLimit }])
      await shouldNotRevert(pool, 'mint', [toDec6(30_000), { gasLimit }])
      expect(
        (await pool.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toBe(toDec6(60_000).toNumber())

      await pool.withSigner(deployer).tx.redeemAll({ gasLimit })
      expect(
        (await pool.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toBe(0)
      expect(
        (await token.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toBe(toDec6(60_000).toNumber())
    })
  })

  describe('.borrow', () => {
    let deployer: KeyringPair
    let token: PSP22Token
    let pool: Pool
    let users: KeyringPair[]
    let gasLimit: WeightV2

    beforeAll(async () => {
      ;({
        deployer,
        users,
        pools: {
          dai: { token, pool },
        },
        gasLimit,
      } = await setup())
    })

    it('preparations', async () => {
      const [user1, user2] = users
      await token.withSigner(deployer).tx.mint(user1.address, 5_000)
      await token.withSigner(deployer).tx.mint(user2.address, 5_000)
      await token.withSigner(user1).tx.approve(pool.address, 5_000)
      await pool.withSigner(user1).tx.mint(5_000, { gasLimit })
      await token.withSigner(user2).tx.approve(pool.address, 5_000)
      await pool.withSigner(user2).tx.mint(5_000, { gasLimit })
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
      const { events: events1 } = await pool
        .withSigner(user1)
        .tx.borrow(3_000, { gasLimit })

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

      const { events: events2 } = await pool
        .withSigner(user2)
        .tx.borrow(2_500, { gasLimit })

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
        gasLimit,
      } = await setup()

      await usdc.token.tx.mint(deployer.address, toDec6(5_000))
      await usdc.token.tx.approve(usdc.pool.address, toDec6(5_000))
      await usdc.pool.tx.mint(toDec6(5_000), { gasLimit })

      const { value } = await usdt.pool.query.borrow(toDec6(1_000))
      expect(value.ok.err).toStrictEqual({ borrowCashNotAvailable: null })
    })
  })

  describe('.repay_borrow', () => {
    let deployer: KeyringPair
    let pools: Pools
    let users: KeyringPair[]
    let gasLimit: WeightV2

    beforeAll(async () => {
      ;({ deployer, users, pools } = await setup())
    })

    it('preparations', async () => {
      const { dai, usdc } = pools

      // add liquidity to usdc pool
      await usdc.token.tx.mint(deployer.address, toDec6(10_000))
      await usdc.token.tx.approve(usdc.pool.address, toDec6(10_000))
      await usdc.pool.tx.mint(toDec6(10_000), { gasLimit })
      expect(
        BigInt(
          (
            await usdc.pool.query.balanceOf(deployer.address)
          ).value.ok.toString(),
        ).toString(),
      ).toEqual(toDec6(10_000).toString())

      // mint to dai pool for collateral
      const [user1] = users
      await dai.token.tx.mint(user1.address, toDec18(20_000))
      await dai.token
        .withSigner(user1)
        .tx.approve(dai.pool.address, toDec18(20_000))
      await dai.pool.withSigner(user1).tx.mint(toDec18(20_000), { gasLimit })
      expect(
        BigInt(
          (await dai.pool.query.balanceOf(user1.address)).value.ok.toString(),
        ).toString(),
      ).toEqual(toDec18(20_000).toString())

      // borrow usdc
      await usdc.pool.withSigner(user1).tx.borrow(toDec6(10_000), { gasLimit })
      expect(
        BigInt(
          (await usdc.token.query.balanceOf(user1.address)).value.ok.toString(),
        ).toString(),
      ).toEqual(toDec6(10_000).toString())
    })

    it('execute', async () => {
      const { token, pool } = pools.usdc
      const [user1] = users
      await token.withSigner(user1).tx.approve(pool.address, toDec6(4_500))
      const { events } = await pool
        .withSigner(user1)
        .tx.repayBorrow(toDec6(4_500), { gasLimit })

      expect(
        BigInt(
          (await token.query.balanceOf(user1.address)).value.ok.toString(),
        ).toString(),
      ).toEqual(toDec6(5_500).toString())
      expect(
        BigInt(
          (await token.query.balanceOf(pool.address)).value.ok.toString(),
        ).toString(),
      ).toEqual(toDec6(4_500).toString())

      const event = events[0]
      expect(event.name).toEqual('RepayBorrow')
      expect(event.args.payer).toEqual(user1.address)
      expect(event.args.borrower).toEqual(user1.address)
      expect(event.args.repayAmount.toString()).toEqual(
        toDec6(4_500).toString(),
      )
      expect(event.args.accountBorrows.toString()).toEqual(
        toDec6(5_500).toString(),
      )
      expect(event.args.totalBorrows.toNumber()).toEqual(
        toDec6(5_500).toNumber(),
      )
    })
  })

  describe('.repay_borrow_behalf', () => {
    let deployer: KeyringPair
    let pools: Pools
    let users: KeyringPair[]

    beforeAll(async () => {
      ;({ deployer, users, pools } = await setup())
    })

    it('preparations', async () => {
      const { dai, usdc } = pools

      // add liquidity to usdc pool
      await usdc.token.tx.mint(deployer.address, toDec6(10_000))
      await usdc.token.tx.approve(usdc.pool.address, toDec6(10_000))
      await usdc.pool.tx.mint(toDec6(10_000))
      expect(
        BigInt(
          (
            await usdc.pool.query.balanceOf(deployer.address)
          ).value.ok.toString(),
        ).toString(),
      ).toEqual(toDec6(10_000).toString())

      // mint to dai pool for collateral
      const [user1] = users
      await dai.token.tx.mint(user1.address, toDec18(20_000))
      await dai.token
        .withSigner(user1)
        .tx.approve(dai.pool.address, toDec18(20_000))
      await dai.pool.withSigner(user1).tx.mint(toDec18(20_000))
      expect(
        BigInt(
          (await dai.pool.query.balanceOf(user1.address)).value.ok.toString(),
        ).toString(),
      ).toEqual(toDec18(20_000).toString())

      // borrow usdc
      await usdc.pool.withSigner(user1).tx.borrow(toDec6(10_000))
      expect(
        BigInt(
          (await usdc.token.query.balanceOf(user1.address)).value.ok.toString(),
        ).toString(),
      ).toEqual(toDec6(10_000).toString())
    })

    it('execute', async () => {
      const { token, pool } = pools.usdc
      const [user1] = users
      await token.withSigner(user1).tx.approve(pool.address, toDec6(4_500))
      const { events } = await pool
        .withSigner(user1)
        .tx.repayBorrowBehalf(user1.address, toDec6(4_500))

      expect(
        BigInt(
          (await token.query.balanceOf(user1.address)).value.ok.toString(),
        ).toString(),
      ).toEqual(toDec6(5_500).toString())
      expect(
        BigInt(
          (await token.query.balanceOf(pool.address)).value.ok.toString(),
        ).toString(),
      ).toEqual(toDec6(4_500).toString())

      const event = events[0]
      expect(event.name).toEqual('RepayBorrow')
      expect(event.args.payer).toEqual(user1.address)
      expect(event.args.borrower).toEqual(user1.address)
      expect(event.args.repayAmount.toString()).toEqual(
        toDec6(4_500).toString(),
      )
      expect(event.args.accountBorrows.toString()).toEqual(
        toDec6(5_500).toString(),
      )
      expect(event.args.totalBorrows.toNumber()).toEqual(
        toDec6(5_500).toNumber(),
      )
    })
  })

  describe('.repay_borrow_all', () => {
    it('success', async () => {
      const {
        deployer,
        pools: { usdc, usdt },
        users,
        gasLimit,
      } = await setup()
      const [borrower] = users

      // prepares
      //// add liquidity to usdc pool
      await usdc.token.tx.mint(deployer.address, toDec6(100_000))
      await usdc.token.tx.approve(usdc.pool.address, toDec6(100_000))
      await usdc.pool.tx.mint(toDec6(100_000), { gasLimit })
      //// mint to usdt pool for collateral
      await usdt.token.tx.mint(borrower.address, toDec6(100_000))
      await usdt.token
        .withSigner(borrower)
        .tx.approve(usdt.pool.address, toDec6(100_000))
      await usdt.pool
        .withSigner(borrower)
        .tx.mint(toDec6(100_000), { gasLimit })
      //// borrow usdc
      await usdc.pool
        .withSigner(borrower)
        .tx.borrow(toDec6(10_000), { gasLimit })
      await usdc.pool
        .withSigner(borrower)
        .tx.borrow(toDec6(20_000), { gasLimit })
      await usdc.pool
        .withSigner(borrower)
        .tx.borrow(toDec6(30_000), { gasLimit })
      expect(
        BigInt(
          (
            await usdc.token.query.balanceOf(borrower.address)
          ).value.ok.toString(),
        ),
      ).toBe(BigInt(toDec6(60_000).toString()))

      // execute
      await usdc.token
        .withSigner(borrower)
        .tx.approve(usdc.pool.address, toDec6(999_999))
      await usdc.pool.withSigner(borrower).tx.repayBorrowAll({ gasLimit })
      expect(
        (
          await usdc.token.query.balanceOf(borrower.address)
        ).value.ok.toNumber(),
      ).toBe(0)
    })
  })

  describe('.liquidate_borrow', () => {
    const setupForShortage = async () => {
      const { deployer, controller, pools, users } = await setup()
      const { dai, usdc } = pools

      // add liquidity to usdc pool
      await usdc.token.tx.mint(deployer.address, toDec6(10_000))
      await usdc.token.tx.approve(usdc.pool.address, toDec6(10_000))
      await usdc.pool.tx.mint(toDec6(10_000))
      expect(
        (await usdc.pool.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(toDec6(10_000).toNumber())

      // mint to dai pool for collateral
      const [borrower] = users
      await dai.token.tx.mint(borrower.address, toDec18(20_000))
      await dai.token
        .withSigner(borrower)
        .tx.approve(dai.pool.address, toDec18(20_000))
      await dai.pool.withSigner(borrower).tx.mint(toDec18(20_000))
      expect(
        BigInt(
          (
            await dai.pool.query.balanceOf(borrower.address)
          ).value.ok.toString(),
        ).toString(),
      ).toEqual(toDec18(20_000).toString())

      // borrow usdc
      await usdc.pool.withSigner(borrower).tx.borrow(toDec6(10_000))
      expect(
        (
          await usdc.token.query.balanceOf(borrower.address)
        ).value.ok.toNumber(),
      ).toEqual(toDec6(10_000).toNumber())

      // down collateral_factor for dai
      await controller.tx.setCollateralFactorMantissa(dai.pool.address, [
        new BN(1),
      ])
      const [collateralValue, shortfallValue] = (
        await controller.query.getHypotheticalAccountLiquidity(
          borrower.address,
          ZERO_ADDRESS,
          0,
          0,
          null,
        )
      ).value.ok.ok
      expect(BigInt(collateralValue.toString())).toEqual(BigInt(0))
      expect(BigInt(shortfallValue.toString())).toBeGreaterThanOrEqual(
        BigInt(9999) * BigInt(10) ** BigInt(18),
      )

      return { deployer, controller, pools, users }
    }

    it('execute', async () => {
      const {
        controller,
        pools: { dai: collateral, usdc: borrowing },
        users,
      } = await setupForShortage()
      const [borrower, liquidator] = users
      await borrowing.token.tx.mint(liquidator.address, toDec6(5_000))
      await borrowing.token
        .withSigner(liquidator)
        .tx.approve(borrowing.pool.address, toDec6(5_000))

      const liquidationIncentiveMantissa = mantissa()
        .mul(new BN(108))
        .div(new BN(100)) // 1.08
      await controller.tx.setLiquidationIncentiveMantissa([
        liquidationIncentiveMantissa,
      ])
      const res = await borrowing.pool
        .withSigner(liquidator)
        .tx.liquidateBorrow(
          borrower.address,
          toDec6(5_000),
          collateral.pool.address,
        )

      expect(
        (
          await borrowing.token.query.balanceOf(liquidator.address)
        ).value.ok.toNumber(),
      ).toEqual(0)
      expect(
        (
          await borrowing.token.query.balanceOf(borrower.address)
        ).value.ok.toNumber(),
      ).toEqual(toDec6(10_000).toNumber())
      expect(
        (
          await borrowing.pool.query.borrowBalanceStored(borrower.address)
        ).value.ok.toNumber(),
      ).toEqual(toDec6(5_000).toNumber())

      expect(Object.keys(res.events).length).toBe(2)
      expect(res.events[0].name).toBe('RepayBorrow')
      const liquidateBorrowEvent = res.events[1]
      expect(liquidateBorrowEvent.name).toBe('LiquidateBorrow')
      expect(liquidateBorrowEvent.args.liquidator).toBe(liquidator.address)
      expect(liquidateBorrowEvent.args.borrower).toBe(borrower.address)
      expect(liquidateBorrowEvent.args.repayAmount.toNumber()).toBe(
        toDec6(5_000).toNumber(),
      )
      expect(liquidateBorrowEvent.args.tokenCollateral).toBe(
        collateral.pool.address,
      )
      const seizeTokens = liquidateBorrowEvent.args.seizeTokens.toString()
      // seizeTokens ≒ actual_repay_amount * liquidation_incentive
      const dec18 = BigInt(10) ** BigInt(18)
      expect(BigInt(seizeTokens)).toBe(
        (BigInt(5000) * dec18 * BigInt(108)) / BigInt(100),
      )

      // console.log(res.events)
      // // check events from Pool (execute seize)
      // const contractEvents = res.events
      // //// Burn
      // const burnEvent = contractEvents.find(
      //   (e) => e.name == 'Transfer' && e.from == borrower.address,
      // )
      // expect(burnEvent.from).toBe(borrower.address.toString())
      // expect(burnEvent.to).toBe('')
      // expect(BigInt(burnEvent.value)).toBe(5400n * dec18)
      // //// Mint
      // const mintEvent = contractEvents.find(
      //   (e) => e.name == 'Transfer' && e.to == liquidator.address,
      // )
      // expect(mintEvent.from).toBe('')
      // expect(mintEvent.to).toBe(liquidator.address.toString())
      // const minted = BigInt(mintEvent.value)
      // expect(minted).toBeGreaterThanOrEqual(5248n * dec18)
      // expect(minted).toBeLessThanOrEqual(5249n * dec18)
      // //// ReserveAdded
      // const reservesAddedEvent = contractEvents.find(
      //   (e) => e.name == 'ReservesAdded',
      // )
      // expect(reservesAddedEvent.benefactor).toBe(
      //   collateral.pool.address.toString(),
      // )
      // const addedAmount = BigInt(reservesAddedEvent.add_amount)
      // expect(addedAmount).toBeGreaterThanOrEqual(151n * dec18)
      // expect(addedAmount).toBeLessThanOrEqual(152n * dec18)
      // const totalReserves = BigInt(reservesAddedEvent.new_total_reserves)
      // expect(totalReserves).toBeGreaterThanOrEqual(151n * dec18)
      // expect(totalReserves).toBeLessThanOrEqual(152n * dec18)
    })

    it('fail when liquidator is equal to borrower', async () => {
      const {
        pools: { dai: collateral, usdc: borrowing },
        users,
      } = await setupForShortage()
      const [borrower] = users
      const { value } = await borrowing.pool
        .withSigner(borrower)
        .query.liquidateBorrow(borrower.address, 0, collateral.pool.address)
      expect(value.ok.err).toStrictEqual({
        liquidateLiquidatorIsBorrower: null,
      })
    })
    it('fail when repay_amount is zero', async () => {
      const {
        pools: { dai: collateral, usdc: borrowing },
        users,
      } = await setupForShortage()
      const [borrower, liquidator] = users
      const { value } = await borrowing.pool
        .withSigner(liquidator)
        .query.liquidateBorrow(borrower.address, 0, collateral.pool.address)
      expect(value.ok.err).toStrictEqual({
        liquidateCloseAmountIsZero: null,
      })
    })
  })

  it('.seize (cannot call by users)', async () => {
    const {
      api,
      deployer,
      controller,
      rateModel,
      priceOracle,
      users,
      incentivesController,
    } = await setup()
    const { dai, usdc } = await preparePoolsWithPreparedTokens({
      api,
      controller,
      rateModel,
      manager: deployer,
      incentivesController,
    })
    const toParam = (m: BN) => [m.toString()]
    for (const sym of [dai, usdc]) {
      await priceOracle.tx.setFixedPrice(sym.token.address, ONE_ETHER)
      await controller.tx.supportMarketWithCollateralFactorMantissa(
        sym.pool.address,
        sym.token.address,
        toParam(ONE_ETHER.mul(new BN(90)).div(new BN(100))),
      )
    }

    // call dai pool from user
    const { value: val1 } = await dai.pool.query.seize(
      deployer.address,
      users[0].address,
      toDec18(100),
    )
    expect(val1.ok.err).toEqual({ controller: 'MarketNotListed' })
    // call usdc pool from user
    const { value: val2 } = await usdc.pool.query.seize(
      deployer.address,
      users[0].address,
      toDec6(100),
    )
    expect(val2.ok.err).toEqual({ controller: 'MarketNotListed' })
  })

  it('Unprotected liquidation threshold setter', async () => {
    /*
    reproduced in `tests/Pool1.spec.ts`
    command: `yarn test:single --testNamePattern "Unprotected liquidation threshold setter"`
    */
    const { pools, users } = await setup()
    const liqThresholdBefore = (
      await pools.dai.pool.query.liquidationThreshold()
    ).value.ok.toString()
    console.log('threshold before: ', liqThresholdBefore)
    const result = await pools.dai.pool
      .withSigner(users[1])
      .query.setLiquidationThreshold(10)

    expect(result.value.ok.err).toStrictEqual({
      callerIsNotManager: null,
    })
  })
})
