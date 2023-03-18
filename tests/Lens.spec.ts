import type { KeyringPair } from '@polkadot/keyring/types'
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
    args: [],
  })

  const interestRateModel = await deployDefaultInterestRateModel({
    api,
    signer: deployer,
  })

  const pool = await deployPoolFromAsset({
    api,
    signer: deployer,
    args: [token.address, controller.address, interestRateModel.address],
  })

  const priceOracle = await deployPriceOracle({
    api,
    signer: deployer,
  })

  const users = [bob, charlie]

  // initialize
  await controller.tx.supportMarket(pool.address)
  await controller.tx.setPriceOracle(priceOracle.address)

  const lens = await deployLens({ api, signer: deployer })

  return { api, deployer, token, pool, controller, lens, users }
}

describe('Lens', () => {
  let lens: Lens
  let token: PSP22Token
  let pool: Pool
  let signer: KeyringPair

  beforeAll(async () => {
    ;({
      lens,
      token,
      pool,
      users: [signer],
    } = await setup())
  })

  describe('returns value', () => {
    it('Pool Metadata', async () => {
      const tokenDecimals = (await token.query.tokenDecimals()).value.ok
      const {
        value: { ok: res },
      } = await lens.query.poolMetadata(pool.address)

      expect(res.pool).toBe(pool.address)
      expect(res.poolDecimals).toBe(tokenDecimals)
      expect(res.underlyingAssetAddress).toBe(token.address)
      expect(res.underlyingDecimals).toBe(tokenDecimals)
      expect(res.isListed).toBeTruthy()
      expect(res.totalCash.toNumber()).toBe(0)
      expect(res.totalBorrows.toNumber()).toBe(0)
      expect(res.totalSupply.toNumber()).toBe(0)
      expect(res.totalReserves.toNumber()).toBe(0)
      // TODO U256 types
      expect(res.exchangeRateCurrent).toEqual(0)
      expect(res.supplyRatePerSec).toEqual(0)
      expect(res.borrowRatePerSec).toEqual(0)
      expect(res.collateralFactorMantissa.toNumber()).toEqual(0)
      expect(res.reserveFactorMantissa).toEqual(0)
      // expect(res.borrowCap).toBeNull()
      expect(res.borrowCap).toBe(0)
    })

    it('Pool Balances', async () => {
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
      const {
        value: { ok: res },
      } = await lens.query.poolUnderlyingPrice(pool.address)

      expect(res.pool).toBe(pool.address)
      expect(res.underlyingPrice.toNumber()).toBe(0)
    })
  })
})
