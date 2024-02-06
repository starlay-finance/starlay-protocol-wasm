/* eslint-disable dot-notation */
import { BN, BN_ONE, BN_TEN } from '@polkadot/util'
import { ONE_ETHER } from '../scripts/helper/constants'
import {
  deployController,
  deployDefaultInterestRateModel,
  deployIncentivesController,
  deployPriceOracle,
} from '../scripts/helper/deploy_helper'
import { getGasLimit } from '../scripts/helper/utils'
import { preparePoolsWithPreparedTokens } from './testContractHelper'
import { shouldNotRevert } from './testHelpers'

const MAX_CALL_WEIGHT = new BN(125_000_000_000).isub(BN_ONE).mul(BN_TEN)
const PROOFSIZE = new BN(2_000_000)
describe('PriceOracle spec', () => {
  const setup = async () => {
    const { api, alice: deployer, bob, charlie, django } = globalThis.setup

    const gasLimit = getGasLimit(api, MAX_CALL_WEIGHT, PROOFSIZE)
    const controller = await deployController({
      api,
      signer: deployer,
      args: [deployer.address],
    })
    const priceOracle = await deployPriceOracle({
      api,
      signer: deployer,
      args: [],
    })

    const rateModel = await deployDefaultInterestRateModel({
      api,
      signer: deployer,
      args: [[0], [0], [0], [0]],
    })

    const incentivesController = await deployIncentivesController({
      api,
      signer: deployer,
      args: [],
    })

    const pools = await preparePoolsWithPreparedTokens({
      api,
      controller,
      rateModel,
      signer: deployer,
      incentivesController,
      manager: deployer.address,
    })

    const users = [bob, charlie, django]

    // for pool
    for (const sym of [pools.dai, pools.usdc, pools.usdt]) {
      await priceOracle.tx.setFixedPrice(sym.token.address, ONE_ETHER)
    }

    return {
      api,
      deployer,
      pools,
      rateModel,
      controller,
      priceOracle,
      users,
      incentivesController,
      gasLimit,
    }
  }

  it('The manager is authorized as sybil default.', async () => {
    const { priceOracle, deployer } = await setup()
    const authorized = (
      await priceOracle.query.isSybilAuthorized(deployer.address)
    ).value.ok

    expect(authorized).toBe(true)
  })

  it('Unauthorized sybil cannot set price.', async () => {
    const { priceOracle, users, pools } = await setup()
    const user = users[0]

    const result = (
      await priceOracle
        .withSigner(user)
        .query.setFixedPrice(pools.dai.token.address, '1000')
    ).value.ok

    expect(result.err).toStrictEqual({ callerIsNotAuthorized: null })
  })

  it('Authorized sybil can set price.', async () => {
    const { priceOracle, users, pools } = await setup()
    const user = users[0]

    /// Authorize sybil
    await shouldNotRevert(priceOracle, 'authorizeSybil', [user.address])

    const price = '100000'
    const token = pools.dai.token.address
    await shouldNotRevert(priceOracle.withSigner(user), 'setFixedPrice', [
      token,
      price,
    ])

    const setPrice = (await priceOracle.query.getPrice(token)).value.ok

    expect(setPrice.toString()).toEqual(price)
  })
})
