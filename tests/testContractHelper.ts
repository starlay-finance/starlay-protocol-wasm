import type { ApiPromise } from '@polkadot/api'
import type { KeyringPair } from '@polkadot/keyring/types'
import { ONE_ETHER } from '../scripts/helper/constants'
import {
  deployPoolFromAsset,
  deployPSP22Token,
} from '../scripts/helper/deploy_helper'
import Controller from '../types/contracts/controller'
import DefaultInterestRateModel from '../types/contracts/default_interest_rate_model'
import IncentivesController from '../types/contracts/incentives_controller'
import Pool from '../types/contracts/pool'
import PSP22Token from '../types/contracts/psp22_token'
import WETH from '../types/contracts/weth'

export type Metadata = {
  name: string
  symbol: string
  decimals: number
}
export type PoolContracts = {
  metadata: Metadata
  token: PSP22Token
  pool: Pool
}
export type WrappedPoolContracts = {
  metadata: Metadata
  token: WETH
  pool: Pool
}

export type Pools = {
  [key in (typeof TEST_TOKENS)[number]]?: PoolContracts
} & {
  [key in (typeof TEST_WRAPPED_TOKENS)[number]]?: WrappedPoolContracts
}

export const TEST_TOKENS = ['dai', 'usdc', 'usdt'] as const
export const TEST_WRAPPED_TOKENS = ['weth'] as const

export const TEST_METADATAS: {
  [key in (typeof TEST_TOKENS)[number]]: Metadata
} = {
  dai: {
    name: 'Dai Stablecoin',
    symbol: 'DAI',
    decimals: 18,
  },
  usdc: {
    name: 'USD Coin',
    symbol: 'USDC',
    decimals: 6,
  },
  usdt: {
    name: 'USD Tether',
    symbol: 'USDT',
    decimals: 6,
  },
} as const

export const TEST_WRAPPED_METADATAS: {
  [key in (typeof TEST_WRAPPED_TOKENS)[number]]: Metadata
} = {
  weth: {
    name: 'Wrapped Astar',
    symbol: 'WASTR',
    decimals: 18,
  },
} as const

export const preparePoolWithMockToken = async ({
  api,
  metadata,
  controller,
  rateModel,
  manager,
  incentivesController,
  signer,
}: {
  api: ApiPromise
  metadata: Metadata
  controller: Controller
  rateModel: DefaultInterestRateModel
  signer: KeyringPair
  manager: string
  incentivesController?: IncentivesController
}): Promise<PoolContracts> => {
  const token = await deployPSP22Token({
    api,
    signer,
    args: [0, metadata.name, metadata.symbol, metadata.decimals],
  })

  const pool = await deployPoolFromAsset({
    api,
    signer,
    args: [
      incentivesController ? incentivesController.address : null,
      token.address,
      controller.address,
      rateModel.address,
      manager,
      [ONE_ETHER.toString()],
      10000,
    ],
    token,
  })

  return { metadata, token, pool }
}

// eslint-disable-next-line @typescript-eslint/naming-convention
export const preparePoolWithWETH = async ({
  api,
  metadata,
  controller,
  rateModel,
  manager,
  signer,
  incentivesController,
  token,
}: {
  api: ApiPromise
  metadata: Metadata
  controller: Controller
  rateModel: DefaultInterestRateModel
  signer: KeyringPair
  manager: string
  incentivesController?: IncentivesController
  token: WETH
}): Promise<WrappedPoolContracts> => {
  const pool = await deployPoolFromAsset({
    api,
    signer,
    args: [
      incentivesController ? incentivesController.address : null,
      token.address,
      controller.address,
      rateModel.address,
      manager,
      [ONE_ETHER.toString()],
      10000,
    ],
    token,
  })

  return { metadata, token, pool }
}

export const preparePoolsWithPreparedTokens = async ({
  api,
  controller,
  rateModel,
  manager,
  signer,
  incentivesController,
  wethToken = undefined,
}: {
  api: ApiPromise
  controller: Controller
  rateModel: DefaultInterestRateModel
  incentivesController?: IncentivesController
  signer: KeyringPair
  manager: string
  wethToken?: WETH
}): Promise<Pools> => {
  const dai = await preparePoolWithMockToken({
    api,
    controller,
    rateModel,
    manager,
    signer,
    incentivesController,
    metadata: TEST_METADATAS.dai,
  })
  const usdc = await preparePoolWithMockToken({
    api,
    controller,
    rateModel,
    manager,
    signer,
    incentivesController,
    metadata: TEST_METADATAS.usdc,
  })
  const usdt = await preparePoolWithMockToken({
    api,
    controller,
    rateModel,
    manager,
    signer,
    incentivesController,
    metadata: TEST_METADATAS.usdt,
  })

  if (wethToken == undefined) {
    return { dai, usdc, usdt }
  }
  const weth = await preparePoolWithWETH({
    api,
    metadata: TEST_WRAPPED_METADATAS.weth,
    controller,
    rateModel,
    manager,
    signer,
    incentivesController,
    token: wethToken,
  })
  return { dai, usdc, usdt, weth }
}
