import type { ApiPromise } from '@polkadot/api'
import type { KeyringPair } from '@polkadot/keyring/types'
import { WeightV2 } from '@polkadot/types/interfaces'
import { BN, BN_ONE } from '@polkadot/util'

import Controller_factory from '../../types/constructors/controller'
import DefaultInterestRateModel_factory from '../../types/constructors/default_interest_rate_model'
import Pool_factory from '../../types/constructors/pool'
import PSP22Token_factory from '../../types/constructors/psp22_token'
import DefaultInterestRateModel from '../../types/contracts/default_interest_rate_model'
import PSP22Token from '../../types/contracts/psp22_token'

import { SignAndSendSuccessResponse } from '@727-ventures/typechain-types'
import Controller from '../../types/contracts/controller'
import Pool from '../../types/contracts/pool'

type FactoryArgs = {
  api: ApiPromise
  signer: KeyringPair
}
const MAX_CALL_WEIGHT = new BN(900_000_000).isub(BN_ONE).mul(new BN(5))
const PROOFSIZE = new BN(1_000_000)
export const getGasLimit = (
  api: ApiPromise,
  refTime?: BN | number,
  proofSize?: BN | number,
): WeightV2 => {
  refTime = refTime || MAX_CALL_WEIGHT
  proofSize = proofSize || PROOFSIZE
  return api.registry.createType('WeightV2', {
    refTime: refTime,
    proofSize: proofSize,
  })
}
export const defaultArgs = (
  api: ApiPromise,
): {
  storageDepositLimit: BN
  gasLimit: WeightV2
} => {
  return {
    storageDepositLimit: new BN(10).pow(new BN(18)),
    gasLimit: getGasLimit(api),
  }
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

export const deployDefaultInterestRateModel = async ({
  api,
  signer,
  args,
}: FactoryArgs & {
  args: Parameters<DefaultInterestRateModel_factory['new']>
}): Promise<DefaultInterestRateModel> => {
  const factory = new DefaultInterestRateModel_factory(api, signer)
  const contract = await factory.new(...args)
  const result = new DefaultInterestRateModel(contract.address, signer, api)
  afterDeployment(result.name, contract)
  return result
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
  const result = new PSP22Token(contract.address, signer, api)
  afterDeployment(`${args[2]}${result.name}`, contract)
  return result
}
