import type { ApiPromise } from '@polkadot/api'
import type { KeyringPair } from '@polkadot/keyring/types'
import { ONE_ETHER } from '../scripts/helper/constants'
import {
  deployPoolFromAsset,
  deployPSP22Token,
} from '../scripts/helper/deploy_helper'
import Controller from '../types/contracts/controller'
import DefaultInterestRateModel from '../types/contracts/default_interest_rate_model'
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
}: {
  api: ApiPromise
  metadata: Metadata
  controller: Controller
  rateModel: DefaultInterestRateModel
  manager: KeyringPair
}): Promise<PoolContracts> => {
  const token = await deployPSP22Token({
    api,
    signer: manager,
    args: [
      0,
      metadata.name as unknown as string[],
      metadata.symbol as unknown as string[],
      metadata.decimals,
    ],
  })

  const pool = await deployPoolFromAsset({
    api,
    signer: manager,
    args: [
      token.address,
      controller.address,
      rateModel.address,
      [ONE_ETHER.toString()],
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
  token,
}: {
  api: ApiPromise
  metadata: Metadata
  controller: Controller
  rateModel: DefaultInterestRateModel
  manager: KeyringPair
  token: WETH
}): Promise<WrappedPoolContracts> => {
  const pool = await deployPoolFromAsset({
    api,
    signer: manager,
    args: [
      token.address,
      controller.address,
      rateModel.address,
      [ONE_ETHER.toString()],
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
  wethToken = undefined,
}: {
  api: ApiPromise
  controller: Controller
  rateModel: DefaultInterestRateModel
  manager: KeyringPair
  wethToken?: WETH
}): Promise<Pools> => {
  const dai = await preparePoolWithMockToken({
    api,
    controller,
    rateModel,
    manager: manager,
    metadata: TEST_METADATAS.dai,
  })
  const usdc = await preparePoolWithMockToken({
    api,
    controller,
    rateModel,
    manager: manager,
    metadata: TEST_METADATAS.usdc,
  })
  const usdt = await preparePoolWithMockToken({
    api,
    controller,
    rateModel,
    manager: manager,
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
    token: wethToken,
  })
  return { dai, usdc, usdt, weth }
}
