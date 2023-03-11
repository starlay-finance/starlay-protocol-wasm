import type { ApiPromise } from '@polkadot/api'
import type { KeyringPair } from '@polkadot/keyring/types'

import Pool_factory from '../types/constructors/pool'
import Pool from '../types/contracts/pool'

type FactoryArgs = {
  api: ApiPromise
  signer: KeyringPair
}

export const deployPool = async ({
  api,
  signer,
  args,
}: FactoryArgs & {
  args: Parameters<Pool_factory['new']>
}): Promise<Pool> => {
  const factory = new Pool_factory(api, signer)
  const contract = await factory.new(...args)
  return new Pool(contract.address, signer, api)
}
