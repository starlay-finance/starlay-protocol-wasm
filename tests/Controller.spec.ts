import type { ApiPromise } from '@polkadot/api'
import { encodeAddress } from '@polkadot/keyring'
import type { KeyringPair } from '@polkadot/keyring/types'
import BN from 'bn.js'
import {
  deployController,
  deployDefaultInterestRateModel,
  deployPoolFromAsset,
  deployPriceOracle,
  deployPSP22Token,
} from '../scripts/helper/deploy_helper'
import { ONE_ETHER } from '../scripts/tokens'
import Controller from '../types/contracts/controller'
import DefaultInterestRateModel from '../types/contracts/default_interest_rate_model'
import Pool from '../types/contracts/pool'
import PSP22Token from '../types/contracts/psp22_token'
import { shouldNotRevert } from './testHelpers'

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
    decimals: 8,
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
    args: [token.address, controller.address, rateModel.address],
    token,
  })

  return { token, pool }
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
}): Promise<{
  [key in (typeof TOKENS)[number]]: {
    token: PSP22Token
    pool: Pool
  }
}> => {
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

describe('Controller spec', () => {
  const setup = async () => {
    const { api, alice: deployer, bob, charie } = globalThis.setup

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
      users: [bob, charie],
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
          metadata: METADATAS.dai,
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
})
