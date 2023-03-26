import type { KeyringPair } from '@polkadot/keyring/types'
import { BN } from '@polkadot/util'
import {
  expectToEmit,
  hexToUtf8,
  shouldNotRevert,
  zeroAddress,
} from './testHelpers'

import Pool from '../types/contracts/pool'

import {
  deployController,
  deployDefaultInterestRateModel,
  deployPoolFromAsset,
  deployPriceOracle,
  deployPSP22Token,
} from '../scripts/helper/deploy_helper'
import { ONE_ETHER } from '../scripts/tokens'
import PSP22Token from '../types/contracts/psp22_token'
import { Mint, Redeem } from '../types/event-types/pool'
import { Transfer } from '../types/event-types/psp22_token'

describe('Pool spec', () => {
  const setup = async () => {
    const { api, alice: deployer, bob, charlie } = globalThis.setup

    const token = await deployPSP22Token({
      api,
      signer: deployer,
      args: [
        0,
        'Dai Stablecoin' as unknown as string[],
        'DAI' as unknown as string[],
        8,
      ],
    })

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

    const pool = await deployPoolFromAsset({
      api,
      signer: deployer,
      args: [token.address, controller.address, rateModel.address],
      token,
    })
    const users = [bob, charlie]

    // initialize
    await controller.tx.setPriceOracle(priceOracle.address)
    //// for pool
    await controller.tx.supportMarket(pool.address)
    await priceOracle.tx.setFixedPrice(token.address, ONE_ETHER)
    await controller.tx.setCollateralFactorMantissa(
      pool.address,
      toParam(ONE_ETHER.mul(new BN(90)).div(new BN(100))),
    )

    return {
      api,
      deployer,
      token,
      pool,
      rateModel,
      controller,
      priceOracle,
      users,
    }
  }

  it('instantiate', async () => {
    const { token, pool, controller } = await setup()
    expect(pool.address).not.toBe(zeroAddress)
    expect((await pool.query.underlying()).value.ok).toEqual(token.address)
    expect((await pool.query.controller()).value.ok).toEqual(controller.address)
    expect(hexToUtf8((await pool.query.tokenName()).value.ok)).toEqual(
      'Starlay Dai Stablecoin',
    )
    expect(hexToUtf8((await pool.query.tokenSymbol()).value.ok)).toEqual('sDAI')
    expect((await pool.query.tokenDecimals()).value.ok).toEqual(8)
  })

  describe('.mint', () => {
    let deployer: KeyringPair
    let token: PSP22Token
    let pool: Pool

    beforeAll(async () => {
      ;({ deployer, token, pool } = await setup())
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
      ;({ deployer, token, pool } = await setup())
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
      ;({ deployer, token, pool } = await setup())
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
      const { pool } = await setup()
      const {
        value: { ok: cash },
      } = await pool.query.getCashPrior()
      const { value } = await pool.query.redeemUnderlying(cash.toNumber() + 1)
      expect(value.ok.err).toHaveProperty('redeemTransferOutNotPossible')
    })
  })

  describe('.borrow', () => {
    let deployer: KeyringPair
    let token: PSP22Token
    let pool: Pool
    let users: KeyringPair[]

    beforeAll(async () => {
      ;({ deployer, token, pool, users } = await setup())
    })

    it('preparations', async () => {
      await token.tx.mint(deployer.address, 10_000)
      await token.tx.approve(pool.address, 10_000)
      await pool.tx.mint(10_000)
      expect(
        (await pool.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(10_000)
    })

    it('execute', async () => {
      const [user1, user2] = users
      const { events: events1 } = await pool.withSigner(user1).tx.borrow(3_000)

      expect(
        (await token.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(0)
      expect(
        (await token.query.balanceOf(user1.address)).value.ok.toNumber(),
      ).toEqual(3_000)
      expect(
        (await token.query.balanceOf(user2.address)).value.ok.toNumber(),
      ).toEqual(0)
      expect(
        (await token.query.balanceOf(pool.address)).value.ok.toNumber(),
      ).toEqual(7_000)
      expect(
        (await pool.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(10_000)
      const event1 = events1[0]
      expect(event1.name).toEqual('Borrow')
      expect(event1.args.borrower).toEqual(user1.address)
      expect(event1.args.borrowAmount.toNumber()).toEqual(3_000)
      expect(event1.args.accountBorrows.toNumber()).toEqual(3_000)
      expect(event1.args.totalBorrows.toNumber()).toEqual(3_000)

      const { events: events2 } = await pool.withSigner(user2).tx.borrow(2_500)

      expect(
        (await token.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(0)
      expect(
        (await token.query.balanceOf(user1.address)).value.ok.toNumber(),
      ).toEqual(3_000)
      expect(
        (await token.query.balanceOf(user2.address)).value.ok.toNumber(),
      ).toEqual(2_500)
      expect(
        (await token.query.balanceOf(pool.address)).value.ok.toNumber(),
      ).toEqual(4_500)
      expect(
        (await pool.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(10_000)
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
      const { pool } = await setup()
      const { value } = await pool.query.borrow(3_000)
      expect(value.ok.err).toStrictEqual({ borrowCashNotAvailable: null })
    })
  })

  describe('.repay_borrow', () => {
    let deployer: KeyringPair
    let token: PSP22Token
    let pool: Pool
    let users: KeyringPair[]

    beforeAll(async () => {
      ;({ deployer, token, pool, users } = await setup())
    })

    it('preparations', async () => {
      await token.tx.mint(deployer.address, 10_000)
      await token.tx.approve(pool.address, 10_000)
      await pool.tx.mint(10_000)
      expect(
        (await pool.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(10_000)

      const [user1, _] = users
      await pool.withSigner(user1).tx.borrow(10_000)
      expect(
        (await token.query.balanceOf(user1.address)).value.ok.toNumber(),
      ).toEqual(10_000)
    })

    it('execute', async () => {
      const [user1, _] = users
      await token.withSigner(user1).tx.approve(pool.address, 4_500)
      const { events } = await pool.withSigner(user1).tx.repayBorrow(4_500)

      expect(
        (await token.query.balanceOf(user1.address)).value.ok.toNumber(),
      ).toEqual(5_500)
      expect(
        (await token.query.balanceOf(pool.address)).value.ok.toNumber(),
      ).toEqual(4_500)

      const event = events[0]
      expect(event.name).toEqual('RepayBorrow')
      expect(event.args.payer).toEqual(user1.address)
      expect(event.args.borrower).toEqual(user1.address)
      expect(event.args.repayAmount.toNumber()).toEqual(4_500)
      expect(event.args.accountBorrows.toNumber()).toEqual(5_500)
      expect(event.args.totalBorrows.toNumber()).toEqual(5_500)
    })
  })

  it('.repay_borrow_behalf', async () => {
    const { pool, users } = await setup()
    const { value } = await pool
      .withSigner(users[0])
      .query.repayBorrowBehalf(users[1].address, 0)
    expect(value.ok.err).toStrictEqual({ notImplemented: null })
  })

  describe('.liquidate_borrow', () => {
    // TODO: check seize
    let deployer: KeyringPair
    let token: PSP22Token
    let pool: Pool
    let users: KeyringPair[]
    let secondToken: PSP22Token
    let secondPool: Pool

    beforeAll(async () => {
      let api
      let controller
      ;({ api, deployer, controller, token, pool, users } = await setup())
      secondToken = await deployPSP22Token({
        api: api,
        signer: deployer,
        args: [
          0,
          'Dai Stablecoin' as unknown as string[],
          'DAI' as unknown as string[],
          8,
        ],
      })

      secondPool = await deployPoolFromAsset({
        api,
        signer: deployer,
        args: [secondToken.address, secondToken.address, zeroAddress],
        token: secondToken,
      })

      // initialize
      await controller.tx.supportMarket(secondPool.address)
    })

    it('preparations', async () => {
      await token.tx.mint(deployer.address, 10_000)
      await token.tx.approve(pool.address, 10_000)
      await pool.tx.mint(10_000)
      expect(
        (await pool.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(10_000)

      const [borrower, repayer] = users
      await pool.withSigner(borrower).tx.borrow(10_000)
      await token.tx.mint(repayer.address, 10_000)
      expect(
        (await token.query.balanceOf(repayer.address)).value.ok.toNumber(),
      ).toEqual(10_000)
      expect(
        (await token.query.balanceOf(borrower.address)).value.ok.toNumber(),
      ).toEqual(10_000)
      expect(
        (
          await pool.query.borrowBalanceStored(borrower.address)
        ).value.ok.toNumber(),
      ).toEqual(10_000)
    })

    // TODO: fix
    it.skip('execute', async () => {
      const [borrower, repayer] = users
      await token.withSigner(repayer).tx.approve(pool.address, 10_000)
      const { events } = await pool
        .withSigner(repayer)
        .tx.liquidateBorrow(borrower.address, 10_000, secondPool.address)
      expect(
        (await token.query.balanceOf(repayer.address)).value.ok.toNumber(),
      ).toEqual(0)
      expect(
        (await token.query.balanceOf(borrower.address)).value.ok.toNumber(),
      ).toEqual(10_000)
      expect(
        (
          await pool.query.borrowBalanceStored(borrower.address)
        ).value.ok.toNumber(),
      ).toEqual(0)

      expect(events[0].name).toEqual('RepayBorrow')
      const event = events[1]
      expect(event.name).toEqual('LiquidateBorrow')
      expect(event.args.liquidator).toEqual(repayer.address)
      expect(event.args.borrower).toEqual(borrower.address)
      expect(event.args.repayAmount.toNumber()).toEqual(10_000)
      expect(event.args.tokenCollateral).toEqual(secondPool.address)
      expect(event.args.seizeTokens.toNumber()).toEqual(0)
    })
  })

  describe.skip('.liquidate_borrow (fail case)', () => {
    const setup_extended = async () => {
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
      const { pool, users, secondPool } = await setup_extended()
      const [user1] = users
      const { value } = await pool
        .withSigner(user1)
        .query.liquidateBorrow(user1.address, 0, secondPool.address)
      expect(value.ok.err).toStrictEqual({
        liquidateLiquidatorIsBorrower: null,
      })
    })
    it('when repay_amount is zero', async () => {
      const { pool, users, secondPool } = await setup_extended()
      const [user1, user2] = users
      const { value } = await pool
        .withSigner(user1)
        .query.liquidateBorrow(user2.address, 0, secondPool.address)
      expect(value.ok.err).toStrictEqual({
        liquidateCloseAmountIsZero: null,
      })
    })
  })

  it.todo('.reduceReserves')
})
