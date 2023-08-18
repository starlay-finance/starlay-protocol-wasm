import { ApiPromise, Keyring, WsProvider } from '@polkadot/api'
// Create a new instance of contract
const wsProvider = new WsProvider('ws://127.0.0.1:9944')
// Create a keyring instance
const keyring = new Keyring({ type: 'sr25519' })

const delay = (time: number) => {
  return new Promise((resolve) => setTimeout(resolve, time))
}

export default async function setupApi(): Promise<void> {
  await delay(1000)
  const api = await ApiPromise.create({
    provider: wsProvider,
    throwOnConnect: true,
  }).catch((_) => {
    console.error('Please run swanky-node before executing e2e test')
    process.exit(1)
  })
  const alice = keyring.addFromUri('//Alice')
  const bob = keyring.addFromUri('//Bob')
  const charlie = keyring.addFromUri('//Charlie')
  const django = keyring.addFromUri('//Django')
  const eve = keyring.addFromUri('//Eve')
  const frank = keyring.addFromUri('//Frank')
  globalThis.setup = { api, alice, bob, charlie, django, eve, frank }
}
