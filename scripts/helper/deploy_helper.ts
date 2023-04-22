import type { ApiPromise } from '@polkadot/api'
import type { KeyringPair } from '@polkadot/keyring/types'
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
import WETH_factory from '../../types/constructors/weth'
import WETHGateway_factory from '../../types/constructors/weth_gateway'
import DefaultInterestRateModel from '../../types/contracts/default_interest_rate_model'
import PSP22Token from '../../types/contracts/psp22_token'

import { SignAndSendSuccessResponse } from '@727-ventures/typechain-types'
import { LastArrayElement } from 'type-fest'
import Controller from '../../types/contracts/controller'
import Pool from '../../types/contracts/pool'
import Token from '../../types/contracts/psp22_token'
import WETH from '../../types/contracts/weth'
import WETHGateway from '../../types/contracts/weth_gateway'
import { ExcludeLastArrayElement } from './utilityTypes'
import { defaultOption, hexToUtf8, isTest, waitForTx } from './utils'

type FactoryArgs<C extends (...args: unknown[]) => unknown> = {
  api: ApiPromise
  signer: KeyringPair
} & {
  args: ExcludeLastArrayElement<Parameters<C>>
  option?: LastArrayElement<Parameters<C>>
}

const afterDeployment = async (
  name: string,
  contract: {
    result: SignAndSendSuccessResponse
    address: string
  },
) => {
  if (!isTest()) console.log(name + ' was deployed at: ' + contract.address)
  await waitForTx(contract.result)
}

export const deployController = async ({
  api,
  signer,
  args,
  option = defaultOption(api),
}: FactoryArgs<Controller_factory['new']>): Promise<Controller> => {
  const factory = new Controller_factory(api, signer)
  const contract = await factory.new(...args, option)
  const result = new Controller(contract.address, signer, api)
  await afterDeployment(result.name, contract)
  return result
}

export const deployManager = async ({
  api,
  signer,
  args,
  option = defaultOption(api),
}: FactoryArgs<Manager_factory['new']>): Promise<Manager> => {
  const factory = new Manager_factory(api, signer)
  const contract = await factory.new(...args, option)
  const result = new Manager(contract.address, signer, api)
  await afterDeployment(result.name, contract)
  return result
}

export const deployPriceOracle = async ({
  api,
  signer,
  args,
  option = defaultOption(api),
}: FactoryArgs<PriceOracle_factory['new']>): Promise<PriceOracle> => {
  const factory = new PriceOracle_factory(api, signer)
  const contract = await factory.new(...args, option)
  const result = new PriceOracle(contract.address, signer, api)
  await afterDeployment(result.name, contract)
  return result
}

export const deployFaucet = async ({
  api,
  signer,
  args,
  option = defaultOption(api),
}: FactoryArgs<Lens_factory['new']>): Promise<Faucet> => {
  const factory = new Faucet_factory(api, signer)
  const contract = await factory.new(...args, option)
  const result = new Faucet(contract.address, signer, api)
  await afterDeployment(result.name, contract)
  return result
}
export const deployLens = async ({
  api,
  signer,
  args,
  option = defaultOption(api),
}: FactoryArgs<Lens_factory['new']>): Promise<Lens> => {
  const factory = new Lens_factory(api, signer)
  const contract = await factory.new(...args, option)
  const result = new Lens(contract.address, signer, api)
  await afterDeployment(result.name, contract)
  return result
}

export const deployPoolFromAsset = async ({
  api,
  signer,
  args,
  option = defaultOption(api),
  token,
}: FactoryArgs<Pool_factory['newFromAsset']> & {
  token: Token | WETH
}): Promise<Pool> => {
  const factory = new Pool_factory(api, signer)

  // FIXME: calling token_name or token_symbol on contract will fail
  const name = `Starlay ${hexToUtf8(
    (await token.query.tokenName()).value.ok,
  )}` as unknown as string[]
  const symbol = `s${hexToUtf8(
    (await token.query.tokenSymbol()).value.ok,
  )}` as unknown as string[]
  const decimals = (await token.query.tokenDecimals()).value.ok
  const contract = await factory.new(...args, name, symbol, decimals, option)

  const result = new Pool(contract.address, signer, api)
  await afterDeployment(result.name, contract)
  return result
}

export const deployDefaultInterestRateModel = async ({
  api,
  signer,
  args,
  option = defaultOption(api),
}: FactoryArgs<
  DefaultInterestRateModel_factory['new']
>): Promise<DefaultInterestRateModel> => {
  const factory = new DefaultInterestRateModel_factory(api, signer)
  const contract = await factory.new(...args, option)
  const result = new DefaultInterestRateModel(contract.address, signer, api)
  await afterDeployment(result.name, contract)
  return result
}

export const deployPool = async ({
  api,
  signer,
  args,
  option = defaultOption(api),
}: FactoryArgs<Pool_factory['new']>): Promise<Pool> => {
  const factory = new Pool_factory(api, signer)
  const contract = await factory.new(...args, option)
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
  option = defaultOption(api),
}: FactoryArgs<PSP22Token_factory['new']>): Promise<PSP22Token> => {
  const factory = new PSP22Token_factory(api, signer)
  const contract = await factory.new(...args, option)
  const result = new PSP22Token(contract.address, signer, api)
  await afterDeployment(`${args[2]}${result.name}`, contract)
  return result
}

// eslint-disable-next-line @typescript-eslint/naming-convention
export const deployWETHGateway = async ({
  api,
  signer,
  args,
  option = defaultOption(api),
}: FactoryArgs<WETHGateway_factory['new']>): Promise<WETHGateway> => {
  const factory = new WETHGateway_factory(api, signer)
  const contract = await factory.new(...args, option)
  const result = new WETHGateway(contract.address, signer, api)
  await afterDeployment(`${args[0]}${result.name}`, contract)
  return result
}

// eslint-disable-next-line @typescript-eslint/naming-convention
export const deployWETH = async ({
  api,
  signer,
  args,
  option = defaultOption(api),
}: FactoryArgs<WETH_factory['new']>): Promise<WETH> => {
  const factory = new WETH_factory(api, signer)
  const contract = await factory.new(...args, option)
  const result = new WETH(contract.address, signer, api)
  await afterDeployment(`${result.name}`, contract)
  return result
}

// eslint-disable-next-line @typescript-eslint/naming-convention
export const deployWETHPool = async ({
  api,
  signer,
  args,
  option = defaultOption(api),
  token,
}: FactoryArgs<Pool_factory['newFromAsset']> & {
  token: WETH
}): Promise<Pool> => {
  const factory = new Pool_factory(api, signer)

  // FIXME: calling token_name or token_symbol on contract will fail
  const name = `Starlay ${hexToUtf8(
    (await token.query.tokenName()).value.ok,
  )}` as unknown as string[]
  const symbol = `s${hexToUtf8(
    (await token.query.tokenSymbol()).value.ok,
  )}` as unknown as string[]
  const decimals = (await token.query.tokenDecimals()).value.ok
  const contract = await factory.new(...args, name, symbol, decimals, option)

  const result = new Pool(contract.address, signer, api)
  await afterDeployment(result.name, contract)
  return result
}
