import { encodeAddress } from '@polkadot/keyring'
import type { KeyringPair } from '@polkadot/keyring/types'
import { WeightV2 } from '@polkadot/types/interfaces'
import { BN, BN_ONE, BN_TEN } from '@polkadot/util'
import { ReturnNumber } from '@starlay-finance/typechain-types'
import { ONE_ETHER, ZERO_ADDRESS } from '../scripts/helper/constants'
import {
  deployController,
  deployDefaultInterestRateModel,
  deployIncentivesController,
  deployPriceOracle,
} from '../scripts/helper/deploy_helper'
import { getGasLimit } from '../scripts/helper/utils'
import Controller from '../types/contracts/controller'
import PriceOracle from '../types/contracts/price_oracle'
import {
  Pools,
  TEST_METADATAS,
  preparePoolWithMockToken,
  preparePoolsWithPreparedTokens,
} from './testContractHelper'
import { mantissa, shouldNotRevert, toDec18, toDec6 } from './testHelpers'

const MAX_CALL_WEIGHT = new BN(100_000_000_000).isub(BN_ONE).mul(BN_TEN)
const PROOFSIZE = new BN(2_000_000)
describe('Controller spec', () => {
  const setup = async () => {
    const { api, alice: deployer, bob, charlie } = globalThis.setup
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
    // temp: declare params for rate_model
    const rateModelArg = new BN(100).mul(ONE_ETHER)
    const rateModel = await deployDefaultInterestRateModel({
      api,
      signer: deployer,
      args: [[rateModelArg], [rateModelArg], [rateModelArg], [rateModelArg]],
    })

    const incentivesController = await deployIncentivesController({
      api,
      signer: deployer,
      args: [],
    })

    // initialize
    await controller.tx.setPriceOracle(priceOracle.address)

    return {
      api,
      deployer,
      controller,
      rateModel,
      priceOracle,
      gasLimit,
      users: [bob, charlie],
      incentivesController,
    }
  }

  const setupWithPools = async () => {
    const {
      api,
      deployer,
      rateModel,
      controller,
      priceOracle,
      users,
      gasLimit,
      incentivesController,
    } = await setup()

    const pools = await preparePoolsWithPreparedTokens({
      api,
      controller,
      rateModel,
      manager: deployer,
      incentivesController,
    })

    // initialize
    await controller.tx.setPriceOracle(priceOracle.address)
    //// for pool
    for (const sym of [pools.dai, pools.usdc, pools.usdt]) {
      await priceOracle.tx.setFixedPrice(sym.token.address, ONE_ETHER)
      await controller.tx.supportMarketWithCollateralFactorMantissa(
        sym.pool.address,
        sym.token.address,
        [ONE_ETHER.mul(new BN(75)).div(new BN(100))],
      )
    }

    return {
      api,
      deployer,
      rateModel,
      controller,
      priceOracle,
      pools,
      users,
      gasLimit,
    }
  }

  it('instantiate', async () => {
    const { controller, priceOracle } = await setup()
    const markets = (await controller.query.markets()).value.ok
    expect(markets.length).toBe(0)
    expect((await controller.query.oracle()).value.ok).toEqual(
      priceOracle.address,
    )
    const closeFactorMantissa = (await controller.query.closeFactorMantissa())
      .value.ok
    expect(closeFactorMantissa.toNumber()).toEqual(0)
    const liquidationIncentiveMantissa = (
      await controller.query.liquidationIncentiveMantissa()
    ).value.ok
    expect(liquidationIncentiveMantissa.toNumber()).toEqual(0)
  })

  describe('.mint_allowed', () => {
    it('check pause status', async () => {
      const { controller } = await setup()
      const poolAddr = encodeAddress(
        '0x0000000000000000000000000000000000000000000000000000000000000001',
      )
      await controller.tx.setMintGuardianPaused(poolAddr, true)
      const { value } = await controller.query.mintAllowed(
        poolAddr,
        ZERO_ADDRESS,
        0,
      )
      expect(value.ok.err).toBe('MintIsPaused')
    })
  })

  describe('.redeem_allowed', () => {
    it('check account liquidity', async () => {
      const {
        api,
        deployer,
        rateModel,
        controller,
        priceOracle,
        incentivesController,
      } = await setup()
      const usdc = await preparePoolWithMockToken({
        api,
        controller,
        rateModel,
        manager: deployer,
        metadata: TEST_METADATAS.usdc,
        incentivesController,
      })
      await priceOracle.tx.setFixedPrice(usdc.token.address, ONE_ETHER)
      await controller.tx.supportMarketWithCollateralFactorMantissa(
        usdc.pool.address,
        usdc.token.address,
        [ONE_ETHER.mul(new BN(90)).div(new BN(100))],
      )
      const { value } = await controller.query.redeemAllowed(
        usdc.pool.address,
        deployer.address,
        1,
        null,
      )
      expect(value.ok.err).toBe('InsufficientLiquidity')
    })
  })

  describe('.borrow_allowed', () => {
    it('check pause status', async () => {
      const {
        controller,
        pools: { dai },
      } = await setupWithPools()
      await controller.tx.setBorrowGuardianPaused(dai.pool.address, true)
      const { value } = await controller.query.borrowAllowed(
        dai.pool.address,
        ZERO_ADDRESS,
        0,
        null,
      )
      expect(value.ok.err).toBe('BorrowIsPaused')
    })
    it('check price error', async () => {
      const {
        api,
        deployer,
        rateModel,
        controller,
        priceOracle,
        incentivesController,
      } = await setup()
      const sampleCoin = await preparePoolWithMockToken({
        api,
        controller,
        rateModel,
        manager: deployer,
        metadata: {
          name: 'Sample',
          symbol: 'SAMPLE',
          decimals: 6,
        },
        incentivesController,
      })
      await controller.tx.supportMarket(
        sampleCoin.pool.address,
        sampleCoin.token.address,
      )

      const { value: value1 } = await controller.query.borrowAllowed(
        sampleCoin.pool.address,
        ZERO_ADDRESS,
        0,
        null,
      )
      expect(value1.ok.err).toBe('PriceError')

      await priceOracle.tx.setFixedPrice(sampleCoin.token.address, 0)
      const { value: value2 } = await controller.query.borrowAllowed(
        sampleCoin.pool.address,
        ZERO_ADDRESS,
        0,
        null,
      )
      expect(value2.ok.err).toBe('PriceError')
    })
    it('check borrow cap', async () => {
      const {
        controller,
        pools: { dai },
      } = await setupWithPools()
      await controller.tx.setBorrowCap(dai.pool.address, 1)

      const { value } = await controller.query.borrowAllowed(
        dai.pool.address,
        ZERO_ADDRESS,
        2,
        null,
      )
      expect(value.ok.err).toBe('BorrowCapReached')
    })
    it('check account liquidity', async () => {
      const {
        controller,
        pools: { dai },
      } = await setupWithPools()

      const { value } = await controller.query.borrowAllowed(
        dai.pool.address,
        ZERO_ADDRESS,
        1,
        null,
      )
      expect(value.ok.err).toBe('InsufficientLiquidity')
    })
  })

  describe('.repay_borrow_allowed', () => {
    it('do nothing', async () => {
      const { controller } = await setup()
      const { value } = await controller.query.repayBorrowAllowed(
        ZERO_ADDRESS,
        ZERO_ADDRESS,
        ZERO_ADDRESS,
        0,
      )
      expect(value.ok.ok).toBeNull()
    })
  })

  describe('.liquidate_borrow_allowed', () => {
    it('check listed markets', async () => {
      const {
        controller,
        pools: { dai, usdc },
      } = await setupWithPools()

      const { value: val1 } = await controller.query.liquidateBorrowAllowed(
        dai.pool.address,
        ZERO_ADDRESS,
        ZERO_ADDRESS,
        ZERO_ADDRESS,
        0,
        null,
      )
      expect(val1.ok.err).toBe('MarketNotListed')
      const { value: val2 } = await controller.query.liquidateBorrowAllowed(
        ZERO_ADDRESS,
        usdc.pool.address,
        ZERO_ADDRESS,
        ZERO_ADDRESS,
        0,
        null,
      )
      expect(val2.ok.err).toBe('MarketNotListed')
    })
    it('check account liquidity', async () => {
      const {
        controller,
        pools: { dai, usdc },
      } = await setupWithPools()
      const { value } = await controller.query.liquidateBorrowAllowed(
        dai.pool.address,
        usdc.pool.address,
        ZERO_ADDRESS,
        ZERO_ADDRESS,
        0,
        null,
      )
      expect(value.ok.err).toBe('InsufficientShortfall')
    })
    it('check too much repay', async () => {
      const {
        deployer,
        controller,
        pools: { dai, usdc },
        users: [borrower],
        gasLimit,
      } = await setupWithPools()

      // Prepares
      //// add liquidity to usdc pool
      await usdc.token.tx.mint(deployer.address, toDec6(10_000))
      await usdc.token.tx.approve(usdc.pool.address, toDec6(10_000))
      await usdc.pool.tx.mint(toDec6(10_000), { gasLimit })
      //// mint to dai pool for collateral
      await dai.token.tx.mint(borrower.address, toDec18(20_000))
      await dai.token
        .withSigner(borrower)
        .tx.approve(dai.pool.address, toDec18(20_000))
      await dai.pool.withSigner(borrower).tx.mint(toDec18(20_000), { gasLimit })
      //// borrow usdc
      await usdc.pool
        .withSigner(borrower)
        .tx.borrow(toDec6(10_000), { gasLimit })
      //// down collateral_factor for dai
      await controller.tx.setCollateralFactorMantissa(dai.pool.address, [
        new BN(1),
      ])

      const { value } = await controller.query.liquidateBorrowAllowed(
        usdc.pool.address,
        dai.pool.address,
        deployer.address,
        borrower.address,
        toDec6(10_000),
        null,
      )
      expect(value.ok.err).toBe('TooMuchRepay')
    })
  })

  describe('.seize_allowed', () => {
    it('check pause status', async () => {
      const { controller } = await setupWithPools()
      await controller.tx.setSeizeGuardianPaused(true)
      const { value } = await controller.query.seizeAllowed(
        ZERO_ADDRESS,
        ZERO_ADDRESS,
        ZERO_ADDRESS,
        ZERO_ADDRESS,
        0,
      )
      expect(value.ok.err).toBe('SeizeIsPaused')
    })
    it('check listed markets', async () => {
      const {
        controller,
        pools: { dai },
      } = await setupWithPools()
      const { value: val1 } = await controller.query.seizeAllowed(
        dai.pool.address,
        ZERO_ADDRESS,
        ZERO_ADDRESS,
        ZERO_ADDRESS,
        0,
      )
      expect(val1.ok.err).toBe('MarketNotListed')
      const { value: val2 } = await controller.query.seizeAllowed(
        ZERO_ADDRESS,
        dai.pool.address,
        ZERO_ADDRESS,
        ZERO_ADDRESS,
        0,
      )
      expect(val2.ok.err).toBe('MarketNotListed')
    })
  })

  it('.transfer_allowed', async () => {
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
      incentivesController,
      manager: deployer,
    })
    const [sender, receiver] = users

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
    for await (const { sym, value, user } of [
      {
        sym: usdc,
        value: toDec6(50_000),
        user: sender,
      },
    ]) {
      const { pool, token } = sym
      await token.withSigner(deployer).tx.mint(user.address, value)
      await token.withSigner(user).tx.approve(pool.address, value)
      await pool.withSigner(user).tx.mint(value, { gasLimit })
    }

    {
      const res = await controller.query.transferAllowed(
        usdc.pool.address,
        sender.address,
        ZERO_ADDRESS,
        toDec6(50_000),
        null,
      )
      expect(res.value.ok.ok).toBe(null)
    }
    {
      const res = await controller.query.transferAllowed(
        usdc.pool.address,
        sender.address,
        ZERO_ADDRESS,
        toDec6(50_000).add(new BN(1)),
        null,
      )
      expect(res.value.ok.err).toBe('InsufficientLiquidity')
    }
    {
      const res = await usdc.pool
        .withSigner(sender)
        .query.transfer(receiver.address, toDec6(50_000).add(new BN(1)), [])
      expect(res.value.ok.err.custom).toBe('InsufficientLiquidity')
    }
  })

  it('.set_close_factor_mantissa', async () => {
    const { controller } = await setup()
    const expScale = BN_TEN.pow(new BN(18))
    const bn = expScale.mul(new BN(5)).div(new BN(100)) // 5%
    await controller.tx.setCloseFactorMantissa([bn])
    const after = (await controller.query.closeFactorMantissa()).value.ok
    expect(bn.toString()).toEqual(BigInt(after.toString()).toString())
  })

  it('.liquidation_incentive_mantissa', async () => {
    const { controller } = await setup()
    const expScale = BN_TEN.pow(new BN(18))
    const bn = expScale.mul(new BN(5)).div(new BN(100)) // 5%
    await controller.tx.setLiquidationIncentiveMantissa([bn])
    const after = (await controller.query.liquidationIncentiveMantissa()).value
      .ok
    expect(bn.toString()).toEqual(BigInt(after.toString()).toString())
  })

  it('.support_market', async () => {
    const { api, deployer, rateModel, controller, incentivesController } =
      await setup()

    const pools = await preparePoolsWithPreparedTokens({
      api,
      controller,
      rateModel,
      manager: deployer,
      incentivesController,
    })

    await controller.tx.supportMarket(
      pools.dai.pool.address,
      pools.dai.token.address,
    )
    const markets = (await controller.query.markets()).value.ok
    expect(markets.length).toBe(1)
    expect(markets[0]).toBe(pools.dai.pool.address)
  })

  describe('.support_market_with_collateral_factor_mantissa', () => {
    it('success', async () => {
      const {
        api,
        deployer,
        controller,
        rateModel,
        priceOracle,
        incentivesController,
      } = await setup()
      const pools = await preparePoolsWithPreparedTokens({
        api,
        controller,
        rateModel,
        manager: deployer,
        incentivesController,
      })

      // prepares
      for (const sym of [pools.dai, pools.usdc, pools.usdt]) {
        await priceOracle.tx.setFixedPrice(sym.token.address, ONE_ETHER)
        await controller.tx.supportMarketWithCollateralFactorMantissa(
          sym.pool.address,
          sym.token.address,
          [ONE_ETHER.mul(new BN(90)).div(new BN(100))],
        )
      }

      const markets = (await controller.query.markets()).value.ok
      expect(markets.length).toBe(3)
      expect(markets).toEqual([
        pools.dai.pool.address,
        pools.usdc.pool.address,
        pools.usdt.pool.address,
      ])
    })
    describe('fail', () => {
      const setupWithOnePool = async () => {
        const { api, deployer, controller, rateModel, incentivesController } =
          await setup()
        const dai = await preparePoolWithMockToken({
          api,
          controller,
          rateModel,
          manager: deployer,
          metadata: TEST_METADATAS.dai,
          incentivesController,
        })
        return { controller, pool: dai.pool, token: dai.token }
      }
      it('if collateral_factor is over limit', async () => {
        const { controller, pool, token } = await setupWithOnePool()

        const res =
          await controller.query.supportMarketWithCollateralFactorMantissa(
            pool.address,
            token.address,
            [ONE_ETHER.mul(new BN(90)).div(new BN(100)).add(new BN(1))],
          )
        expect(res.value.ok.err).toStrictEqual('InvalidCollateralFactor')
      })
      it('if cannot get price', async () => {
        const { controller, pool, token } = await setupWithOnePool()

        const res =
          await controller.query.supportMarketWithCollateralFactorMantissa(
            pool.address,
            token.address,
            [ONE_ETHER.mul(new BN(90)).div(new BN(100))],
          )
        expect(res.value.ok.err).toStrictEqual('PriceError')
      })
    })
  })

  it('.account_assets', async () => {
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
    const user = users[0]
    const pools = await preparePoolsWithPreparedTokens({
      api,
      controller,
      rateModel,
      manager: deployer,
      incentivesController,
    })
    const { dai, usdc, usdt } = pools

    // prepares
    for (const sym of [dai, usdc, usdt]) {
      await priceOracle.tx.setFixedPrice(sym.token.address, ONE_ETHER)
      await controller.tx.supportMarketWithCollateralFactorMantissa(
        sym.pool.address,
        sym.token.address,
        [ONE_ETHER.mul(new BN(90)).div(new BN(100))],
      )
    }

    const getAccountAssets = async (address: string) =>
      (await controller.query.accountAssets(address)).value.ok
    expect(await getAccountAssets(user.address)).toEqual([])

    for (const sym of [dai, usdc, usdt]) {
      const { token, pool } = sym
      await shouldNotRevert(token.withSigner(user), 'mint', [
        user.address,
        10_000,
      ])
      await shouldNotRevert(token.withSigner(user), 'approve', [
        pool.address,
        10_000,
      ])
    }

    await shouldNotRevert(dai.pool.withSigner(user), 'mint', [
      1_000,
      { gasLimit },
    ])
    expect(await getAccountAssets(user.address)).toEqual([dai.pool.address])

    await shouldNotRevert(usdc.pool.withSigner(user), 'mint', [
      1_000,
      { gasLimit },
    ])
    await shouldNotRevert(usdc.pool.withSigner(user), 'mint', [
      1_000,
      { gasLimit },
    ])
    expect(await getAccountAssets(user.address)).toEqual([
      dai.pool.address,
      usdc.pool.address,
    ])

    await shouldNotRevert(usdt.pool.withSigner(user), 'mint', [
      1_000,
      { gasLimit },
    ])
    await shouldNotRevert(usdt.pool.withSigner(user), 'mint', [
      1_000,
      { gasLimit },
    ])
    await shouldNotRevert(usdt.pool.withSigner(user), 'mint', [
      1_000,
      { gasLimit },
    ])
    expect(await getAccountAssets(user.address)).toEqual([
      dai.pool.address,
      usdc.pool.address,
      usdt.pool.address,
    ])
  })

  describe('.get_account_liquidity / .get_hypothetical_account_liquidity', () => {
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

    describe('only mint', () => {
      it('single asset', async () => {
        const {
          api,
          deployer,
          controller,
          rateModel,
          priceOracle,
          users,
          incentivesController,
          gasLimit,
        } = await setup()
        const { dai, usdc } = await preparePoolsWithPreparedTokens({
          api,
          controller,
          rateModel,
          manager: deployer,
          incentivesController,
        })
        const [daiUser, usdcUser] = users

        // prerequisite
        //// initialize
        for (const sym of [dai, usdc]) {
          await priceOracle.tx.setFixedPrice(sym.token.address, ONE_ETHER)
          await controller.tx.supportMarketWithCollateralFactorMantissa(
            sym.pool.address,
            sym.token.address,
            [ONE_ETHER.mul(new BN(90)).div(new BN(100))],
          )
        }
        //// use protocol
        for await (const { sym, value, user } of [
          {
            sym: dai,
            value: toDec18(100),
            user: daiUser,
          },
          {
            sym: usdc,
            value: toDec6(500),
            user: usdcUser,
          },
        ]) {
          const { pool, token } = sym
          await token.withSigner(deployer).tx.mint(user.address, value)
          await token.withSigner(user).tx.approve(pool.address, value)
          await pool.withSigner(user).tx.mint(value, { gasLimit })
        }

        // execute
        //// .get_account_liquidity
        assertAccountLiquidity(
          (await controller.query.getAccountLiquidity(daiUser.address)).value.ok
            .ok,
          {
            collateral: 90,
            shortfall: 0,
          },
        )
        assertAccountLiquidity(
          (await controller.query.getAccountLiquidity(usdcUser.address)).value
            .ok.ok,
          {
            collateral: 450,
            shortfall: 0,
          },
        )
        //// .get_hypothetical_account_liquidity
        assertAccountLiquidity(
          (
            await controller.query.getHypotheticalAccountLiquidity(
              daiUser.address,
              usdc.pool.address,
              toDec6(50),
              new BN(0),
              null,
            )
          ).value.ok.ok,
          {
            collateral: 90 - (50 * 90) / 100,
            shortfall: 0,
          },
        )
        assertAccountLiquidity(
          (
            await controller.query.getHypotheticalAccountLiquidity(
              usdcUser.address,
              dai.pool.address,
              new BN(0),
              toDec18(500),
              null,
            )
          ).value.ok.ok,
          {
            collateral: 0,
            shortfall: 500 - 450,
          },
        )
        expect(1).toEqual(1)
      })
      it('multi asset', async () => {
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
        const { dai, usdc, usdt } = await preparePoolsWithPreparedTokens({
          api,
          controller,
          rateModel,
          manager: deployer,
          incentivesController,
        })
        const user = users[0]

        // prerequisite
        //// initialize
        for (const sym of [dai, usdc, usdt]) {
          await priceOracle.tx.setFixedPrice(sym.token.address, ONE_ETHER)
          await controller.tx.supportMarketWithCollateralFactorMantissa(
            sym.pool.address,
            sym.token.address,
            [ONE_ETHER.mul(new BN(90)).div(new BN(100))],
          )
        }

        //// use protocol
        for await (const { sym, value } of [
          {
            sym: dai,
            value: toDec18(1_000),
          },
          {
            sym: usdc,
            value: toDec6(2_000),
          },
          {
            sym: usdt,
            value: toDec6(3_000),
          },
        ]) {
          const { pool, token } = sym
          await token.withSigner(deployer).tx.mint(user.address, value)
          await token.withSigner(user).tx.approve(pool.address, value)
          await pool.withSigner(user).tx.mint(value, { gasLimit })
        }

        // execute
        //// .get_account_liquidity
        assertAccountLiquidity(
          (await controller.query.getAccountLiquidity(user.address)).value.ok
            .ok,
          {
            collateral: 5_400,
            shortfall: 0,
          },
        )
        //// .get_hypothetical_account_liquidity
        assertAccountLiquidity(
          (
            await controller.query.getHypotheticalAccountLiquidity(
              user.address,
              ZERO_ADDRESS,
              new BN(0),
              new BN(0),
              null,
            )
          ).value.ok.ok,
          {
            collateral: 5_400,
            shortfall: 0,
          },
        )
        assertAccountLiquidity(
          (
            await controller.query.getHypotheticalAccountLiquidity(
              user.address,
              dai.pool.address,
              toDec18(10_000), // some redeem
              new BN(0),
              null,
            )
          ).value.ok.ok,
          {
            collateral: 0,
            shortfall: (10_000 * 90) / 100 - 5_400,
          },
        )
        assertAccountLiquidity(
          (
            await controller.query.getHypotheticalAccountLiquidity(
              user.address,
              usdc.pool.address,
              new BN(0),
              toDec6(5_399), // some borrow
              null,
            )
          ).value.ok.ok,
          {
            collateral: 1,
            shortfall: 0,
          },
        )
        expect(1).toEqual(1)
      })
    })
    describe('with borrows', () => {
      it('multi asset', async () => {
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
        const { dai, usdc, usdt } = await preparePoolsWithPreparedTokens({
          api,
          controller,
          rateModel,
          manager: deployer,
          incentivesController,
        })
        const user = users[0]

        // prerequisite
        //// initialize
        for (const sym of [dai, usdc, usdt]) {
          await priceOracle.tx.setFixedPrice(sym.token.address, ONE_ETHER)
          await controller.tx.supportMarketWithCollateralFactorMantissa(
            sym.pool.address,
            sym.token.address,
            [ONE_ETHER.mul(new BN(90)).div(new BN(100))],
          )
        }

        //// use protocol
        ////// add liquidity
        for await (const { sym, liquidity } of [
          {
            sym: dai,
            liquidity: toDec18(500_000),
          },
          {
            sym: usdc,
            liquidity: toDec6(500_000),
          },
          {
            sym: usdt,
            liquidity: toDec6(500_000),
          },
        ]) {
          const { pool, token } = sym
          await token.withSigner(deployer).tx.mint(deployer.address, liquidity)
          await token.withSigner(deployer).tx.approve(pool.address, liquidity)
          await pool.withSigner(deployer).tx.mint(liquidity, { gasLimit })
        }
        ////// mint, borrow from user
        for await (const { sym, mintValue, borrowValue } of [
          {
            sym: dai,
            mintValue: toDec18(250_000),
            borrowValue: toDec18(50_000),
          },
          {
            sym: usdc,
            borrowValue: toDec6(150_000),
          },
          {
            sym: usdt,
            mintValue: toDec6(300_000),
          },
        ]) {
          const { pool, token } = sym
          if (mintValue) {
            await token.withSigner(deployer).tx.mint(user.address, mintValue)
            await token.withSigner(user).tx.approve(pool.address, mintValue)
            await pool.withSigner(user).tx.mint(mintValue, { gasLimit })
          }
          if (borrowValue) {
            await pool.withSigner(user).tx.borrow(borrowValue, { gasLimit })
          }
        }
        const expectedCollateral = ((250_000 + 300_000) * 90) / 100
        const expectedShortfall = 50_000 + 150_000

        // execute
        //// .get_account_liquidity
        assertAccountLiquidity(
          (await controller.query.getAccountLiquidity(user.address)).value.ok
            .ok,
          {
            collateral: expectedCollateral - expectedShortfall,
            shortfall: 0,
          },
        )
        //// .get_hypothetical_account_liquidity
        assertAccountLiquidity(
          (
            await controller.query.getHypotheticalAccountLiquidity(
              user.address,
              ZERO_ADDRESS,
              new BN(0),
              new BN(0),
              null,
            )
          ).value.ok.ok,
          {
            collateral: expectedCollateral - expectedShortfall,
            shortfall: 0,
          },
        )
        assertAccountLiquidity(
          (
            await controller.query.getHypotheticalAccountLiquidity(
              user.address,
              dai.pool.address,
              toDec18(10_000), // some redeem
              new BN(0),
              null,
            )
          ).value.ok.ok,
          {
            collateral:
              expectedCollateral - expectedShortfall - (10_000 * 90) / 100,
            shortfall: 0,
          },
        )
        assertAccountLiquidity(
          (
            await controller.query.getHypotheticalAccountLiquidity(
              user.address,
              dai.pool.address,
              new BN(0),
              toDec18(10_000), // some borrow
              null,
            )
          ).value.ok.ok,
          {
            collateral: expectedCollateral - expectedShortfall - 10_000,
            shortfall: 0,
          },
        )
        expect(1).toEqual(1)
      })
    })
  })

  describe('.calculate_user_account_data', () => {
    let controller: Controller
    let pools: Pools
    let deployer: KeyringPair
    let users: KeyringPair[]
    let priceOracle: PriceOracle
    let gasLimit: WeightV2

    it('instantiate', async () => {
      ;({ controller, deployer, pools, users, priceOracle, gasLimit } =
        await setupWithPools())
      const { dai, usdt, usdc } = pools

      const markets = (await controller.query.markets()).value.ok
      expect(markets.length).toBe(3)
      expect((await controller.query.oracle()).value.ok).toEqual(
        priceOracle.address,
      )
      const closeFactorMantissa = (await controller.query.closeFactorMantissa())
        .value.ok
      expect(closeFactorMantissa.toNumber()).toEqual(0)
      const liquidationIncentiveMantissa = (
        await controller.query.liquidationIncentiveMantissa()
      ).value.ok
      expect(liquidationIncentiveMantissa.toNumber()).toEqual(0)

      await shouldNotRevert(dai.pool, 'setLiquidationThreshold', [8500])
      await shouldNotRevert(usdt.pool, 'setLiquidationThreshold', [9000])
      await shouldNotRevert(usdc.pool, 'setLiquidationThreshold', [8000])

      expect(
        (await dai.pool.query.liquidationThreshold()).value.ok.toNumber(),
      ).toEqual(8500)
      expect(
        (await usdt.pool.query.liquidationThreshold()).value.ok.toNumber(),
      ).toEqual(9000)
      expect(
        (await usdc.pool.query.liquidationThreshold()).value.ok.toNumber(),
      ).toEqual(8000)
    })

    const daiDeposited = 50_000
    const usdcDeposited = 30_000
    const daiBorrowed = 20_000
    it('preparation', async () => {
      const { dai, usdc } = pools

      await shouldNotRevert(dai.token, 'mint', [users[0].address, daiDeposited])
      await shouldNotRevert(dai.token.withSigner(users[0]), 'approve', [
        dai.pool.address,
        daiDeposited,
      ])
      await shouldNotRevert(dai.pool.withSigner(users[0]), 'mint', [
        daiDeposited,
        { gasLimit },
      ])

      expect(
        (await dai.pool.query.balanceOf(users[0].address)).value.ok.toNumber(),
      ).toEqual(daiDeposited)

      await shouldNotRevert(usdc.token, 'mint', [
        deployer.address,
        usdcDeposited,
      ])
      await shouldNotRevert(usdc.token, 'approve', [
        usdc.pool.address,
        usdcDeposited,
      ])
      await shouldNotRevert(usdc.pool, 'mint', [usdcDeposited, { gasLimit }])

      expect(
        (await usdc.pool.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(usdcDeposited)

      await shouldNotRevert(dai.pool, 'borrow', [daiBorrowed, { gasLimit }])

      expect(
        (
          await dai.pool.query.borrowBalanceStored(deployer.address)
        ).value.ok.toNumber(),
      ).toEqual(daiBorrowed)
    })

    it('check account data', async () => {
      const deployerAccountData = (
        await controller.query.calculateUserAccountData(deployer.address, null)
      ).value.ok.ok

      // Total Collateral In Eth
      expect(
        new BN(
          BigInt(
            deployerAccountData.totalCollateralInBaseCurrency.toString(),
          ).toString(),
        ).toString(),
      ).toEqual(new BN(usdcDeposited).toString())

      // Total Debt In Eth
      expect(
        new BN(
          BigInt(
            deployerAccountData.totalDebtInBaseCurrency.toString(),
          ).toString(),
        ).toString(),
      ).toEqual(new BN(daiBorrowed).toString())

      expect(
        (await controller.query.accountAssets(users[0].address)).value.ok
          .length,
      ).toEqual(1)
    })
  })
})
