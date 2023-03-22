import type { KeyringPair } from '@polkadot/keyring/types'
import Controller from '../types/contracts/controller'
import Lens from '../types/contracts/lens'
import Pool from '../types/contracts/pool'
import PSP22Token from '../types/contracts/psp22_token'
import {
  deployController,
  deployDefaultInterestRateModel,
  deployLens,
  deployPoolFromAsset,
  deployPriceOracle,
  deployPSP22Token,
} from './testContractsHelper'

const setup = async () => {
  const { api, alice: deployer, bob, charlie } = globalThis.setup

  const token1 = await deployPSP22Token({
    api,
    signer: deployer,
    args: [
      0,
      'Dai Stablecoin' as unknown as string[],
      'DAI' as unknown as string[],
      8,
    ],
  })
  const token2 = await deployPSP22Token({
    api,
    signer: deployer,
    args: [
      0,
      'USDCoin' as unknown as string[],
      'USDC' as unknown as string[],
      18,
    ],
  })

  const controller = await deployController({
    api,
    signer: deployer,
    args: [deployer.address],
  })

  const interestRateModel = await deployDefaultInterestRateModel({
    api,
    signer: deployer,
  })

  const pool1 = await deployPoolFromAsset({
    api,
    signer: deployer,
    args: [token1.address, controller.address, interestRateModel.address],
  })

  const pool2 = await deployPoolFromAsset({
    api,
    signer: deployer,
    args: [token2.address, controller.address, interestRateModel.address],
  })

  const priceOracle = await deployPriceOracle({
    api,
    signer: deployer,
  })

  const users = [bob, charlie]

  // initialize
  await controller.tx.supportMarket(pool1.address)
  await controller.tx.supportMarket(pool2.address)
  await controller.tx.setPriceOracle(priceOracle.address)
  await priceOracle.tx.setFixedPrice(token1.address, 0)
  await priceOracle.tx.setFixedPrice(token2.address, 0)

  const lens = await deployLens({ api, signer: deployer })

  console.log({
    lens: lens.address,
    controller: controller.address,
    pools: [pool1.address, pool2.address],
    interestRateModel: interestRateModel.address,
    tokens: [token1.address, token2.address],
  })
  return {
    api,
    deployer,
    tokens: [token1, token2],
    pools: [pool1, pool2],
    controller,
    lens,
    users,
  }
}

describe('Lens', () => {
  let lens: Lens
  let tokens: PSP22Token[]
  let pools: Pool[]
  let controller: Controller
  let signer: KeyringPair

  beforeAll(async () => {
    ;({
      lens,
      tokens,
      pools,
      controller,
      users: [signer],
    } = await setup())
  })

  describe('returns value', () => {
    it('Pools', async () => {
      const {
        value: { ok: res },
      } = await lens.query.pools(controller.address)

      expect(res).toHaveLength(pools.length)
      pools.forEach((pool, idx) => {
        expect(res[idx]).toBe(pool.address)
      })
    })
    it('Pool Metadata', async () => {
      const token = tokens[0]
      const pool = pools[0]
      const tokenDecimals = (await token.query.tokenDecimals()).value.ok
      const tokenSymbol = (await token.query.tokenSymbol()).value.ok
      const {
        value: { ok: res },
      } = await lens.query.poolMetadata(pool.address)

      expect(res.pool).toBe(pool.address)
      expect(res.poolDecimals).toBe(tokenDecimals)
      expect(res.underlyingAssetAddress).toBe(token.address)
      expect(res.underlyingDecimals).toBe(tokenDecimals)
      expect(res.underlyingSymbol).toBe(tokenSymbol)
      expect(res.isListed).toBeTruthy()
      expect(res.totalCash.toNumber()).toBe(0)
      expect(res.totalBorrows.toNumber()).toBe(0)
      expect(res.totalSupply.toNumber()).toBe(0)
      expect(res.totalReserves.toNumber()).toBe(0)
      expect(res.exchangeRateCurrent.toHuman()).toEqual('0')
      expect(res.supplyRatePerMsec.toHuman()).toEqual('0')
      expect(res.borrowRatePerMsec.toHuman()).toEqual('0')
      expect(res.collateralFactorMantissa.toNumber()).toEqual(0)
      expect(res.reserveFactorMantissa.toHuman()).toEqual('0')
      // expect(res.borrowCap).toBeNull()
      expect(res.borrowCap).toBe(0)
    })

    it('Pool Balances', async () => {
      const pool = pools[0]
      const {
        value: { ok: res },
      } = await lens
        .withSigner(signer)
        .query.poolBalances(pool.address, signer.address)

      expect(res.pool).toBe(pool.address)
      expect(res.balanceOf.toNumber()).toBe(0)
      expect(res.borrowBalanceCurrent.toNumber()).toBe(0)
      expect(res.balanceOfUnderlying.toNumber()).toBe(0)
      expect(res.tokenBalance.toNumber()).toBe(0)
      expect(res.tokenAllowance.toNumber()).toBe(0)
    })

    it('Pool UnderlyingPrice', async () => {
      const pool = pools[0]
      const {
        value: { ok: res },
      } = await lens.query.poolUnderlyingPrice(pool.address)

      expect(res.pool).toBe(pool.address)
      expect(res.underlyingPrice.toNumber()).toBe(0)
    })
  })

  it('UnderlyingBalance', async () => {
    const pool = pools[0]
    const {
      value: { ok: res },
    } = await lens.query.underlyingBalance(pool.address, signer.address)

    expect(res.toNumber()).toBe(0)
  })
})
