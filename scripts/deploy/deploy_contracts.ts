import { ApiPromise } from '@polkadot/api'
import type { KeyringPair } from '@polkadot/keyring/types'
import { Config } from '../config'
import { defaultOption } from '../helper/utils'
import { DummyToken } from '../tokens'
import {
  deployFaucet,
  deployFlashLoanGateway,
  deployLens,
  deployLeverager,
  deployPriceOracle,
  deployWETHGateway,
} from './../helper/deploy_helper'
import { deployManagerAndController } from './deploy_manager_and_controller'
import { deployPools } from './deploy_pools'

type DeployContractArgs = {
  api: ApiPromise
  signer: KeyringPair
  config: Config
  tokenConfigs: DummyToken[]
  option: ReturnType<typeof defaultOption>
  incentivesController: string | null
}

export const deployContracts = async ({
  api,
  signer,
  config,
  tokenConfigs,
  option,
  incentivesController,
}: DeployContractArgs) => {
  const lens = await deployLens({ api, signer, args: [] })
  const faucet = await deployFaucet({ api, signer, args: [] })
  const priceOracle = await deployPriceOracle({ api, signer, args: [] })

  const { manager, controller } = await deployManagerAndController({
    api,
    signer,
    priceOracle,
    config,
    option,
  })

  const pools = await deployPools({
    api,
    signer,
    tokenConfigs,
    controller,
    priceOracle,
    config,
    option,
    incentivesController,
  })

  const wethGateway = await deployWETHGateway({
    api,
    signer,
    args: [pools.WASTR.token.address, pools.WASTR.pool.address],
  })

  const flashloanGateway = await deployFlashLoanGateway({
    api,
    signer,
    args: [controller.address],
  })
  await controller.tx.setFlashloanGateway(flashloanGateway.address)

  const leverager = await deployLeverager({
    api,
    signer,
    args: [signer.address],
  })

  return {
    lens,
    faucet,
    controller,
    manager,
    priceOracle,
    pools,
    wethGateway,
    flashloanGateway,
    leverager,
  }
}
