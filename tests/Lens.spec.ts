import type { KeyringPair } from '@polkadot/keyring/types'
import { WeightV2 } from '@polkadot/types/interfaces'
import { BN, BN_ONE, BN_TEN } from '@polkadot/util'
import { ReturnNumber } from '@starlay-finance/typechain-types'
import { ONE_ETHER } from '../scripts/helper/constants'
import {
  deployController,
  deployDefaultInterestRateModel,
  deployFaucet,
  deployLens,
  deployPSP22Token,
  deployPoolFromAsset,
  deployPriceOracle,
} from '../scripts/helper/deploy_helper'
import { getGasLimit } from '../scripts/helper/utils'
import Controller from '../types/contracts/controller'
import Faucet from '../types/contracts/faucet'
import Lens from '../types/contracts/lens'
import Pool from '../types/contracts/pool'
import PriceOracle from '../types/contracts/price_oracle'
import PSP22Token from '../types/contracts/psp22_token'
import { shouldNotRevert, shouldNotRevertWithNetworkGas } from './testHelpers'

const MAX_CALL_WEIGHT = new BN(100_000_000_000).isub(BN_ONE).mul(BN_TEN)
const PROOFSIZE = new BN(2_000_000)
const setup = async (
  args: Partial<{
    price: string | number | BN
    collateralFactor: string | number | BN
    reserveFactor: string | number | BN
    borrowCap: string | number | BN
    liquidationIncentive: string | number | BN
    closeFactor: string | number | BN
  }> = {},
) => {
  const {
    price = 1,
    collateralFactor = ONE_ETHER.mul(new BN(90)).div(new BN(100)),
    reserveFactor = ONE_ETHER.mul(BN_TEN).div(new BN(100)),
    borrowCap = ONE_ETHER,
    liquidationIncentive = ONE_ETHER.mul(BN_TEN).div(new BN(100)),
    closeFactor = ONE_ETHER.mul(new BN(90)).div(new BN(100)),
  } = args
  const { api, alice: deployer, bob, charlie } = globalThis.setup

  const token1 = await deployPSP22Token({
    api,
    signer: deployer,
    args: [0, 'Dai Stablecoin', 'DAI', 8],
  })
  const token2 = await deployPSP22Token({
    api,
    signer: deployer,
    args: [0, 'USDCoin', 'USDC', 18],
  })

  const controller = await deployController({
    api,
    signer: deployer,
    args: [deployer.address],
  })

  const interestRateModel = await deployDefaultInterestRateModel({
    api,
    signer: deployer,
    args: [[ONE_ETHER], [ONE_ETHER], [ONE_ETHER], [ONE_ETHER]],
  })

  const pool1 = await deployPoolFromAsset({
    api,
    signer: deployer,
    args: [
      null,
      token1.address,
      controller.address,
      interestRateModel.address,
      [ONE_ETHER.toString()],
      10000,
    ],
    token: token1,
  })

  const pool2 = await deployPoolFromAsset({
    api,
    signer: deployer,
    args: [
      null,
      token2.address,
      controller.address,
      interestRateModel.address,
      [ONE_ETHER.toString()],
      10000,
    ],
    token: token2,
  })

  const priceOracle = await deployPriceOracle({
    api,
    signer: deployer,
    args: [],
  })

  const users = [bob, charlie]

  // initialize
  await shouldNotRevertWithNetworkGas(
    api,
    controller,
    'setLiquidationIncentiveMantissa',
    [[liquidationIncentive]],
  )
  await shouldNotRevertWithNetworkGas(
    api,
    controller,
    'setCloseFactorMantissa',
    [[closeFactor]],
  )
  await shouldNotRevertWithNetworkGas(api, controller, 'setPriceOracle', [
    priceOracle.address,
  ])
  await shouldNotRevertWithNetworkGas(api, priceOracle, 'setFixedPrice', [
    token1.address,
    price,
  ])
  await shouldNotRevertWithNetworkGas(api, priceOracle, 'setFixedPrice', [
    token2.address,
    price,
  ])
  await shouldNotRevertWithNetworkGas(
    api,
    controller,
    'supportMarketWithCollateralFactorMantissa',
    [pool1.address, token1.address, [collateralFactor]],
  )
  await shouldNotRevertWithNetworkGas(
    api,
    controller,
    'supportMarketWithCollateralFactorMantissa',
    [pool2.address, token2.address, [collateralFactor]],
  )
  await shouldNotRevertWithNetworkGas(api, pool1, 'setReserveFactorMantissa', [
    [reserveFactor],
  ])
  await shouldNotRevertWithNetworkGas(api, controller, 'setBorrowCap', [
    pool1.address,
    borrowCap,
  ])

  const lens = await deployLens({ api, signer: deployer, args: [] })
  const faucet = await deployFaucet({ api, signer: deployer, args: [] })

  const gasLimit = getGasLimit(api, MAX_CALL_WEIGHT, PROOFSIZE)

  return {
    api,
    deployer,
    tokens: [token1, token2],
    pools: [pool1, pool2],
    controller,
    priceOracle,
    lens,
    faucet,
    users,
    gasLimit,
  }
}

