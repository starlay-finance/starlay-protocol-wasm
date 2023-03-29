import { ApiPromise } from '@polkadot/api'
import type { KeyringPair } from '@polkadot/keyring/types'
import Controller from '../../types/contracts/controller'
import PriceOracle from '../../types/contracts/price_oracle'
import PSP22Token from '../../types/contracts/psp22_token'
import { Config } from '../config'
import { defaultOption, sendTxWithPreview } from '../helper/utils'
import { DummyToken, TokenConfig } from '../tokens'
import {
  deployDefaultInterestRateModel,
  deployPSP22Token,
  deployPool,
} from './../helper/deploy_helper'

type DeployPoolArgs = {
  api: ApiPromise
  signer: KeyringPair
  tokenConfigs: DummyToken[]
} & Omit<SetupPoolArgs, 'token'>

export const deployPools = async ({
  api,
  signer,
  tokenConfigs,
  ...args
}: DeployPoolArgs): Promise<void> => {
  const tokens = await deployDummyTokens(api, signer, tokenConfigs)
  for (const token of tokens) {
    await deployAndSetupPool(api, signer, { token, ...args })
  }
}

const deployDummyTokens = async (
  api: ApiPromise,
  signer: KeyringPair,
  tokenConfigs: DummyToken[],
) => {
  const res: { token: PSP22Token; config: TokenConfig }[] = []
  for (const config of tokenConfigs) {
    const token = await deployPSP22Token({
      api,
      signer,
      args: [
        config.totalSupply,
        config.name as unknown as string[],
        config.symbol as unknown as string[],
        config.decimals,
      ],
    })
    res.push({ token, config })
  }
  return res
}

type SetupPoolArgs = {
  token: { token: PSP22Token; config: TokenConfig }
  controller: Controller
  priceOracle: PriceOracle
  config: Config
  option: ReturnType<typeof defaultOption>
}
const deployAndSetupPool = async (
  api: ApiPromise,
  signer: KeyringPair,
  {
    token: { token, config },
    controller,
    priceOracle,
    config: { collateralNamePrefix, collateralSymbolPrefix },
    option,
  }: SetupPoolArgs,
) => {
  const rateModelContract = await deployDefaultInterestRateModel({
    api,
    signer,
    args: [
      [config.rateModel.baseRatePerYear],
      [config.rateModel.multiplierPerYearSlope1],
      [config.rateModel.multiplierPerYearSlope2],
      [config.rateModel.kink],
    ],
  })
  const pool = await deployPool({
    api,
    signer,
    args: [
      token.address,
      controller.address,
      rateModelContract.address,
      [config.initialExchangeRateMantissa],
      [collateralNamePrefix + config.name],
      [collateralSymbolPrefix + config.symbol],
      config.decimals,
    ],
  })

  await sendTxWithPreview(priceOracle, 'setFixedPrice', [
    token.address,
    config.price,
    option,
  ])

  await sendTxWithPreview(
    controller,
    'supportMarketWithCollateralFactorMantissa',
    [pool.address, [config.collateralFactor], option],
  )
  await sendTxWithPreview(pool, 'setReserveFactorMantissa', [
    [config.reserveFactor],
    option,
  ])
}
