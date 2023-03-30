import { ReturnNumber } from '@727-ventures/typechain-types'
import { encodeAddress } from '@polkadot/keyring'
import BN from 'bn.js'
import { ONE_ETHER, ZERO_ADDRESS } from '../scripts/helper/constants'
import {
  deployController,
  deployDefaultInterestRateModel,
  deployPriceOracle,
} from '../scripts/helper/deploy_helper'
import { hexToUtf8 } from '../scripts/helper/utils'
import {
  preparePoolsWithPreparedTokens,
  preparePoolWithMockToken,
  TEST_METADATAS,
} from './testContractHelper'
import { mantissa, shouldNotRevert, toDec18, toDec6 } from './testHelpers'

describe('Controller spec', () => {
  const setup = async () => {
    const { api, alice: deployer, bob, charlie } = globalThis.setup

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

    // initialize
    await controller.tx.setPriceOracle(priceOracle.address)

    return {
      api,
      deployer,
      controller,
      rateModel,
      priceOracle,
      users: [bob, charlie],
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

  it('.set_close_factor_mantissa', async () => {
    const { controller } = await setup()
    const expScale = new BN(10).pow(new BN(18))
    const bn = expScale.mul(new BN(5)).div(new BN(100)) // 5%
    await controller.tx.setCloseFactorMantissa([bn])
    const after = (await controller.query.closeFactorMantissa()).value.ok
    expect(bn.toString()).toEqual(BigInt(after.toString()).toString())
  })

  it('.liquidation_incentive_mantissa', async () => {
    const { controller } = await setup()
    const expScale = new BN(10).pow(new BN(18))
    const bn = expScale.mul(new BN(5)).div(new BN(100)) // 5%
    await controller.tx.setLiquidationIncentiveMantissa([bn])
    const after = (await controller.query.liquidationIncentiveMantissa()).value
      .ok
    expect(bn.toString()).toEqual(BigInt(after.toString()).toString())
  })

  it('.support_market', async () => {
    const { controller } = await setup()

    const tokenAddress = encodeAddress(
      '0x0000000000000000000000000000000000000000000000000000000000000001',
    )

    await controller.tx.supportMarket(tokenAddress)
    const markets = (await controller.query.markets()).value.ok
    expect(markets.length).toBe(1)
    expect(markets[0]).toBe(tokenAddress)
  })

  describe('.support_market_with_collateral_factor_mantissa', () => {
    it('success', async () => {
      const { api, deployer, controller, rateModel, priceOracle } =
        await setup()
      const pools = await preparePoolsWithPreparedTokens({
        api,
        controller,
        rateModel,
        manager: deployer,
      })

      // prepares
      for (const sym of [pools.dai, pools.usdc, pools.usdt]) {
        await priceOracle.tx.setFixedPrice(sym.token.address, ONE_ETHER)
        await controller.tx.supportMarketWithCollateralFactorMantissa(
          sym.pool.address,
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
        const { api, deployer, controller, rateModel } = await setup()
        const dai = await preparePoolWithMockToken({
          api,
          controller,
          rateModel,
          manager: deployer,
          metadata: TEST_METADATAS.dai,
        })
        return { controller, pool: dai.pool }
      }
      it('if collateral_factor is over limit', async () => {
        const { controller, pool } = await setupWithOnePool()

        const res =
          await controller.query.supportMarketWithCollateralFactorMantissa(
            pool.address,
            [ONE_ETHER.mul(new BN(90)).div(new BN(100)).add(new BN(1))],
          )
        expect(res.value.ok.err).toStrictEqual('InvalidCollateralFactor')
      })
      it('if cannot get price', async () => {
        const { controller, pool } = await setupWithOnePool()

        const res =
          await controller.query.supportMarketWithCollateralFactorMantissa(
            pool.address,
            [ONE_ETHER.mul(new BN(90)).div(new BN(100))],
          )
        expect(res.value.ok.err).toStrictEqual('PriceError')
      })
    })
  })

  it('.account_assets', async () => {
    const { api, deployer, controller, rateModel, priceOracle, users } =
      await setup()
    const user = users[0]
    const pools = await preparePoolsWithPreparedTokens({
      api,
      controller,
      rateModel,
      manager: deployer,
    })
    const { dai, usdc, usdt } = pools

    // prepares
    for (const sym of [dai, usdc, usdt]) {
      await priceOracle.tx.setFixedPrice(sym.token.address, ONE_ETHER)
      await controller.tx.supportMarketWithCollateralFactorMantissa(
        sym.pool.address,
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

    await shouldNotRevert(dai.pool.withSigner(user), 'mint', [1_000])
    expect(await getAccountAssets(user.address)).toEqual([dai.pool.address])

    await shouldNotRevert(usdc.pool.withSigner(user), 'mint', [1_000])
    await shouldNotRevert(usdc.pool.withSigner(user), 'mint', [1_000])
    expect(await getAccountAssets(user.address)).toEqual([
      dai.pool.address,
      usdc.pool.address,
    ])

    await shouldNotRevert(usdt.pool.withSigner(user), 'mint', [1_000])
    await shouldNotRevert(usdt.pool.withSigner(user), 'mint', [1_000])
    await shouldNotRevert(usdt.pool.withSigner(user), 'mint', [1_000])
    expect(await getAccountAssets(user.address)).toEqual([
      dai.pool.address,
      usdc.pool.address,
      usdt.pool.address,
    ])
  })

  describe('.get_account_liquidity / .get_hypothetical_account_liquidity', () => {
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

    describe('only mint', () => {
      it('single asset', async () => {
        const { api, deployer, controller, rateModel, priceOracle, users } =
          await setup()
        const { dai, usdc } = await preparePoolsWithPreparedTokens({
          api,
          controller,
          rateModel,
          manager: deployer,
        })
        const [daiUser, usdcUser] = users

        // prerequisite
        //// initialize
        for (const sym of [dai, usdc]) {
          await priceOracle.tx.setFixedPrice(sym.token.address, ONE_ETHER)
          await controller.tx.supportMarketWithCollateralFactorMantissa(
            sym.pool.address,
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
          await pool.withSigner(user).tx.mint(value)
        }

        // execute
        //// .get_account_liquidity
        assertAccountLiqudity(
          (await controller.query.getAccountLiquidity(daiUser.address)).value.ok
            .ok,
          {
            collateral: 90,
            shortfall: 0,
          },
        )
        assertAccountLiqudity(
          (await controller.query.getAccountLiquidity(usdcUser.address)).value
            .ok.ok,
          {
            collateral: 450,
            shortfall: 0,
          },
        )
        //// .get_hypothetical_account_liquidity
        assertAccountLiqudity(
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
        assertAccountLiqudity(
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
      })
      it('multi asset', async () => {
        const { api, deployer, controller, rateModel, priceOracle, users } =
          await setup()
        const { dai, usdc, usdt } = await preparePoolsWithPreparedTokens({
          api,
          controller,
          rateModel,
          manager: deployer,
        })
        const user = users[0]

        // prerequisite
        //// initialize
        for (const sym of [dai, usdc, usdt]) {
          await priceOracle.tx.setFixedPrice(sym.token.address, ONE_ETHER)
          await controller.tx.supportMarketWithCollateralFactorMantissa(
            sym.pool.address,
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
          await pool.withSigner(user).tx.mint(value)
        }

        // execute
        //// .get_account_liquidity
        assertAccountLiqudity(
          (await controller.query.getAccountLiquidity(user.address)).value.ok
            .ok,
          {
            collateral: 5_400,
            shortfall: 0,
          },
        )
        //// .get_hypothetical_account_liquidity
        assertAccountLiqudity(
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
        assertAccountLiqudity(
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
        assertAccountLiqudity(
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
      })
    })
    describe('with borrows', () => {
      it('multi asset', async () => {
        const { api, deployer, controller, rateModel, priceOracle, users } =
          await setup()
        const { dai, usdc, usdt } = await preparePoolsWithPreparedTokens({
          api,
          controller,
          rateModel,
          manager: deployer,
        })
        const user = users[0]

        // prerequisite
        //// initialize
        for (const sym of [dai, usdc, usdt]) {
          await priceOracle.tx.setFixedPrice(sym.token.address, ONE_ETHER)
          await controller.tx.supportMarketWithCollateralFactorMantissa(
            sym.pool.address,
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
          await pool.withSigner(deployer).tx.mint(liquidity)
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
            await pool.withSigner(user).tx.mint(mintValue)
          }
          if (borrowValue) {
            await pool.withSigner(user).tx.borrow(borrowValue)
          }
        }
        const expectedCollateral = ((250_000 + 300_000) * 90) / 100
        const expectedShortfall = 50_000 + 150_000

        // execute
        //// .get_account_liquidity
        assertAccountLiqudity(
          (await controller.query.getAccountLiquidity(user.address)).value.ok
            .ok,
          {
            collateral: expectedCollateral - expectedShortfall,
            shortfall: 0,
          },
        )
        //// .get_hypothetical_account_liquidity
        assertAccountLiqudity(
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
        assertAccountLiqudity(
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
        assertAccountLiqudity(
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
      })
    })
  })

  describe('.xxx_allowed', () => {
    it('.transfer_allowed', async () => {
      const { api, deployer, controller, rateModel, priceOracle, users } =
        await setup()
      const { dai, usdc } = await preparePoolsWithPreparedTokens({
        api,
        controller,
        rateModel,
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
        await pool.withSigner(user).tx.mint(value)
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
        expect(hexToUtf8(res.value.ok.err['custom'])).toBe(
          'InsufficientLiquidity',
        )
      }
    })
  })
})