describe('Lens', () => {
  let lens: Lens
  let tokens: PSP22Token[]
  let pools: Pool[]
  let controller: Controller
  let priceOracle: PriceOracle
  let faucet: Faucet
  let signer: KeyringPair
  let deployer: KeyringPair
  let gasLimit: WeightV2

  describe('returns value', () => {
    const price = 1
    const collateralFactor = ONE_ETHER.mul(new BN(90)).div(new BN(100))
    const reserveFactor = ONE_ETHER.mul(BN_TEN).div(new BN(100))
    const borrowCap = ONE_ETHER.mul(new BN(100))
    const liquidationIncentive = ONE_ETHER.mul(BN_TEN).div(new BN(100))
    const closeFactor = ONE_ETHER.mul(new BN(90)).div(new BN(100))
    beforeAll(async () => {
      ;({
        lens,
        tokens,
        pools,
        controller,
        priceOracle,
        users: [signer],
        deployer,
        gasLimit,
      } = await setup({
        price,
        collateralFactor,
        reserveFactor,
        borrowCap,
        liquidationIncentive,
        closeFactor,
      }))
    })

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
        value: {
          ok: [res],
        },
      } = await lens.query.poolMetadataAll([pool.address])

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
      expect(res.exchangeRateCurrent.toHuman()).toEqual(ONE_ETHER.toString())
      expect(res.supplyRatePerMsec.toHuman()).toEqual('0')
      expect(res.borrowRatePerMsec.toHuman()).toEqual('31709791') // TODO
      expect(res.collateralFactorMantissa.toHuman()).toEqual(
        collateralFactor.toString(),
      )
      expect(res.reserveFactorMantissa.toHuman()).toEqual(
        reserveFactor.toString(),
      )
      // TODO fix typechain-polkadot to be able to handle optional result
      // @ts-ignore
      expect(new ReturnNumber(res.borrowCap).toHuman()).toBe(
        borrowCap.toString(),
      )
      expect(res.mintGuardianPaused).toBeFalsy()
      expect(res.borrowGuardianPaused).toBeFalsy()
    })

    it('Pool Balances', async () => {
      const pool = pools[0]
      const {
        value: {
          ok: [res],
        },
      } = await lens
        .withSigner(signer)
        .query.poolBalancesAll([pool.address], signer.address)

      expect(res.pool).toBe(pool.address)
      expect(res.balanceOf.toNumber()).toBe(0)
      expect(res.borrowBalanceCurrent.toNumber()).toBe(0)
      expect(res.tokenBalance.toNumber()).toBe(0)
      expect(res.tokenAllowance.toNumber()).toBe(0)
    })

    it('Pool UnderlyingPrice', async () => {
      const pool = pools[0]
      const {
        value: {
          ok: [res],
        },
      } = await lens.query.poolUnderlyingPriceAll([pool.address])

      expect(res.pool).toBe(pool.address)
      expect(res.underlyingPrice.toNumber()).toBe(price)
    })

    it('UnderlyingBalance', async () => {
      const pool = pools[0]
      const {
        value: {
          ok: [res],
        },
      } = await lens.query.underlyingBalanceAll([pool.address], signer.address)

      expect(res.toNumber()).toBe(0)
    })

    it('Configuration', async () => {
      const {
        value: { ok: res },
      } = await lens.query.configuration(controller.address)

      expect(res.manager).toBe(deployer.address)
      expect(res.oracle).toBe(priceOracle.address)
      expect(res.seizeGuardianPaused).toBeFalsy()
      expect(res.transferGuardianPaused).toBeFalsy()
      expect(res.liquidationIncentiveMantissa.toHuman()).toBe(
        liquidationIncentive.toString(),
      )
      expect(res.closeFactorMantissa.toHuman()).toBe(closeFactor.toString())
    })
  })

  describe('underlying Balance', () => {
    const amount = 100
    beforeAll(async () => {
      ;({
        lens,
        tokens,
        pools,
        controller,
        faucet,
        users: [signer],
      } = await setup())
      await faucet.tx.mintUnderlyingAll(
        controller.address,
        amount,
        signer.address,
      )
    })

    it('underlying_balance_all', async () => {
      const {
        value: { ok: res },
      } = await lens.query.underlyingBalanceAll(
        pools.map(({ address }) => address),
        signer.address,
      )
      expect(res).toHaveLength(pools.length)
      res.forEach((balance) => {
        expect(balance.toNumber()).toBe(amount)
      })
    })
  })
  describe('reflect pool values', () => {
    const balance = 1000
    beforeEach(async () => {
      ;({
        lens,
        tokens,
        pools,
        controller,
        faucet,
        users: [signer],
      } = await setup())
      await shouldNotRevert(faucet, 'mintUnderlyingAll', [
        controller.address,
        balance,
        signer.address,
      ])
    })
    it('faucet', async () => {
      const {
        value: { ok: pools },
      } = await lens.query.pools(controller.address)

      const {
        value: { ok: balances },
      } = await lens.query.poolBalancesAll(pools, signer.address)

      balances.forEach(({ tokenBalance }) => {
        expect(tokenBalance.toNumber()).toBe(balance)
      })
    })
    it('on minted', async () => {
      const depositAmount = 10
      const pool = pools[0].withSigner(signer)
      const token = tokens[0].withSigner(signer)

      await shouldNotRevert(token, 'approve', [pool.address, depositAmount])
      await shouldNotRevert(pool, 'mint', [depositAmount, { gasLimit }])

      const {
        value: { ok: metadata },
      } = await lens.query.poolMetadata(pool.address)

      const {
        value: {
          ok: [balances],
        },
      } = await lens.query.poolBalancesAll([pool.address], signer.address)

      expect(metadata.totalSupply.toNumber()).toBe(depositAmount)
      expect(metadata.totalCash.toNumber()).toBe(depositAmount)

      expect(balances.balanceOf.toNumber()).toBe(depositAmount)
      expect(balances.tokenBalance.toNumber()).toBe(balance - depositAmount)
    })
    it('on redeemed', async () => {
      const depositAmount = 100
      const redeemAmount = 50
      const pool = pools[0].withSigner(signer)
      const token = tokens[0].withSigner(signer)

      await shouldNotRevert(token, 'approve', [pool.address, depositAmount])
      await shouldNotRevert(pool, 'mint', [depositAmount, { gasLimit }])
      await shouldNotRevert(pool, 'redeemUnderlying', [
        redeemAmount,
        { gasLimit },
      ])

      const {
        value: { ok: metadata },
      } = await lens.query.poolMetadata(pool.address)

      const {
        value: {
          ok: [balances],
        },
      } = await lens.query.poolBalancesAll([pool.address], signer.address)

      expect(metadata.totalSupply.toNumber()).toBe(depositAmount - redeemAmount)
      expect(metadata.totalCash.toNumber()).toBe(depositAmount - redeemAmount)

      expect(balances.balanceOf.toNumber()).toBe(depositAmount - redeemAmount)
      expect(balances.tokenBalance.toNumber()).toBe(
        balance - depositAmount + redeemAmount,
      )
    })
    it('on borrowed', async () => {
      const depositAmount = 100
      const borrowAmount = 50
      const pool = pools[0].withSigner(signer)
      const token = tokens[0].withSigner(signer)

      await shouldNotRevert(token, 'approve', [pool.address, depositAmount])
      await shouldNotRevert(pool, 'mint', [depositAmount, { gasLimit }])
      await shouldNotRevert(pool, 'borrow', [borrowAmount, { gasLimit }])

      const {
        value: { ok: metadata },
      } = await lens.query.poolMetadata(pool.address)

      const {
        value: {
          ok: [balances],
        },
      } = await lens.query.poolBalancesAll([pool.address], signer.address)

      expect(metadata.totalSupply.toNumber()).toBe(depositAmount)
      expect(metadata.totalCash.toNumber()).toBe(depositAmount - borrowAmount)
      expect(metadata.totalBorrows.toNumber()).toBe(borrowAmount)

      expect(balances.balanceOf.toNumber()).toBe(depositAmount)
      expect(balances.borrowBalanceCurrent.toNumber()).toBe(borrowAmount)
      expect(balances.tokenBalance.toNumber()).toBe(
        balance - depositAmount + borrowAmount,
      )
    })
    it('on repaid', async () => {
      const depositAmount = 100
      const borrowAmount = 50
      const repayAmount = 20
      const pool = pools[0].withSigner(signer)
      const token = tokens[0].withSigner(signer)

      await shouldNotRevert(token, 'approve', [
        pool.address,
        depositAmount + repayAmount,
      ])
      await shouldNotRevert(pool, 'mint', [depositAmount, { gasLimit }])
      await shouldNotRevert(pool, 'borrow', [borrowAmount, { gasLimit }])
      await shouldNotRevert(pool, 'repayBorrow', [repayAmount, { gasLimit }])

      const {
        value: { ok: metadata },
      } = await lens.query.poolMetadata(pool.address)

      const {
        value: {
          ok: [balances],
        },
      } = await lens.query.poolBalancesAll([pool.address], signer.address)

      expect(metadata.totalSupply.toNumber()).toBe(depositAmount)
      expect(metadata.totalCash.toNumber()).toBe(
        depositAmount - borrowAmount + repayAmount,
      )
      expect(metadata.totalBorrows.toNumber()).toBe(borrowAmount - repayAmount)

      expect(balances.balanceOf.toNumber()).toBe(depositAmount)
      expect(balances.borrowBalanceCurrent.toNumber()).toBe(
        borrowAmount - repayAmount,
      )
      expect(balances.tokenBalance.toNumber()).toBe(
        balance - depositAmount + borrowAmount - repayAmount,
      )
    })
    it('on paused', async () => {
      const pool = pools[0]
      await shouldNotRevert(controller, 'setMintGuardianPaused', [
        pool.address,
        true,
      ])
      await shouldNotRevert(controller, 'setBorrowGuardianPaused', [
        pool.address,
        true,
      ])
      const {
        value: { ok: metadata },
      } = await lens.query.poolMetadata(pool.address)
      expect(metadata.mintGuardianPaused).toBeTruthy()
      expect(metadata.borrowGuardianPaused).toBeTruthy()
    })
  })
})
