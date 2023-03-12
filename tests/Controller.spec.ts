import { encodeAddress } from '@polkadot/keyring'
import { deployController } from './testContractsHelper'

describe('Controller spec', () => {
  const setup = async () => {
    const { api, alice: deployer } = globalThis.setup

    const controller = await deployController({
      api,
      signer: deployer,
      args: [],
    })

    return { controller }
  }

  it('instantiate', async () => {
    const { controller } = await setup()
    const markets = (await controller.query.markets()).value.ok
    expect(markets.length).toBe(0)
  })

  it('add pool', async () => {
    const { controller } = await setup()

    const tokenAddress = encodeAddress(
      '0x0000000000000000000000000000000000000000000000000000000000000001',
    )

    await controller.tx.supportMarket(tokenAddress)
    const markets = (await controller.query.markets()).value.ok
    expect(markets.length).toBe(1)
    expect(markets[0]).toBe(tokenAddress)
  })
})
