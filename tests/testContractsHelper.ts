import { SignAndSendSuccessResponse } from '@727-ventures/typechain-types'
import type { ApiPromise } from '@polkadot/api'
import type { KeyringPair } from '@polkadot/keyring/types'

import Controller_factory from '../types/constructors/controller'
import Pool_factory from '../types/constructors/pool'
import PSP22Token_factory from '../types/constructors/psp22_token'

import Controller from '../types/contracts/controller'
import Pool from '../types/contracts/pool'
import PSP22Token from '../types/contracts/psp22_token'

type FactoryArgs = {
  api: ApiPromise
  signer: KeyringPair
}

export const deployController = async ({
  api,
  signer,
  args,
}: FactoryArgs & {
  args: Parameters<Controller_factory['new']>
}): Promise<Controller> => {
  const factory = new Controller_factory(api, signer)
  const contract = await factory.new(...args)
  const result = new Controller(contract.address, signer, api)
  afterDeployment(result.name, contract)
  return result
}

const afterDeployment = (
  name: string,
  contract: {
    result: SignAndSendSuccessResponse
    address: string
  },
) => {
  console.log(name + ' deployed at:' + contract.address)
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
  const result = new Pool(contract.address, signer, api)
  afterDeployment(result.name, contract)
  return result
}

// Mocks
// eslint-disable-next-line @typescript-eslint/naming-convention
export const deployPSP22Token = async ({
  api,
  signer,
  args,
}: FactoryArgs & {
  args: Parameters<PSP22Token_factory['new']>
}): Promise<PSP22Token> => {
  const factory = new PSP22Token_factory(api, signer)
  const contract = await factory.new(...args)
  return new PSP22Token(contract.address, signer, api)
}
