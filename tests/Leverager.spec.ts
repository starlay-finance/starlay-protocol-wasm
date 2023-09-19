/* eslint-disable dot-notation */
import { ApiPromise } from '@polkadot/api'
import type { KeyringPair } from '@polkadot/keyring/types'
import { WeightV2 } from '@polkadot/types/interfaces'
import { BN, BN_ONE, BN_TEN } from '@polkadot/util'
import { ONE_ETHER } from '../scripts/helper/constants'
import {
  deployController,
  deployDefaultInterestRateModel,
  deployLeverager,
  deployPriceOracle,
  deployWETH,
} from '../scripts/helper/deploy_helper'
import { getGasLimit } from '../scripts/helper/utils'

import Controller from '../types/contracts/controller'
import Leverager from '../types/contracts/leverager'
import WETH from '../types/contracts/weth'

import {
  PoolContracts,
  Pools,
  preparePoolsWithPreparedTokens,
} from './testContractHelper'
import { shouldNotRevert } from './testHelpers'

const MAX_CALL_WEIGHT = new BN(128_000_000_000).isub(BN_ONE).mul(BN_TEN)
const PROOFSIZE = new BN(2_000_000)
describe('Leverager spec', () => {
  let api: ApiPromise
  let deployer: KeyringPair
  let users: KeyringPair[]
  let controller: Controller
  let pools: Pools
  let dai: PoolContracts
  let gasLimit: WeightV2
  let leverager: Leverager
  let weth: WETH
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

    const weth = await deployWETH({
      api,
      signer: deployer,
      args: [],
    })

    const pools = await preparePoolsWithPreparedTokens({
      api,
      controller,
      rateModel,
      manager: deployer,
      wethToken: weth,
    })

    const users = [bob, charlie, django]

    // initialize
    await controller.tx.setPriceOracle(priceOracle.address)
    await controller.tx.setCloseFactorMantissa([ONE_ETHER])
    //// for pool
    for (const sym of [pools.dai, pools.weth]) {
      await priceOracle.tx.setFixedPrice(sym.token.address, ONE_ETHER)
      await controller.tx.supportMarketWithCollateralFactorMantissa(
        sym.pool.address,
        sym.token.address,
        [ONE_ETHER.mul(new BN(90)).div(new BN(100))],
      )
    }

    const leverager = await deployLeverager({
      api,
      signer: deployer,
      args: [deployer.address],
    })

    await leverager.tx.initialize(
      controller.address,
      priceOracle.address,
      weth.address,
    )

    return {
      api,
      deployer,
      pools,
      rateModel,
      controller,
      priceOracle,
      users,
      gasLimit,
      leverager,
    }
  }

  beforeAll(async () => {
    ;({ api, deployer, gasLimit, users, controller, pools, leverager } =
      await setup())
    ;({ dai } = pools)
  })

  it('.loop', async () => {
    const depositAmount = 2_000
    await shouldNotRevert(dai.token, 'mint', [
      deployer.address,
      depositAmount,
      { gasLimit },
    ])
    await shouldNotRevert(dai.token, 'approve', [leverager.address, ONE_ETHER])
    await shouldNotRevert(dai.pool, 'approveDelegate', [
      leverager.address,
      ONE_ETHER,
    ])

    await shouldNotRevert(leverager, 'loopAsset', [
      dai.token.address,
      depositAmount,
      5000,
      2,
      { gasLimit },
    ])
  })
})
