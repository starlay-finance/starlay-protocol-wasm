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
    await manager.withSigner(deployer).tx.setController(controller.address)

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
})
