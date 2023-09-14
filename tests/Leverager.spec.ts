/* eslint-disable dot-notation */
import { ApiPromise } from '@polkadot/api'
import type { KeyringPair } from '@polkadot/keyring/types'
import { WeightV2 } from '@polkadot/types/interfaces'
import { BN, BN_ONE, BN_TEN } from '@polkadot/util'
import { ONE_ETHER } from '../scripts/helper/constants'
import {
  deployController,
  deployDefaultInterestRateModel,
  deployIncentivesController,
  deployPriceOracle,
} from '../scripts/helper/deploy_helper'
import { getGasLimit } from '../scripts/helper/utils'
import Contract from '../types/contracts/default_interest_rate_model'
import { Pools, preparePoolsWithPreparedTokens } from './testContractHelper'

const MAX_CALL_WEIGHT = new BN(125_000_000_000).isub(BN_ONE).mul(BN_TEN)
const PROOFSIZE = new BN(2_000_000)
describe('Pool spec 1', () => {
  let api: ApiPromise
  let deployer: KeyringPair
  let users: KeyringPair[]
  let controller: Controller
  let pools: Pools
  let usdt: PoolContracts
  let usdc: PoolContracts
  let dai: PoolContracts
  let incentivesController: IncentivesController
  let gasLimit: WeightV2
  const setup = async (model?: Contract) => {
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

    const rateModel = model
      ? model
      : await deployDefaultInterestRateModel({
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
      manager: deployer,
      incentivesController,
    })

    const users = [bob, charlie, django]

    // initialize
    await controller.tx.setPriceOracle(priceOracle.address)
    await controller.tx.setCloseFactorMantissa([ONE_ETHER])
    //// for pool
    for (const sym of [pools.dai, pools.usdc, pools.usdt]) {
      await priceOracle.tx.setFixedPrice(sym.token.address, ONE_ETHER)
      await controller.tx.supportMarketWithCollateralFactorMantissa(
        sym.pool.address,
        sym.token.address,
        [ONE_ETHER.mul(new BN(90)).div(new BN(100))],
      )
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

  beforeAll(async () => {
    ;({
      api,
      deployer,
      gasLimit,
      users,
      controller,
      pools,
      incentivesController,
    } = await setup())
    ;({ usdt, usdc, dai } = pools)
  })
})
