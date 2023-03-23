import type { ApiPromise } from '@polkadot/api'
import type { KeyringPair } from '@polkadot/keyring/types'

import Controller_factory from '../types/constructors/controller'
import DefaultInterestRateModel_factory from '../types/constructors/default_interest_rate_model'
import Faucet_factory from '../types/constructors/faucet'
import Lens_factory from '../types/constructors/lens'
import Manager_factory from '../types/constructors/manager'
import Pool_factory from '../types/constructors/pool'
import PriceOracle_factory from '../types/constructors/price_oracle'
import PSP22Token_factory from '../types/constructors/psp22_token'

import Controller from '../types/contracts/controller'
import DefaultInterestRateModel from '../types/contracts/default_interest_rate_model'
import Faucet from '../types/contracts/faucet'
import Lens from '../types/contracts/lens'
import Manager from '../types/contracts/manager'
import Pool from '../types/contracts/pool'
import PriceOracle from '../types/contracts/price_oracle'
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
  return new Controller(contract.address, signer, api)
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

export const deployPoolFromAsset = async ({
  api,
  signer,
  args,
}: FactoryArgs & {
  args: Parameters<Pool_factory['newFromAsset']>
}): Promise<Pool> => {
  const factory = new Pool_factory(api, signer)
  const contract = await factory.newFromAsset(...args)
  return new Pool(contract.address, signer, api)
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
  return new DefaultInterestRateModel(contract.address, signer, api)
}

export const deployPriceOracle = async ({
  api,
  signer,
}: FactoryArgs): Promise<PriceOracle> => {
  const factory = new PriceOracle_factory(api, signer)
  const contract = await factory.new()
  return new PriceOracle(contract.address, signer, api)
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
  return new Manager(contract.address, signer, api)
}

export const deployLens = async ({
  api,
  signer,
}: FactoryArgs): Promise<Lens> => {
  const factory = new Lens_factory(api, signer)
  const contract = await factory.new()
  return new Lens(contract.address, signer, api)
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

export const deployFaucet = async ({
  api,
  signer,
}: FactoryArgs): Promise<Faucet> => {
  const factory = new Faucet_factory(api, signer)
  const contract = await factory.new()
  return new Faucet(contract.address, signer, api)
}
