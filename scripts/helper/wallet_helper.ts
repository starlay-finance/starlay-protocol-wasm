import { ApiPromise, Keyring, WsProvider } from '@polkadot/api'
import type { KeyringPair } from '@polkadot/keyring/types'
import { BN } from '@polkadot/util'
import { waitReady } from '@polkadot/wasm-crypto'
import { ENV, Env, mnemonic, valueOf } from '../env'

export const providerAndSigner = async (
  env: Env,
): Promise<{ api: ApiPromise; signer: KeyringPair }> => {
  const api = await provider(env)
  const signer = await getSigner(env, api)
  return { api, signer }
}

const provider = (env: Env) => {
  const { rpc } = valueOf(env)
  return ApiPromise.create({ provider: new WsProvider(rpc) })
}

const getSigner = async (env: Env, api) => {
  await waitReady()
  const keyring = new Keyring({ type: 'sr25519' })
  const signer = keyring.addFromMnemonic(mnemonic())
  if (env === ENV.local) {
    const alice = keyring.addFromUri('//Alice')

    const amount = new BN('1100000000000000000')
    const transfer = api.tx.balances.transfer(signer.address, amount)
    await transfer.signAndSend(alice)

    return signer
  }
  return signer
}
