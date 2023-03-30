import { CONFIG } from '../config'
import { setEnv } from '../env'
import { defaultOption } from '../helper/utils'
import { providerAndSigner } from '../helper/wallet_helper'
import { DUMMY_TOKENS } from '../tokens'
import { deployContracts } from './deploy_contracts'

const main = async () => {
  const env = setEnv(process.argv[2])
  console.log(`env: ${env}`)
  const { api, signer } = await providerAndSigner(env)
  const option = defaultOption(api)
  await deployContracts({
    api,
    signer,
    config: CONFIG,
    tokenConfigs: DUMMY_TOKENS,
    option,
  })
}

main()
  .then(() => {
    console.log('finish')
    process.exit(0)
  })
  .catch((e) => {
    console.log(e)
    process.exit(1)
  })
