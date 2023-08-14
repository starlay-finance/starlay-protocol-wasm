import { encodeAddress } from '@polkadot/keyring'
import { BN } from '@polkadot/util'
import { ONE_ETHER, ROLE, ZERO_ADDRESS } from '../scripts/helper/constants'
import {
  deployController,
  deployDefaultInterestRateModel,
  deployIncentivesController,
  deployManager,
  deployPriceOracle,
} from '../scripts/helper/deploy_helper'
import { preparePoolsWithPreparedTokens } from './testContractHelper'
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
      manager: deployer,
      incentivesController,
    })

    const priceOracle = await deployPriceOracle({
      api,
      signer: deployer,
      args: [],
    })

    return { deployer, manager, controller, pools, priceOracle }
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
      const { deployer, manager, controller } = await setup()
      const poolAddr = encodeAddress(
        '0x0000000000000000000000000000000000000000000000000000000000000000',
      )
      const { value: value1 } = await manager.query.setBorrowCap(poolAddr, 10)
      expect(value1.ok.err).toStrictEqual({ accessControl: 'MissingRole' })

      await manager.tx.grantRole(ROLE.BORROW_CAP_GUARDIAN, deployer.address)
      await manager.tx.setBorrowCap(poolAddr, 10)

      const { value: value2 } = await controller.query.borrowCap(poolAddr)
      expect(value2.ok).toEqual(10)
    })
    it('.set_mint_guardian_paused', async () => {
      const { deployer, manager, controller } = await setup()
      const poolAddr = encodeAddress(
        '0x0000000000000000000000000000000000000000000000000000000000000000',
      )
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
})
