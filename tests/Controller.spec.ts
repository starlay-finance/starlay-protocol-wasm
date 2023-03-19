import { encodeAddress } from '@polkadot/keyring'
import BN from 'bn.js'
import { deployController } from './testContractsHelper'
import { zeroAddress } from './testHelpers'

describe('Controller spec', () => {
  const setup = async () => {
    const { api, alice: deployer } = globalThis.setup

    const controller = await deployController({
      api,
      signer: deployer,
      args: [deployer.address],
    })

    return { controller }
  }

  it('instantiate', async () => {
    const { controller } = await setup()
    const markets = (await controller.query.markets()).value.ok
    expect(markets.length).toBe(0)
    expect((await controller.query.oracle()).value.ok).toEqual(zeroAddress)
    const closeFactorMantissa = (await controller.query.closeFactorMantissa())
      .value.ok
    expect(closeFactorMantissa.toNumber()).toEqual(0)
    const liquidationIncentiveMantissa = (
      await controller.query.liquidationIncentiveMantissa()
    ).value.ok
    expect(liquidationIncentiveMantissa.toNumber()).toEqual(0)
  })

  it('.set_close_factor_mantissa', async () => {
    const { controller } = await setup()
    const expScale = new BN(10).pow(new BN(18))
    const bn = expScale.mul(new BN(5)).div(new BN(100)) // 5%
    await controller.tx.setCloseFactorMantissa([bn])
    const after = (await controller.query.closeFactorMantissa()).value.ok
    expect(bn.toString()).toEqual(BigInt(after.toString()).toString())
  })

  it('.liquidation_incentive_mantissa', async () => {
    const { controller } = await setup()
    const expScale = new BN(10).pow(new BN(18))
    const bn = expScale.mul(new BN(5)).div(new BN(100)) // 5%
    await controller.tx.setLiquidationIncentiveMantissa([bn])
    const after = (await controller.query.liquidationIncentiveMantissa()).value
      .ok
    expect(bn.toString()).toEqual(BigInt(after.toString()).toString())
  })

  it('.support_market', async () => {
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
