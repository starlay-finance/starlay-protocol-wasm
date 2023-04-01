import { ApiPromise, Keyring, WsProvider } from '@polkadot/api'
import type { KeyringPair } from '@polkadot/keyring/types'
import { waitReady } from '@polkadot/wasm-crypto'
import { ENV, Env, mnemonic, valueOf } from '../env'

export const providerAndSigner = async (
  env: Env,
): Promise<{ api: ApiPromise; signer: KeyringPair }> => {
  const [api, signer] = await Promise.all([provider(env), getSigner(env)])
  return { api, signer }
}

const provider = (env: Env) => {
  const { rpc } = valueOf(env)
  return ApiPromise.create({ provider: new WsProvider(rpc) })
}

const getSigner = async (env: Env) => {
  await waitReady()
  const keyring = new Keyring({ type: 'sr25519' })
  if (env === ENV.local) return keyring.addFromUri('//Alice')
  return keyring.addFromMnemonic(mnemonic())
}
