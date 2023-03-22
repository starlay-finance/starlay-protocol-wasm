import { ApiPromise } from '@polkadot/api'
import type { KeyringPair } from '@polkadot/keyring/types'
import { deployController } from '../../tests/testContractsHelper'
import PSP22Token from '../../types/contracts/psp22_token'
import { deployer, provider } from '../helper/wallet_helper'
import { DUMMY_TOKENS, Token } from '../tokens'
import { Env } from './../env'
import {
  defaultArgs,
  deployDefaultInterestRateModel,
  deployPool,
  deployPSP22Token,
} from './../helper/deploy_helper'

const main = async () => {
  await deployContracts(0)
}
const deployContracts = async (env: Env) => {
  const api = await provider(env)
  const signer = await deployer(env)
  const controller = await deployController({
    api,
    signer,
    args: [defaultArgs(api)],
  })
  for (const token of await deployDummyTokens(api, signer)) {
    await deployDefaultInterestRateModel({
      api,
      signer,
      args: [defaultArgs(api)],
    })
    await deployPool({
      api,
      signer,
      args: [
        token.contract.address,
        controller.address,
        [resolvePoolName(token.token.name)],
        [token.token.symbol],
        token.token.decimal,
        defaultArgs(api),
      ],
    })
  }
}

const resolvePoolName = (token: string) => {
  return `${token}Pool`
}
const deployDummyTokens = async (api: ApiPromise, signer: KeyringPair) => {
  const res: { contract: PSP22Token; token: Token }[] = []
  for (const token of DUMMY_TOKENS) {
    const deployed = await deployPSP22Token({
      api,
      signer,
      args: [
        token.totalSupply,
        [token.name],
        [token.symbol],
        token.decimal,
        defaultArgs(api),
      ],
    })
    res.push({ contract: deployed, token: token })
  }
  return res
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
