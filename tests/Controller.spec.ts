import { encodeAddress } from '@polkadot/keyring'
import BN from 'bn.js'
import {
  deployController,
  deployDefaultInterestRateModel,
  deployPriceOracle,
} from '../scripts/helper/deploy_helper'
import { ONE_ETHER } from '../scripts/tokens'
import {
  preparePoolsWithPreparedTokens,
  preparePoolWithMockToken,
  TEST_METADATAS,
} from './testContractHelper'
import { shouldNotRevert } from './testHelpers'

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
    const toParam = (m: BN) => [m.toString()] // temp

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
      const toParam = (m: BN) => [m.toString()] // temp
      for (const sym of [pools.dai, pools.usdc, pools.usdt]) {
        await priceOracle.tx.setFixedPrice(sym.token.address, ONE_ETHER)
        await controller.tx.supportMarketWithCollateralFactorMantissa(
          sym.pool.address,
          toParam(ONE_ETHER.mul(new BN(90)).div(new BN(100))),
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
            toParam(ONE_ETHER.mul(new BN(90)).div(new BN(100)).add(new BN(1))),
          )
        expect(res.value.ok.err).toStrictEqual('InvalidCollateralFactor')
      })
      it('if cannot get price', async () => {
        const { controller, pool } = await setupWithOnePool()

        const res =
          await controller.query.supportMarketWithCollateralFactorMantissa(
            pool.address,
            toParam(ONE_ETHER.mul(new BN(90)).div(new BN(100))),
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
    const toParam = (m: BN) => [m.toString()] // temp
    for (const sym of [dai, usdc, usdt]) {
      await priceOracle.tx.setFixedPrice(sym.token.address, ONE_ETHER)
      await controller.tx.supportMarketWithCollateralFactorMantissa(
        sym.pool.address,
        toParam(ONE_ETHER.mul(new BN(90)).div(new BN(100))),
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

  describe('.get_account_liquidity', () => {
    const pow10 = (exponent: number) => new BN(10).pow(new BN(exponent))
    const mantissa = () => pow10(18)
    const to_dec6 = (val: number | string) => new BN(val).mul(pow10(6))
    const to_dec18 = (val: number | string) => new BN(val).mul(pow10(18))
    const trimPrefix = (hex: string) => hex.replace(/^0x/, '')

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
            sym: dai,
            value: to_dec18(100),
            user: daiUser,
          },
          {
            sym: usdc,
            value: to_dec6(500),
            user: usdcUser,
          },
        ]) {
          const { pool, token } = sym
          await token.withSigner(deployer).tx.mint(user.address, value)
          await token.withSigner(user).tx.approve(pool.address, value)
          await pool.withSigner(user).tx.mint(value)
        }

        // execute
        const resDaiUser = (
          await controller.query.getAccountLiquidity(daiUser.address)
        ).value.ok.ok
        const collateral1 = BigInt(resDaiUser[0].toString()).toString()
        const shortfall1 = BigInt(resDaiUser[1].toString()).toString()
        expect(collateral1.toString()).toEqual(
          new BN(90).mul(mantissa()).toString(),
        )
        expect(shortfall1.toString()).toEqual(
          new BN(0).mul(mantissa()).toString(),
        )

        const resUsdcUser = (
          await controller.query.getAccountLiquidity(usdcUser.address)
        ).value.ok.ok
        const collateral2 = BigInt(resUsdcUser[0].toString()).toString()
        const shortfall2 = BigInt(resUsdcUser[1].toString()).toString()
        expect(collateral2.toString()).toEqual(
          new BN(450).mul(mantissa()).toString(),
        )
        expect(shortfall2.toString()).toEqual(
          new BN(0).mul(mantissa()).toString(),
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
        const toParam = (m: BN) => [m.toString()]
        for (const sym of [dai, usdc, usdt]) {
          await priceOracle.tx.setFixedPrice(sym.token.address, ONE_ETHER)
          await controller.tx.supportMarketWithCollateralFactorMantissa(
            sym.pool.address,
            toParam(ONE_ETHER.mul(new BN(90)).div(new BN(100))),
          )
        }

        //// use protocol
        for await (const { sym, value } of [
          {
            sym: dai,
            value: to_dec18(1_000),
          },
          {
            sym: usdc,
            value: to_dec6(2_000),
          },
          {
            sym: usdt,
            value: to_dec6(3_000),
          },
        ]) {
          const { pool, token } = sym
          await token.withSigner(deployer).tx.mint(user.address, value)
          await token.withSigner(user).tx.approve(pool.address, value)
          await pool.withSigner(user).tx.mint(value)
        }

        // execute
        const res = (await controller.query.getAccountLiquidity(user.address))
          .value.ok.ok
        const collateral = new BN(trimPrefix(res[0].toString()), 16)
        const shortfall = new BN(trimPrefix(res[1].toString()), 16)
        expect(collateral.toString()).toEqual(
          new BN(5_400).mul(mantissa()).toString(),
        )
        expect(shortfall.toString()).toEqual(new BN(0).toString())
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
        const toParam = (m: BN) => [m.toString()]
        for (const sym of [dai, usdc, usdt]) {
          await priceOracle.tx.setFixedPrice(sym.token.address, ONE_ETHER)
          await controller.tx.supportMarketWithCollateralFactorMantissa(
            sym.pool.address,
            toParam(ONE_ETHER.mul(new BN(90)).div(new BN(100))),
          )
        }

        //// use protocol
        ////// add liquidity
        for await (const { sym, liquidity } of [
          {
            sym: dai,
            liquidity: to_dec18(500_000),
          },
          {
            sym: usdc,
            liquidity: to_dec6(500_000),
          },
          {
            sym: usdt,
            liquidity: to_dec6(500_000),
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
            mintValue: to_dec18(250_000),
            borrowValue: to_dec18(50_000),
          },
          {
            sym: usdc,
            borrowValue: to_dec6(150_000),
          },
          {
            sym: usdt,
            mintValue: to_dec6(300_000),
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
        const res = (await controller.query.getAccountLiquidity(user.address))
          .value.ok.ok
        const collateral = new BN(trimPrefix(res[0].toString()), 16)
        const shortfall = new BN(trimPrefix(res[1].toString()), 16)

        expect(collateral.toString()).toBe(
          new BN(expectedCollateral - expectedShortfall)
            .mul(mantissa())
            .toString(),
        )
        expect(shortfall.toString()).toBe('0')
      })
    })
  })
})
