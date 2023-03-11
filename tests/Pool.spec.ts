import { encodeAddress } from '@polkadot/keyring'
import { deployPool } from './testContractsHelper'
import { zeroAddress } from './testHelpers'

describe('Pool spec', () => {
  const setup = async () => {
    const { api, alice: deployer } = globalThis.setup

    const tokenAddr = encodeAddress(
      '0x0000000000000000000000000000000000000000000000000000000000000001',
    )

    const pool = await deployPool({
      api,
      signer: deployer,
      args: [tokenAddr],
    })

    return { pool, tokenAddr }
  }

  it('instantiate', async () => {
    const { pool, tokenAddr } = await setup()
    expect(pool.address).not.toBe(zeroAddress)
    expect((await pool.query.underlying()).value.ok).toEqual(tokenAddr)
  })
})
