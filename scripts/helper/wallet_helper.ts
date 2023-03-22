import { Keyring } from '@polkadot/api'
/* eslint-disable @typescript-eslint/explicit-module-boundary-types */
import { ApiPromise, WsProvider } from '@polkadot/api'
import { Env, valueOf } from '../env'
// eslint-disable-next-line @typescript-eslint/no-var-requires
require('dotenv').config()

// eslint-disable-next-line @typescript-eslint/naming-convention, @typescript-eslint/no-unused-vars
export const deployer = (env: Env) => {
  return fromDotEnv()
}

const fromDotEnv = () => {
  const mnemonic = process.env.MNEMONIC
  return new Keyring({ type: 'sr25519' }).addFromMnemonic(mnemonic)
}

export const provider = async (env: Env): Promise<ApiPromise> => {
  const { rpc } = valueOf(env)
  return await ApiPromise.create({ provider: new WsProvider(rpc) })
}
