import { encodeAddress } from '@polkadot/keyring'
import {
  ROLE,
  deployController,
  deployManager,
} from '../scripts/helper/deploy_helper'
import { ZERO_ADDRESS } from '../scripts/helper/utils'
import { ONE_ETHER } from '../scripts/tokens'
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

    // initialize
    await manager.tx.setController(controller.address)

    return { deployer, manager, controller }
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
    it.todo('.support_market_with_collateral_factor_mantissa', async () => {
      const { deployer, manager, controller } = await setup()
      const collateralFactor = ONE_ETHER

      await shouldNotRevert(manager, 'grantRole', [
        ROLE.CONTROLLER_ADMIN,
        deployer.address,
      ])
      // FIXME reverted with: {"issue":"OUTPUT_IS_NULL"}
      await shouldNotRevert(
        manager,
        'supportMarketWithCollateralFactorMantissa',
        [ZERO_ADDRESS, [collateralFactor]],
      )

      const {
        value: { ok: actual },
      } = await controller.query.collateralFactorMantissa(ZERO_ADDRESS)
      expect(actual.rawNumber).toEqual(collateralFactor)
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
