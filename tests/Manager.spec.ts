import { encodeAddress } from '@polkadot/keyring'
import { deployController, deployManager } from './testContractsHelper'
import { zeroAddress } from './testHelpers'

const Roles = {
  DEFAULT_ADMIN_ROLE: 0,
  CONTROLLER_ADMIN: 2873677832,
  TOKEN_ADMIN: 937842313,
  BORROW_CAP_GUARDIAN: 181502825,
  PAUSE_GUARDIAN: 1332676982,
}

describe('Manager spec', () => {
  const setup = async () => {
    const { api, alice: deployer } = globalThis.setup

    const manager = await deployManager({
      api,
      signer: deployer,
      args: [zeroAddress],
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
      (await manager.query.hasRole(Roles.DEFAULT_ADMIN_ROLE, deployer.address))
        .value.ok,
    ).toBeTruthy

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

      await manager.tx.grantRole(Roles.CONTROLLER_ADMIN, deployer.address)
      await manager.tx.setPriceOracle(oracleAddr)

      // const { value: value2 } = await controller.query.priceOracle()
      // expect(value2.ok).toEqual(oracleAddr)
    })
    it('.set_borrow_cap', async () => {
      const { deployer, manager, controller } = await setup()
      const poolAddr = encodeAddress(
        '0x0000000000000000000000000000000000000000000000000000000000000000',
      )
      const { value: value1 } = await manager.query.setBorrowCap(poolAddr, 10)
      expect(value1.ok.err).toStrictEqual({ accessControl: 'MissingRole' })

      await manager.tx.grantRole(Roles.BORROW_CAP_GUARDIAN, deployer.address)
      await manager.tx.setBorrowCap(poolAddr, 10)

      const { value: value2 } = await controller.query.borrowCap(poolAddr)
      expect(value2.ok).toEqual(10)
    })
    it('.set_price_oracle', async () => {
      const { deployer, manager, controller } = await setup()
      const poolAddr = encodeAddress(
        '0x0000000000000000000000000000000000000000000000000000000000000000',
      )
      const { value: value1 } = await manager.query.setMintGuardianPaused(
        poolAddr,
        true,
      )
      expect(value1.ok.err).toStrictEqual({ accessControl: 'MissingRole' })

      await manager.tx.grantRole(Roles.PAUSE_GUARDIAN, deployer.address)
      await manager.tx.setMintGuardianPaused(poolAddr, true)

      const { value: value2 } = await controller.query.mintGuardianPaused(
        poolAddr,
      )
      expect(value2.ok).toEqual(true)
    })
  })
})
