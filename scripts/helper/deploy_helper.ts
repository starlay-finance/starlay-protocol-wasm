import type { ApiPromise } from '@polkadot/api'
import type { KeyringPair } from '@polkadot/keyring/types'
import { WeightV2 } from '@polkadot/types/interfaces'
import { BN, BN_ONE } from '@polkadot/util'
import Faucet_factory from '../../types/constructors/faucet'
import Lens_factory from '../../types/constructors/lens'
import Manager_factory from '../../types/constructors/manager'
import PriceOracle_factory from '../../types/constructors/price_oracle'
import Faucet from '../../types/contracts/faucet'
import Lens from '../../types/contracts/lens'
import Manager from '../../types/contracts/manager'
import PriceOracle from '../../types/contracts/price_oracle'

import Controller_factory from '../../types/constructors/controller'
import DefaultInterestRateModel_factory from '../../types/constructors/default_interest_rate_model'
import Pool_factory from '../../types/constructors/pool'
import PSP22Token_factory from '../../types/constructors/psp22_token'
import DefaultInterestRateModel from '../../types/contracts/default_interest_rate_model'
import PSP22Token from '../../types/contracts/psp22_token'

import { SignAndSendSuccessResponse } from '@727-ventures/typechain-types'
import { encodeAddress } from '@polkadot/keyring'
import Controller from '../../types/contracts/controller'
import Pool from '../../types/contracts/pool'

type FactoryArgs = {
  api: ApiPromise
  signer: KeyringPair
}

const WAIT_FINALIZED_SECONDS = 10000

export const ZERO_ADDRESS = encodeAddress(
  '0x0000000000000000000000000000000000000000000000000000000000000000',
)
const MAX_CALL_WEIGHT = new BN(900_000_000).isub(BN_ONE).mul(new BN(10))
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

export const ROLE = {
  DEFAULT_ADMIN_ROLE: 0,
  CONTROLLER_ADMIN: 2873677832,
  TOKEN_ADMIN: 937842313,
  BORROW_CAP_GUARDIAN: 181502825,
  PAUSE_GUARDIAN: 1332676982,
} as const

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
  await afterDeployment(result.name, contract)
  return result
}
export const deployManager = async ({
  api,
  signer,
  args,
}: FactoryArgs & {
  args: Parameters<Manager_factory['new']>
}): Promise<Manager> => {
  const factory = new Manager_factory(api, signer)
  const contract = await factory.new(...args)
  const result = new Manager(contract.address, signer, api)
  await afterDeployment(result.name, contract)
  return result
}

export const deployPriceOracle = async ({
  api,
  signer,
  args,
}: FactoryArgs & {
  args: Parameters<PriceOracle_factory['new']>
}): Promise<PriceOracle> => {
  const factory = new PriceOracle_factory(api, signer)
  const contract = await factory.new(...args)
  const result = new PriceOracle(contract.address, signer, api)
  await afterDeployment(result.name, contract)
  return result
}
const afterDeployment = async (
  name: string,
  contract: {
    result: SignAndSendSuccessResponse
    address: string
  },
) => {
  console.log(name + ' was deployed at: ' + contract.address)
  await waitForTx(contract.result)
}

export const waitForTx = async (
  result: SignAndSendSuccessResponse,
): Promise<void> => {
  if (isTestEnv(result)) return

  while (!result.result.isFinalized) {
    await new Promise((resolve) => setTimeout(resolve, WAIT_FINALIZED_SECONDS))
  }
}

const isTestEnv = (result: SignAndSendSuccessResponse) => {
  // TODO
  return result.result.blockNumber.toNumber() < 1000
}

export const deployFaucet = async ({
  api,
  signer,
  args,
}: FactoryArgs & {
  args: Parameters<Lens_factory['new']>
}): Promise<Faucet> => {
  const factory = new Faucet_factory(api, signer)
  const contract = await factory.new(...args)
  const result = new Faucet(contract.address, signer, api)
  await afterDeployment(result.name, contract)
  return result
}
export const deployLens = async ({
  api,
  signer,
  args,
}: FactoryArgs & {
  args: Parameters<Lens_factory['new']>
}): Promise<Lens> => {
  const factory = new Lens_factory(api, signer)
  const contract = await factory.new(...args)
  const result = new Lens(contract.address, signer, api)
  await afterDeployment(result.name, contract)
  return result
}

export const deployPoolFromAsset = async ({
  api,
  signer,
  args,
}: FactoryArgs & {
  args: Parameters<Pool_factory['newFromAsset']>
}): Promise<Pool> => {
  const factory = new Pool_factory(api, signer)
  const contract = await factory.newFromAsset(...args)
  const result = new Pool(contract.address, signer, api)
  await afterDeployment(result.name, contract)
  return result
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
  await afterDeployment(result.name, contract)
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
  await afterDeployment(result.name, contract)
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
  await afterDeployment(`${args[2]}${result.name}`, contract)
  return result
}
