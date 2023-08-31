import { CONFIG } from '../config'
import { setEnv } from '../env'
import {
  defaultOption,
  extractAddressDeep,
  mintNativeToken,
} from '../helper/utils'
import { providerAndSigner } from '../helper/wallet_helper'
import { DUMMY_TOKENS } from '../tokens'
import { ENV } from './../env'
import { deployContracts } from './deploy_contracts'

const main = async () => {
  const env = setEnv(process.argv[2])
  console.log(`Start deploying to: ${env}`)

  const incentivesController = null

  const { api, signer } = await providerAndSigner(env)
  const config = CONFIG
  const option = defaultOption(api)

  const deployments = await deployContracts({
    api,
    signer,
    config,
    tokenConfigs: DUMMY_TOKENS,
    option,
    incentivesController,
  })

  if (env === ENV.local) await mintNativeToken(api, signer, config)

  return {
    env,
    deployments,
  }
}

main()
  .then(({ env, deployments }) => {
    console.log(`Finished deployment for: ${env}`)
    console.log(extractAddressDeep(deployments))
    process.exit(0)
  })
  .catch((e) => {
    console.log(e)
    process.exit(1)
  })
