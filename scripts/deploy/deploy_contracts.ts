import { ApiPromise } from '@polkadot/api'
import type { KeyringPair } from '@polkadot/keyring/types'
import { BN } from '@polkadot/util'
import PSP22Token from '../../types/contracts/psp22_token'
import { deployer, provider } from '../helper/wallet_helper'
import { DUMMY_TOKENS, Token } from '../tokens'
import { Env } from './../env'
import {
  defaultArgs,
  deployController,
  deployDefaultInterestRateModel,
  deployLens,
  deployManager,
  deployPool,
  deployPSP22Token,
  waitForTx,
  ZERO_ADDRESS,
} from './../helper/deploy_helper'

const main = async () => {
  await deployContracts(0)
}
const deployContracts = async (env: Env) => {
  const api = await provider(env)
  const signer = await deployer(env)
  const args = defaultArgs(api)
  const manager = await deployManager({
    api,
    signer,
    args: [ZERO_ADDRESS, args],
  })
  const controller = await deployController({
    api,
    signer,
    args: [manager.address, args],
  })
  await waitForTx(await manager.tx.setController(controller.address, args))
  for (const token of await deployDummyTokens(api, signer)) {
    const {
      baseRatePerYear,
      multiplierPerYearSlope1,
      multiplierPerYearSlope2,
      kink,
    } = token.token.rateModel
    const toParam = (m: BN) => [m.toString()]

    const rateModelContract = await deployDefaultInterestRateModel({
      api,
      signer,
      args: [
        toParam(baseRatePerYear),
        toParam(multiplierPerYearSlope1),
        toParam(multiplierPerYearSlope2),
        toParam(kink),
        args,
      ],
    })
    await deployPool({
      api,
      signer,
      args: [
        token.contract.address,
        controller.address,
        rateModelContract.address,
        [resolvePoolName(token.token.name)],
        [token.token.symbol],
        token.token.decimal,
        args,
      ],
    })
  }
  await deployLens({ api, signer, args: [args] })
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
