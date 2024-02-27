import type { ApiPromise } from '@polkadot/api'
import { encodeAddress } from '@polkadot/keyring'
import type { KeyringPair } from '@polkadot/keyring/types'
import { BN } from '@polkadot/util'
import { ONE_ETHER, ROLE, ZERO_ADDRESS } from '../scripts/helper/constants'
import {
  deployController,
  deployDefaultInterestRateModel,
  deployIncentivesController,
  deployManager,
  deployPriceOracle,
} from '../scripts/helper/deploy_helper'
import Controller from '../types/contracts/controller'
import Manager from '../types/contracts/manager'
import PriceOracle from '../types/contracts/price_oracle'
import {
  PoolContracts,
  Pools,
  preparePoolsWithPreparedTokens,
} from './testContractHelper'
import { shouldNotRevert } from './testHelpers'

describe('Manager spec', () => {
  const setup = async () => {
    const { api, alice: deployer } = globalThis.setup

    const manager = await deployManager({
      api,
      signer: deployer,
      args: [ZERO_ADDRESS],
    })

    const controller = await deployController({
      api,
      signer: deployer,
      args: [manager.address],
    })

    const incentivesController = await deployIncentivesController({
      api,
      signer: deployer,
      args: [],
    })

    // initialize
    await shouldNotRevert(manager, 'setController', [controller.address])

    const rateModelArg = new BN(100).mul(ONE_ETHER)
    const rateModel = await deployDefaultInterestRateModel({
      api,
      signer: deployer,
      args: [[rateModelArg], [rateModelArg], [rateModelArg], [rateModelArg]],
    })

    const pools = await preparePoolsWithPreparedTokens({
      api,
      controller,
      rateModel,
      signer: deployer,
      manager: manager.address,
      incentivesController,
    })

    const priceOracle = await deployPriceOracle({
      api,
      signer: deployer,
      args: [],
    })

    return { deployer, manager, controller, pools, priceOracle, api }
  }

  it('instantiate', async () => {
    const { deployer, manager, controller } = await setup()
    expect(
      (await manager.query.hasRole(ROLE.DEFAULT_ADMIN_ROLE, deployer.address))
        .value.ok,
    ).toBeTruthy()

    // connections
    expect((await controller.query.manager()).value.ok).toBe(manager.address)
    expect((await manager.query.controller()).value.ok).toBe(controller.address)
  })

  describe('call Controller', () => {
    it('.set_price_oracle', async () => {
      const { deployer, manager, controller } = await setup()
      const oracleAddr = encodeAddress(
        '0x0000000000000000000000000000000000000000000000000000000000000000',
      )
      const { value: value1 } = await manager.query.setPriceOracle(oracleAddr)
      expect(value1.ok.err).toStrictEqual({ accessControl: 'MissingRole' })

      await manager.tx.grantRole(ROLE.CONTROLLER_ADMIN, deployer.address)
      await manager.tx.setPriceOracle(oracleAddr)

      const { value: value2 } = await controller.query.oracle()
      expect(value2.ok).toEqual(oracleAddr)
    })
    it('.support_market_with_collateral_factor_mantissa', async () => {
      const {
        deployer,
        manager,
        controller,
        pools: { usdc },
        priceOracle,
      } = await setup()
      const collateralFactor = ONE_ETHER.mul(new BN(90)).div(new BN(100))

      await shouldNotRevert(manager, 'grantRole', [
        ROLE.CONTROLLER_ADMIN,
        deployer.address,
      ])

      await shouldNotRevert(manager, 'setPriceOracle', [priceOracle.address])

      await shouldNotRevert(priceOracle, 'setFixedPrice', [
        usdc.token.address,
        ONE_ETHER,
      ])

      await shouldNotRevert(
        manager,
        'supportMarketWithCollateralFactorMantissa',
        [usdc.pool.address, usdc.token.address, [collateralFactor]],
      )

      const {
        value: { ok: actual },
      } = await controller.query.collateralFactorMantissa(usdc.pool.address)
      expect(new BN(BigInt(actual.toString()).toString())).toEqual(
        collateralFactor,
      )
    })
    it('.set_borrow_cap', async () => {
      const { deployer, manager, controller, priceOracle } = await setup()
      const poolAddr = encodeAddress(
        '0x0000000000000000000000000000000000000000000000000000000000000000',
      )

      await shouldNotRevert(manager, 'grantRole', [
        ROLE.CONTROLLER_ADMIN,
        deployer.address,
      ])
      await shouldNotRevert(manager, 'supportMarket', [poolAddr, poolAddr])
      await shouldNotRevert(priceOracle, 'setFixedPrice', [poolAddr, ONE_ETHER])

      const { value: value1 } = await manager.query.setBorrowCap(poolAddr, 10)
      expect(value1.ok.err).toStrictEqual({ accessControl: 'MissingRole' })

      await shouldNotRevert(manager, 'grantRole', [
        ROLE.BORROW_CAP_GUARDIAN,
        deployer.address,
      ])
      await shouldNotRevert(manager, 'setBorrowCap', [poolAddr, 10])

      const { value: value2 } = await controller.query.borrowCap(poolAddr)
      expect(value2.ok).toEqual(10)
    })
    it('.set_mint_guardian_paused', async () => {
      const { deployer, manager, controller, priceOracle } = await setup()
      const poolAddr = encodeAddress(
        '0x0000000000000000000000000000000000000000000000000000000000000000',
      )

      await shouldNotRevert(manager, 'grantRole', [
        ROLE.CONTROLLER_ADMIN,
        deployer.address,
      ])
      await manager.tx.supportMarket(poolAddr, poolAddr)
      await shouldNotRevert(priceOracle, 'setFixedPrice', [poolAddr, ONE_ETHER])

      const { value: value1 } = await manager.query.setMintGuardianPaused(
        poolAddr,
        true,
      )
      expect(value1.ok.err).toStrictEqual({ accessControl: 'MissingRole' })

      await manager.tx.grantRole(ROLE.PAUSE_GUARDIAN, deployer.address)
      await manager.tx.setMintGuardianPaused(poolAddr, true)

      const { value: value2 } = await controller.query.mintGuardianPaused(
        poolAddr,
      )
      expect(value2.ok).toEqual(true)
    })
  })

  describe('Transfer Controller Manager', () => {
    let manager: Manager
    let newManager: Manager
    let controller: Controller
    let deployer: KeyringPair
    let api: ApiPromise

    beforeAll(async () => {
      ;({ manager, controller, api, deployer } = await setup())

      await shouldNotRevert(manager, 'grantRole', [
        ROLE.CONTROLLER_ADMIN,
        deployer.address,
      ])

      newManager = await deployManager({
        api,
        signer: deployer,
        args: [controller.address],
      })

      await shouldNotRevert(newManager, 'grantRole', [
        ROLE.CONTROLLER_ADMIN,
        deployer.address,
      ])
    })

    it('Accept Controller Manager Failed. (Pending manager is not set)', async () => {
      const result = (await newManager.query.acceptControllerManager()).value.ok
      expect(result.err).toStrictEqual({ controller: 'PendingManagerIsNotSet' })
    })

    it('Set Controller Manager.', async () => {
      await shouldNotRevert(manager, 'setControllerManager', [
        newManager.address,
      ])

      const pendingManager = (await controller.query.pendingManager()).value.ok
      expect(pendingManager).toBe(newManager.address)
    })

    it('Accept Controller Manager Failed. (Caller is not pending manager)', async () => {
      const result = (await manager.query.acceptControllerManager()).value.ok
      expect(result.err).toStrictEqual({
        controller: 'CallerIsNotPendingManager',
      })
    })

    it('Accept Controller Manager Works.', async () => {
      await shouldNotRevert(newManager, 'acceptControllerManager', [])

      const pendingManager = (await controller.query.pendingManager()).value.ok
      expect(pendingManager).toBe(null)

      const manager = (await controller.query.manager()).value.ok
      expect(manager).toBe(newManager.address)
    })
  })

  describe('Transfer Pool Manager', () => {
    let manager: Manager
    let newManager: Manager
    let controller: Controller
    let deployer: KeyringPair
    let api: ApiPromise
    let pools: Pools
    let dai: PoolContracts
    let priceOracle: PriceOracle

    beforeAll(async () => {
      ;({ manager, controller, api, deployer, pools, priceOracle } =
        await setup())
      ;({ dai } = pools)

      await shouldNotRevert(manager, 'grantRole', [
        ROLE.TOKEN_ADMIN,
        deployer.address,
      ])

      await shouldNotRevert(manager, 'grantRole', [
        ROLE.CONTROLLER_ADMIN,
        deployer.address,
      ])

      await shouldNotRevert(manager, 'setPriceOracle', [priceOracle.address])

      await shouldNotRevert(priceOracle, 'setFixedPrice', [
        dai.token.address,
        ONE_ETHER,
      ])

      const collateralFactor = ONE_ETHER.mul(new BN(90)).div(new BN(100))
      await shouldNotRevert(
        manager,
        'supportMarketWithCollateralFactorMantissa',
        [dai.pool.address, dai.token.address, [collateralFactor]],
      )

      newManager = await deployManager({
        api,
        signer: deployer,
        args: [controller.address],
      })

      await shouldNotRevert(newManager, 'grantRole', [
        ROLE.TOKEN_ADMIN,
        deployer.address,
      ])
    })

    it('Accept Pool Manager Failed. (Pending manager is not set)', async () => {
      const result = (
        await newManager.query.acceptPoolManager(dai.pool.address)
      ).value.ok
      expect(result.err).toStrictEqual({
        pool: {
          pendingManagerIsNotSet: null,
        },
      })
    })

    it('Set Pool Manager.', async () => {
      await shouldNotRevert(manager, 'setPoolManager', [
        dai.pool.address,
        newManager.address,
      ])

      const pendingManager = (await dai.pool.query.pendingManager()).value.ok
      expect(pendingManager).toBe(newManager.address)
    })

    it('Accept Pool Manager Failed. (Caller is not pending manager)', async () => {
      const result = (await manager.query.acceptPoolManager(dai.pool.address))
        .value.ok
      expect(result.err).toStrictEqual({
        pool: { callerIsNotPendingManager: null },
      })
    })

    it('Accept Pool Manager Works.', async () => {
      await shouldNotRevert(newManager, 'acceptPoolManager', [dai.pool.address])

      const pendingManager = (await dai.pool.query.pendingManager()).value.ok
      expect(pendingManager).toBe(null)

      const manager = (await dai.pool.query.manager()).value.ok
      expect(manager).toBe(newManager.address)
    })
  })
})
