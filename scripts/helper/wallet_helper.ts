import { ApiPromise, Keyring, WsProvider } from '@polkadot/api'
import type { KeyringPair } from '@polkadot/keyring/types'
import { waitReady } from '@polkadot/wasm-crypto'
import dotenv from 'dotenv'
import { Env, valueOf } from '../env'

dotenv.config()

export const providerAndSigner = async (
  env: Env,
): Promise<{ api: ApiPromise; signer: KeyringPair }> => {
  const [api, signer] = await Promise.all([provider(env), getSigner()])
  return { api, signer }
}

const provider = (env: Env) => {
  const { rpc } = valueOf(env)
  return ApiPromise.create({ provider: new WsProvider(rpc) })
}

const getSigner = () => {
  return fromDotEnv()
}

const fromDotEnv = () => {
  const mnemonic = process.env.MNEMONIC
  return waitReady().then(() =>
    new Keyring({ type: 'sr25519' }).addFromMnemonic(mnemonic),
  )
}
