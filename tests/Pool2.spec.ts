/* eslint-disable dot-notation */
import { ReturnNumber } from '@727-ventures/typechain-types'
import type { KeyringPair } from '@polkadot/keyring/types'
import { BN, BN_ONE, BN_TEN } from '@polkadot/util'
import { ONE_ETHER } from '../scripts/helper/constants'
import {
  deployController,
  deployDefaultInterestRateModel,
  deployIncentivesController,
  deployPSP22Token,
  deployPoolFromAsset,
  deployPriceOracle,
} from '../scripts/helper/deploy_helper'
import { getGasLimit } from '../scripts/helper/utils'
import { SUPPORTED_TOKENS } from '../scripts/tokens'
import DefaultInterestRateModel from '../types/contracts/default_interest_rate_model'
import {
  Redeem,
  ReserveUsedAsCollateralDisabled,
} from '../types/event-types/pool'
import { Transfer } from '../types/event-types/psp22_token'
import {
  PoolContracts,
  Pools,
  preparePoolsWithPreparedTokens,
} from './testContractHelper'
import {
  expectToEmit,
  mantissa,
  shouldNotRevert,
  toDec18,
  toDec6,
} from './testHelpers'

const MAX_CALL_WEIGHT = new BN(125_000_000_000).isub(BN_ONE).mul(BN_TEN)
const PROOFSIZE = new BN(2_000_000)
describe('Pool spec 2', () => {
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

  describe('.transfer', () => {
    const assertAccountLiquidity = (
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
      const {
        api,
        deployer,
        controller,
        rateModel,
        priceOracle,
        users,
        gasLimit,
        incentivesController,
      } = await setup()
      const { dai, usdc } = await preparePoolsWithPreparedTokens({
        api,
        controller,
        rateModel,
        manager: deployer,
        incentivesController,
      })
      const [userA, userB] = users

      // prerequisite
      //// initialize
      const toParam = (m: BN) => [m.toString()]
      for (const sym of [dai, usdc]) {
        await priceOracle.tx.setFixedPrice(sym.token.address, ONE_ETHER)
        await controller.tx.supportMarketWithCollateralFactorMantissa(
          sym.pool.address,
          sym.token.address,
          toParam(ONE_ETHER.mul(new BN(90)).div(new BN(100))),
        )
      }
      //// use protocol
      for await (const { user, sym, amount } of [
        {
          user: userA,
          sym: dai,
          amount: toDec18(500_000),
        },
        {
          user: userB,
          sym: usdc,
          amount: toDec6(500_000),
        },
      ]) {
        const { pool, token } = sym
        await token.withSigner(deployer).tx.mint(user.address, amount)
        await token.withSigner(user).tx.approve(pool.address, amount)
        await pool.withSigner(user).tx.mint(amount)
      }
      expect(
        (await dai.pool.query.balanceOf(userA.address)).value.ok.toString(),
      ).toBe(toDec18(500_000).toString())
      expect(
        (await usdc.pool.query.balanceOf(userB.address)).value.ok.toString(),
      ).toBe(toDec6(500_000).toString())

      {
        const { events } = await shouldNotRevert(
          dai.pool.withSigner(userA),
          'transfer',
          [userB.address, toDec18(100_000), [], { gasLimit }],
        )
        // assertions
        expect(
          (await dai.pool.query.balanceOf(userA.address)).value.ok.toString(),
        ).toBe(toDec18(400_000).toString())
        expect(
          (await dai.pool.query.balanceOf(userB.address)).value.ok.toString(),
        ).toBe(toDec18(100_000).toString())
        //// check event
        expect(events).toHaveLength(1)

        expect(events[0].name).toEqual('Transfer')
        expect(events[0].args.from).toEqual(userA.address)
        expect(events[0].args.to).toEqual(userB.address)
        expect(events[0].args.value.toString()).toEqual(
          toDec18(100_000).toString(),
        )

        await shouldNotRevert(
          dai.pool.withSigner(userB),
          'setUseReserveAsCollateral',
          [true],
        )
        //// check account_liquidity
        assertAccountLiquidity(
          (await controller.query.getAccountLiquidity(userA.address)).value.ok
            .ok,
          {
            collateral: (400_000 * 90) / 100,
            shortfall: 0,
          },
        )
        assertAccountLiquidity(
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
          [userA.address, toDec6(200_000), []],
        )
        await shouldNotRevert(
          usdc.pool.withSigner(userA),
          'setUseReserveAsCollateral',
          [true],
        )
        // assertions
        expect(
          (await usdc.pool.query.balanceOf(userA.address)).value.ok.toString(),
        ).toBe(toDec6(200_000).toString())
        expect(
          (await usdc.pool.query.balanceOf(userB.address)).value.ok.toString(),
        ).toBe(toDec6(300_000).toString())
        expect(events).toHaveLength(1)
        //// check event
        const event = events[0]
        expect(event.name).toEqual('Transfer')
        expect(event.args.from).toEqual(userB.address)
        expect(event.args.to).toEqual(userA.address)
        expect(event.args.value.toString()).toEqual(toDec6(200_000).toString())

        //// check account_liquidity
        assertAccountLiquidity(
          (await controller.query.getAccountLiquidity(userA.address)).value.ok
            .ok,
          {
            collateral: ((400_000 + 200_000) * 90) / 100,
            shortfall: 0,
          },
        )
        assertAccountLiquidity(
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
      const {
        api,
        deployer,
        controller,
        rateModel,
        priceOracle,
        users,
        gasLimit,
        incentivesController,
      } = await setup()
      const { dai, usdc } = await preparePoolsWithPreparedTokens({
        api,
        controller,
        rateModel,
        manager: deployer,
        incentivesController,
      })
      const [userA, userB] = users

      // prerequisite
      //// initialize
      const toParam = (m: BN) => [m.toString()]
      for (const sym of [dai, usdc]) {
        await priceOracle.tx.setFixedPrice(sym.token.address, ONE_ETHER)
        await controller.tx.supportMarketWithCollateralFactorMantissa(
          sym.pool.address,
          sym.token.address,
          toParam(ONE_ETHER.mul(new BN(90)).div(new BN(100))),
        )
      }
      //// use protocol
      for await (const { user, sym, amount } of [
        {
          user: userA,
          sym: dai,
          amount: toDec18(0.05),
        },
        {
          user: userB,
          sym: usdc,
          amount: toDec6(0.05),
        },
      ]) {
        const { pool, token } = sym
        await token.withSigner(deployer).tx.mint(user.address, amount)
        await token.withSigner(user).tx.approve(pool.address, amount)
        await pool.withSigner(user).tx.mint(amount, { gasLimit })
      }

      // case: src == dst
      {
        const res = await dai.pool
          .withSigner(userA)
          .query.transfer(userA.address, new BN(1), [])
        expect(res.value.ok.err.custom).toBe('TransferNotAllowed')
      }
      // case: shortfall of account_liquidity

      {
        await shouldNotRevert(usdc.pool.withSigner(userA), 'borrow', [
          toDec6(0.02),
          { gasLimit },
        ])
        const res = await dai.pool
          .withSigner(userA)
          .query.transfer(userB.address, new BN(2), []) // temp: truncated if less than 1 by collateral_factor?
        expect(res.value.ok.err.custom).toBe('InsufficientLiquidity')
      }
      // // case: paused
      await controller.tx.setTransferGuardianPaused(true)
      {
        const res = await dai.pool
          .withSigner(userA)
          .query.transfer(userB.address, new BN(1), [])
        expect(res.value.ok.err.custom).toBe('TransferIsPaused')
      }
    })
  })

  describe('.transfer_from', () => {
    it('success', async () => {
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
      const [userA, userB] = users
      const spender = deployer

      // prerequisite
      //// initialize
      const toParam = (m: BN) => [m.toString()]
      for (const sym of [dai, usdc]) {
        await priceOracle.tx.setFixedPrice(sym.token.address, ONE_ETHER)
        await controller.tx.supportMarketWithCollateralFactorMantissa(
          sym.pool.address,
          sym.token.address,
          toParam(ONE_ETHER.mul(new BN(90)).div(new BN(100))),
        )
      }
      //// use protocol
      for await (const { user, sym, amount } of [
        {
          user: userA,
          sym: dai,
          amount: toDec18(500_000),
        },
        {
          user: userB,
          sym: usdc,
          amount: toDec6(500_000),
        },
      ]) {
        const { pool, token } = sym
        await token.withSigner(deployer).tx.mint(user.address, amount)
        await token.withSigner(user).tx.approve(pool.address, amount)
        await pool.withSigner(user).tx.mint(amount)
      }
      expect(
        (await dai.pool.query.balanceOf(userA.address)).value.ok.toString(),
      ).toBe(toDec18(500_000).toString())
      expect(
        (await usdc.pool.query.balanceOf(userB.address)).value.ok.toString(),
      ).toBe(toDec6(500_000).toString())

      {
        // approve
        const { events: approveEvents } = await shouldNotRevert(
          dai.pool.withSigner(userA),
          'approve',
          [spender.address, toDec18(100_000)],
        )
        //// assertions
        expect(
          (
            await dai.pool.query.allowance(userA.address, spender.address)
          ).value.ok.toString(),
        ).toBe(toDec18(100_000).toString())
        ////// check event
        expect(approveEvents).toHaveLength(1)
        const event = approveEvents[0]
        expect(event.name).toEqual('Approval')
        expect(event.args.owner).toEqual(userA.address)
        expect(event.args.spender).toEqual(spender.address)
        expect(event.args.value.toString()).toEqual(toDec18(100_000).toString())

        // transfer_from
        const { events: transferFromEvents } = await shouldNotRevert(
          dai.pool.withSigner(spender),
          'transferFrom',
          [userA.address, userB.address, toDec18(100_000), []],
        )
        //// assertions
        expect(
          (
            await dai.pool.query.allowance(userA.address, spender.address)
          ).value.ok.toString(),
        ).toBe('0')
        expect(
          (await dai.pool.query.balanceOf(userA.address)).value.ok.toString(),
        ).toBe(toDec18(400_000).toString())
        expect(
          (await dai.pool.query.balanceOf(userB.address)).value.ok.toString(),
        ).toBe(toDec18(100_000).toString())
        //// check event
        expect(transferFromEvents).toHaveLength(2)
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
          toDec18(100_000).toString(),
        )
      }
    })
    it('failure', async () => {
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
      const [userA, userB] = users
      const spender = deployer

      // prerequisite
      //// initialize
      const toParam = (m: BN) => [m.toString()]
      for (const sym of [dai, usdc]) {
        await priceOracle.tx.setFixedPrice(sym.token.address, ONE_ETHER)
        await controller.tx.supportMarketWithCollateralFactorMantissa(
          sym.pool.address,
          sym.token.address,
          toParam(ONE_ETHER.mul(new BN(90)).div(new BN(100))),
        )
      }
      //// use protocol
      for await (const { user, sym, amount } of [
        {
          user: userA,
          sym: dai,
          amount: toDec18(500_000),
        },
        {
          user: userB,
          sym: usdc,
          amount: toDec6(500_000),
        },
      ]) {
        const { pool, token } = sym
        await token.withSigner(deployer).tx.mint(user.address, amount)
        await token.withSigner(user).tx.approve(pool.address, amount)
        await pool.withSigner(user).tx.mint(amount)
      }
      expect(
        (await dai.pool.query.balanceOf(userA.address)).value.ok.toString(),
      ).toBe(toDec18(500_000).toString())
      expect(
        (await usdc.pool.query.balanceOf(userB.address)).value.ok.toString(),
      ).toBe(toDec6(500_000).toString())

      // case: shortage of approval
      {
        await shouldNotRevert(dai.pool.withSigner(userA), 'approve', [
          spender.address,
          toDec18(99_999),
        ])
        const res = await dai.pool
          .withSigner(spender)
          .query.transferFrom(
            userA.address,
            userB.address,
            toDec18(100_000),
            [],
          )
        expect(res.value.ok.err).toStrictEqual({ insufficientAllowance: null })
      }
      await shouldNotRevert(dai.pool.withSigner(userA), 'approve', [
        spender.address,
        toDec18(100_000),
      ])
      // case: src == dst
      {
        const res = await dai.pool
          .withSigner(spender)
          .query.transferFrom(userA.address, userA.address, new BN(1), [])
        expect(res.value.ok.err.custom).toBe('TransferNotAllowed')
      }
      // case: shortfall of account_liquidity
      {
        await usdc.pool.withSigner(userA).tx.borrow(toDec6(450_000))
        const res = await dai.pool
          .withSigner(spender)
          .query.transferFrom(userA.address, userB.address, new BN(2), []) // temp: truncated if less than 1 by collateral_factor?
        expect(res.value.ok.err.custom).toBe('InsufficientLiquidity')
      }
      // case: paused
      await controller.tx.setTransferGuardianPaused(true)
      {
        const res = await dai.pool
          .withSigner(spender)
          .query.transferFrom(userA.address, userB.address, new BN(1), [])
        expect(res.value.ok.err.custom).toBe('TransferIsPaused')
      }
    })
  })

  describe('.exchange_rate_stored', () => {
    describe('success', () => {
      it('return initial_exchange_rate_stored if no total_supply', async () => {
        const { api, deployer, controller, rateModel, incentivesController } =
          await setup()

        const newToken = await deployPSP22Token({
          api: api,
          signer: deployer,
          args: [0, 'Sample Coin', 'COIN', 8],
        })
        const initialExchangeRate = ONE_ETHER
        const newPool = await deployPoolFromAsset({
          api: api,
          signer: deployer,
          args: [
            incentivesController.address,
            newToken.address,
            controller.address,
            rateModel.address,
            [initialExchangeRate.toString()],
            10000,
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
          (await newPool.query.exchangeRateStored()).value.ok.toString(),
        ).toBe(initialExchangeRate.toString())
      })
    })
  })

  describe('.interest_rate_model', () => {
    const setupExtended = async () => {
      const {
        api,
        deployer,
        controller,
        users,
        priceOracle,
        pools,
        gasLimit,
        incentivesController,
      } = await setup()

      const token = await deployPSP22Token({
        api: api,
        signer: deployer,
        args: [0, 'Dai Stablecoin', 'DAI', 8],
      })
      await priceOracle.tx.setFixedPrice(token.address, ONE_ETHER)
      const dai = SUPPORTED_TOKENS.dai
      const rateModel = await deployDefaultInterestRateModel({
        api,
        signer: deployer,
        args: [
          [dai.rateModel.baseRatePerYear()],
          [dai.rateModel.multiplierPerYearSlope1()],
          [dai.rateModel.multiplierPerYearSlope2()],
          [dai.rateModel.kink()],
        ],
      })

      const pool = await deployPoolFromAsset({
        api,
        signer: deployer,
        args: [
          incentivesController.address,
          token.address,
          controller.address,
          rateModel.address,
          [ONE_ETHER.toString()],
          10000,
        ],
        token: token,
      })
      await controller.tx.supportMarketWithCollateralFactorMantissa(
        pool.address,
        token.address,
        [dai.riskParameter.collateralFactor],
      )
      return { users, api, pools, deployer, controller, pool, token, gasLimit }
    }
    describe('on DAI Stablecoin', () => {
      it('if the utilization rate is 10% then the borrowing interest rate should be about 0.44%', async () => {
        const { pool, users, token, gasLimit } = await setupExtended()
        const [alice] = users
        const deposit = 1000
        const borrow = 100
        await token.withSigner(alice).tx.mint(alice.address, deposit)
        await token.withSigner(alice).tx.approve(pool.address, deposit)
        await pool.withSigner(alice).tx.mint(deposit, { gasLimit })
        await pool.withSigner(alice).tx.borrow(borrow, { gasLimit })
        const borrowRate = new BN(
          (await pool.query.borrowRatePerMsec()).value.ok.toNumber(),
        )
        const msecPerYear = new BN(365 * 24 * 60 * 60 * 1000)

        expect(borrowRate.mul(msecPerYear).toString()).toBe('4444431552000000')
      })
      it('if the utilization rate is 95% then the borrowing interest rate should be about 34%', async () => {
        const { pool, users, pools, token, gasLimit } = await setupExtended()
        const [alice, bob] = users
        const otherPool = pools.usdt
        const deposit = 1000
        const borrow = 950
        await token.withSigner(alice).tx.mint(alice.address, deposit)
        await token.withSigner(alice).tx.approve(pool.address, deposit)
        await pool.withSigner(alice).tx.mint(deposit, { gasLimit })
        await otherPool.token.withSigner(bob).tx.mint(bob.address, ONE_ETHER)
        await otherPool.token
          .withSigner(bob)
          .tx.approve(otherPool.pool.address, ONE_ETHER)
        await otherPool.pool.withSigner(bob).tx.mint(ONE_ETHER, { gasLimit })
        await pool.withSigner(bob).tx.borrow(borrow, { gasLimit })
        const borrowRate = new BN(
          await (await pool.query.borrowRatePerMsec()).value.ok.toNumber(),
        )
        const msecPerYear = new BN(365 * 24 * 60 * 60 * 1000)

        expect(borrowRate.mul(msecPerYear).toString()).toBe(
          '339999959808000000',
        )
      })
      it('borrow balance should grow up', async () => {
        const { pool, users, pools, token, gasLimit } = await setupExtended()
        const [alice, bob] = users
        const otherPool = pools.usdt
        const deposit = new BN(1000).mul(BN_TEN.pow(new BN(8)))
        const borrow = new BN(950).mul(BN_TEN.pow(new BN(8)))
        await token.withSigner(alice).tx.mint(alice.address, deposit)
        await token.withSigner(alice).tx.approve(pool.address, deposit)
        await pool.withSigner(alice).tx.mint(deposit, { gasLimit })
        await otherPool.token
          .withSigner(bob)
          .tx.mint(bob.address, ONE_ETHER.toString())
        await otherPool.token
          .withSigner(bob)
          .tx.approve(otherPool.pool.address, ONE_ETHER.toString())
        await otherPool.pool.withSigner(bob).tx.mint(ONE_ETHER, { gasLimit })
        await pool.withSigner(bob).tx.borrow(borrow, { gasLimit })
        const balance1 = (await pool.query.borrowBalanceCurrent(bob.address))
          .value.ok
        // wait 2 sec
        await new Promise((resolve) => setTimeout(resolve, 2000))
        await pool.tx.accrueInterest({ gasLimit })
        const balance2 = (await pool.query.borrowBalanceCurrent(bob.address))
          .value.ok
        expect(balance2.ok.toNumber()).toBeGreaterThan(balance1.ok.toNumber())
      })
      it('in repay_borrow_all, a user should repay all borrow balance including accrued interest', async () => {
        const { pool, users, pools, token, gasLimit } = await setupExtended()
        const [alice, bob] = users
        const otherPool = pools.usdt
        const deposit = new BN(1000).mul(BN_TEN.pow(new BN(8)))
        const borrow = new BN(950).mul(BN_TEN.pow(new BN(8)))
        await token.withSigner(alice).tx.mint(alice.address, deposit)
        await token.withSigner(alice).tx.approve(pool.address, deposit)
        await pool.withSigner(alice).tx.mint(deposit, { gasLimit })
        await otherPool.token
          .withSigner(bob)
          .tx.mint(bob.address, ONE_ETHER.toString())
        await otherPool.token
          .withSigner(bob)
          .tx.approve(otherPool.pool.address, ONE_ETHER.toString())
        await otherPool.pool.withSigner(bob).tx.mint(ONE_ETHER, { gasLimit })
        await pool.withSigner(bob).tx.borrow(borrow, { gasLimit })
        // wait 2 sec
        await new Promise((resolve) => setTimeout(resolve, 2000))

        await token.withSigner(bob).tx.mint(bob.address, deposit)
        await token
          .withSigner(bob)
          .tx.approve(pool.address, ONE_ETHER.mul(new BN(100)).toString())
        await pool.tx.accrueInterest({ gasLimit })
        const tx = await pool.withSigner(bob).tx.repayBorrowAll({ gasLimit })
        expect(BigInt(tx.events[0].args['repayAmount'])).toBeGreaterThan(
          borrow.toNumber(),
        )
        await pool.tx.accrueInterest({ gasLimit })
        expect(
          (
            await pool.query.borrowBalanceCurrent(bob.address)
          ).value.ok.ok.toNumber(),
        ).toBe(0)
      })
      it('all the borrowed amount can be repayable', async () => {
        const { pool, users, token: dai, gasLimit } = await setupExtended()
        const [alice] = users
        const decimals = SUPPORTED_TOKENS.dai.decimals
        const deposit = new BN(90).mul(BN_TEN.pow(new BN(decimals)))
        const borrowAmount1 = new BN(50).mul(BN_TEN.pow(new BN(decimals)))
        const borrowAmount2 = new BN(22).mul(BN_TEN.pow(new BN(decimals)))
        const repayAmount = new BN(72).mul(BN_TEN.pow(new BN(decimals)))
        const wait = () => new Promise((resolve) => setTimeout(resolve, 100))
        await dai
          .withSigner(alice)
          .tx.mint(alice.address, deposit.mul(new BN(1000000)))
        await dai.withSigner(alice).tx.approve(pool.address, deposit)
        // deposit 90 DAI
        await pool.withSigner(alice).tx.mint(deposit, { gasLimit })
        const totalBorrows0 = (await pool.query.totalBorrows()).value.ok
        expect(totalBorrows0.toNumber()).toBe(0)
        // borrow 50 DAI
        await wait()
        await pool.tx.accrueInterest({ gasLimit })
        await pool.withSigner(alice).tx.borrow(borrowAmount1, { gasLimit })
        expect((await pool.query.totalBorrows()).value.ok.toString()).not.toBe(
          '0',
        )
        // borrow 22 DAI
        await wait()
        await pool.tx.accrueInterest({ gasLimit })
        await pool.withSigner(alice).tx.borrow(borrowAmount2, { gasLimit })
        expect((await pool.query.totalBorrows()).value.ok.toString()).not.toBe(
          '0',
        )
        // repay 72 DAI
        await wait()
        await pool.tx.accrueInterest({ gasLimit })
        await dai
          .withSigner(alice)
          .tx.approve(pool.address, ONE_ETHER.mul(new BN(1000)))
        await pool.withSigner(alice).tx.repayBorrow(repayAmount, { gasLimit })
        expect((await pool.query.totalBorrows()).value.ok.toString()).not.toBe(
          '0',
        )
        // repay all
        await wait()
        await pool.tx.accrueInterest({ gasLimit })
        await dai
          .withSigner(alice)
          .tx.approve(pool.address, ONE_ETHER.mul(new BN(1000)))
        await pool.withSigner(alice).tx.repayBorrowAll({ gasLimit })

        // confirmations
        await wait()
        await pool.tx.accrueInterest({ gasLimit })
        expect(
          (
            await pool.query.borrowBalanceCurrent(alice.address)
          ).value.ok.ok.toString(),
        ).toBe('0')
        expect((await pool.query.totalBorrows()).value.ok.toString()).toBe('0')
      })
    })
  })

  describe('.redeem with liquidation_threshold', () => {
    let deployer: KeyringPair
    let users: KeyringPair[]
    let dai: PoolContracts
    let usdc: PoolContracts
    let usdt: PoolContracts
    let pools: Pools

    beforeAll(async () => {
      ;({ deployer, pools, users } = await setup())
      ;({ dai, usdt, usdc } = pools)
    })

    const deployerDaiDeposited = 10_000
    const user0DaiDeposited = 20_000
    const daiMinted = deployerDaiDeposited + user0DaiDeposited
    it('preparations - deposit DAI', async () => {
      await shouldNotRevert(dai.token, 'mint', [
        deployer.address,
        deployerDaiDeposited,
      ])
      await shouldNotRevert(dai.token, 'approve', [
        dai.pool.address,
        deployerDaiDeposited,
      ])
      await shouldNotRevert(dai.pool, 'mint', [deployerDaiDeposited])

      await shouldNotRevert(dai.token, 'mint', [
        users[0].address,
        user0DaiDeposited,
      ])
      await shouldNotRevert(dai.token.withSigner(users[0]), 'approve', [
        dai.pool.address,
        user0DaiDeposited,
      ])
      await shouldNotRevert(dai.pool.withSigner(users[0]), 'mint', [
        user0DaiDeposited,
      ])

      expect(
        (await dai.pool.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(deployerDaiDeposited)

      expect(
        (await dai.pool.query.balanceOf(users[0].address)).value.ok.toNumber(),
      ).toEqual(user0DaiDeposited)

      expect(
        (await dai.pool.query.exchangeRateCurrent()).value.ok.ok.toString(),
      ).toBe(ONE_ETHER.toString())
    })

    const deployerUsdcDeposited = 20_000
    const user0UsdcDeposited = 10_000
    // const usdcMinted = deployerUsdcDeposited + user0UsdcDeposited
    it('preparations - deposit USDC', async () => {
      await shouldNotRevert(usdc.token, 'mint', [
        deployer.address,
        deployerUsdcDeposited,
      ])
      await shouldNotRevert(usdc.token, 'approve', [
        usdc.pool.address,
        deployerUsdcDeposited,
      ])
      await shouldNotRevert(usdc.pool, 'mint', [deployerUsdcDeposited])

      await shouldNotRevert(usdc.token, 'mint', [
        users[0].address,
        user0UsdcDeposited,
      ])
      await shouldNotRevert(usdc.token.withSigner(users[0]), 'approve', [
        usdc.pool.address,
        user0UsdcDeposited,
      ])
      await shouldNotRevert(usdc.pool.withSigner(users[0]), 'mint', [
        user0UsdcDeposited,
      ])

      expect(
        (await usdc.pool.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(deployerUsdcDeposited)

      expect(
        (await usdc.pool.query.balanceOf(users[0].address)).value.ok.toNumber(),
      ).toEqual(user0UsdcDeposited)

      expect(
        (await usdc.pool.query.exchangeRateCurrent()).value.ok.ok.toString(),
      ).toBe(ONE_ETHER.toString())
    })

    const deployerUsdtDeposited = 20_000
    const user0UsdtDeposited = 20_000
    // const usdtMinted = deployerUsdtDeposited + user0UsdtDeposited
    it('preparations - deposit USDT', async () => {
      await shouldNotRevert(usdt.token, 'mint', [
        deployer.address,
        deployerUsdtDeposited,
      ])
      await shouldNotRevert(usdt.token, 'approve', [
        usdt.pool.address,
        deployerUsdtDeposited,
      ])
      await shouldNotRevert(usdt.pool, 'mint', [deployerUsdtDeposited])

      await shouldNotRevert(usdt.token, 'mint', [
        users[0].address,
        user0UsdtDeposited,
      ])
      await shouldNotRevert(usdt.token.withSigner(users[0]), 'approve', [
        usdt.pool.address,
        user0UsdtDeposited,
      ])
      await shouldNotRevert(usdt.pool.withSigner(users[0]), 'mint', [
        user0UsdtDeposited,
      ])

      expect(
        (await usdt.pool.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(deployerUsdtDeposited)

      expect(
        (await usdt.pool.query.balanceOf(users[0].address)).value.ok.toNumber(),
      ).toEqual(user0UsdtDeposited)

      expect(
        (await usdt.pool.query.exchangeRateCurrent()).value.ok.ok.toString(),
      ).toBe(ONE_ETHER.toString())
    })

    const newLiquidationThreshold = 9500 // 80%
    it('preparations - set Liquidation Threshold', async () => {
      await shouldNotRevert(dai.pool, 'setLiquidationThreshold', [
        newLiquidationThreshold,
      ])
      expect(
        (await dai.pool.query.liquidationThreshold()).value.ok.toNumber(),
      ).toEqual(newLiquidationThreshold)

      await shouldNotRevert(usdc.pool, 'setLiquidationThreshold', [
        newLiquidationThreshold,
      ])

      expect(
        (await usdc.pool.query.liquidationThreshold()).value.ok.toNumber(),
      ).toEqual(newLiquidationThreshold)
      await shouldNotRevert(usdt.pool, 'setLiquidationThreshold', [
        newLiquidationThreshold,
      ])
      expect(
        (await usdt.pool.query.liquidationThreshold()).value.ok.toNumber(),
      ).toEqual(newLiquidationThreshold)
    })

    it('execute', async () => {
      const redeemAmount = 10_000
      const { events } = await shouldNotRevert(dai.pool, 'redeemUnderlying', [
        redeemAmount,
      ])

      expect(
        (await dai.token.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(redeemAmount)
      expect(
        (await dai.token.query.balanceOf(dai.pool.address)).value.ok.toNumber(),
      ).toEqual(daiMinted - redeemAmount)
      expect(
        (await dai.pool.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(deployerDaiDeposited - redeemAmount)

      expect(events).toHaveLength(3)
      expectToEmit<ReserveUsedAsCollateralDisabled>(
        events[0],
        'ReserveUsedAsCollateralDisabled',
        {
          user: deployer.address,
        },
      )
      expectToEmit<Transfer>(events[1], 'Transfer', {
        from: deployer.address,
        to: null,
        value: redeemAmount,
      })
      expectToEmit<Redeem>(events[2], 'Redeem', {
        redeemer: deployer.address,
        redeemAmount,
      })
    })
  })

  describe('.set_use_reserve_as_collateral', () => {
    let users: KeyringPair[]
    let dai: PoolContracts
    let usdt: PoolContracts
    let usdc: PoolContracts
    let pools: Pools

    beforeAll(async () => {
      ;({ pools, users } = await setup())
      ;({ dai, usdt, usdc } = pools)
    })

    it('User 0 Deposits 10_000 DAI, disables DAI as collateral', async () => {
      const depositAmount = 10_000
      await shouldNotRevert(dai.token, 'mint', [
        users[0].address,
        depositAmount,
      ])
      await shouldNotRevert(dai.token.withSigner(users[0]), 'approve', [
        dai.pool.address,
        depositAmount,
      ])
      await shouldNotRevert(dai.pool.withSigner(users[0]), 'mint', [
        depositAmount,
      ])

      await shouldNotRevert(
        dai.pool.withSigner(users[0]),
        'setUseReserveAsCollateral',
        [false],
      )

      const usingReserveAsCollateral = (
        await dai.pool.query.usingReserveAsCollateral(users[0].address)
      ).value.ok.valueOf()
      expect(usingReserveAsCollateral).toEqual(false)
    })

    it('User 1 Deposits 20_000 USDT, disables USDT as collateral, borrows 4000 DAI (revert expected)', async () => {
      const depositAmount = 20_000
      await shouldNotRevert(usdt.token, 'mint', [
        users[1].address,
        depositAmount,
      ])
      await shouldNotRevert(usdt.token.withSigner(users[1]), 'approve', [
        usdt.pool.address,
        depositAmount,
      ])
      await shouldNotRevert(usdt.pool.withSigner(users[1]), 'mint', [
        depositAmount,
      ])

      await shouldNotRevert(
        usdt.pool.withSigner(users[1]),
        'setUseReserveAsCollateral',
        [false],
      )

      const borrowAmount = 4_000
      expect(
        (await dai.pool.withSigner(users[1]).query.borrow(borrowAmount)).value
          .ok.err,
      ).toEqual({ controller: 'InsufficientLiquidity' })
    })

    it('User 1 enables USDT as collateral, borrows 4000 DAI', async () => {
      const borrowAmount = 4_000
      await shouldNotRevert(
        usdt.pool.withSigner(users[1]),
        'setUseReserveAsCollateral',
        [true],
      )

      await shouldNotRevert(dai.pool.withSigner(users[1]), 'borrow', [
        borrowAmount,
      ])
    })

    it('User 1 disables USDT as collateral (revert expected)', async () => {
      expect(
        (
          await usdt.pool
            .withSigner(users[1])
            .query.setUseReserveAsCollateral(false)
        ).value.ok.err,
      ).toEqual({ controller: 'BalanceDecreaseNotAllowed' })
    })

    it('User 1 Deposits 2000 USDC, disables USDT as collateral. Should revert as 2000 USDC are not enough to cover the debt (revert expected)', async () => {
      const depositAmount = 2_000
      await shouldNotRevert(usdc.token, 'mint', [
        users[1].address,
        depositAmount,
      ])
      await shouldNotRevert(usdc.token.withSigner(users[1]), 'approve', [
        usdc.pool.address,
        depositAmount,
      ])
      await shouldNotRevert(usdc.pool.withSigner(users[1]), 'mint', [
        depositAmount,
      ])

      expect(
        (
          await usdt.pool
            .withSigner(users[1])
            .query.setUseReserveAsCollateral(false)
        ).value.ok.err,
      ).toEqual({ controller: 'BalanceDecreaseNotAllowed' })
    })

    it('User 1 Deposits 2000 more USDC (enough to cover the DAI debt), disables USDT as collateral', async () => {
      const depositAmount = 2_000
      await shouldNotRevert(usdc.token, 'mint', [
        users[1].address,
        depositAmount,
      ])
      await shouldNotRevert(usdc.token.withSigner(users[1]), 'approve', [
        usdc.pool.address,
        depositAmount,
      ])
      await shouldNotRevert(usdc.pool.withSigner(users[1]), 'mint', [
        depositAmount,
      ])

      await shouldNotRevert(
        usdt.pool.withSigner(users[1]),
        'setUseReserveAsCollateral',
        [false],
      )
    })

    it('User 1 disables USDC as collateral (revert expected)', async () => {
      expect(
        (
          await usdc.pool
            .withSigner(users[1])
            .query.setUseReserveAsCollateral(false)
        ).value.ok.err,
      ).toEqual({ controller: 'BalanceDecreaseNotAllowed' })
    })

    it('User 1 withdraw USDT', async () => {
      await shouldNotRevert(
        usdt.pool.withSigner(users[1]),
        'redeemUnderlying',
        [20000],
      )
    })
  })
})
