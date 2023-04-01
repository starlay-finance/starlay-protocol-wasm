import { ApiPromise } from '@polkadot/api'
import type { KeyringPair } from '@polkadot/keyring/types'
import { Config } from '../config'
import { defaultOption } from '../helper/utils'
import { DummyToken } from '../tokens'
import {
  deployFaucet,
  deployLens,
  deployPriceOracle,
} from './../helper/deploy_helper'
import { deployManagerAndController } from './deploy_manager_and_controller'
import { deployPools } from './deploy_pools'

type DeployContractArgs = {
  api: ApiPromise
  signer: KeyringPair
  config: Config
  tokenConfigs: DummyToken[]
  option: ReturnType<typeof defaultOption>
}

export const deployContracts = async ({
  api,
  signer,
  config,
  tokenConfigs,
  option,
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
  })

  return {
    lens,
    faucet,
    controller,
    manager,
    priceOracle,
    pools,
  }
}
