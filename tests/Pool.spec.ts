import { deployPool } from './testContractsHelper'
import { zeroAddress } from './testHelpers'

describe('Pool spec', () => {
  const setup = async () => {
    const { api, alice: deployer } = globalThis.setup

    const pool = await deployPool({
      api,
      signer: deployer,
      args: [],
    })

    return pool
  }

  it('instantiate', async () => {
    const pool = await setup()
    expect(pool.address).not.toBe(zeroAddress)
  })
})
