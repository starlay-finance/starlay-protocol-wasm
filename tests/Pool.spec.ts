import { ApiPromise } from '@polkadot/api'
import { KeyringPair } from '@polkadot/keyring/types'
import Pool_factory from '../types/constructors/pool'
import Pool from '../types/contracts/pool'
import { zeroAddress } from './testHelpers'

describe('Pool spec', () => {
  let api: ApiPromise
  let deployer: KeyringPair
  let factory: Pool_factory

  const setup = () => {
    ;({ api, alice: deployer } = globalThis.setup)

    factory = new Pool_factory(api, deployer)
  }

  beforeAll(async () => {
    await setup()
  })

  it('instantiate', async () => {
    const pool = new Pool((await factory.new()).address, deployer, api)
    expect(pool.address).not.toBe(zeroAddress)
  })
})
