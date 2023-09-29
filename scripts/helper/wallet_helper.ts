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
    const { data: aliceBalance } = await api.query.system.account(alice.address)
    console.log('aliceBalance', aliceBalance.free)

    const amount = new BN('1052869610870319726')
    console.log('signer.address', signer.address)
    const transfer = api.tx.balances.transfer(signer.address, amount)
    const result = await transfer.signAndSend(alice)
    console.log('result', JSON.stringify(result))

    const { data: balance } = await api.query.system.account(signer.address)
    console.log('balance', balance.free)

    return signer
  }
  return signer
}
