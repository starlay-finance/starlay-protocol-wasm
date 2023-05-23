import { ApiPromise } from '@polkadot/api'
import type { KeyringPair } from '@polkadot/keyring/types'
import Controller from '../../types/contracts/controller'
import InterestRateModel from '../../types/contracts/default_interest_rate_model'
import Pool from '../../types/contracts/pool'
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
}: DeployPoolArgs): Promise<
  Record<
    string,
    { token: PSP22Token; pool: Pool; interestRateModel: InterestRateModel }
  >
> => {
  const deployments = {}
  const tokens = await deployDummyTokens(api, signer, tokenConfigs)
  for (const token of tokens) {
    const res = await deployAndSetupPool(api, signer, { token, ...args })
    deployments[token.config.symbol] = { token: token.token, ...res }
  }
  return deployments
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
  console.log(
    `---------------- Start setting up pool for ${config.symbol} ----------------`,
  )
  const interestRateModel = await deployDefaultInterestRateModel({
    api,
    signer,
    args: [
      [config.rateModel.baseRatePerYear()],
      [config.rateModel.multiplierPerYearSlope1()],
      [config.rateModel.multiplierPerYearSlope2()],
      [config.rateModel.kink()],
    ],
  })
  const pool = await deployPool({
    api,
    signer,
    args: [
      token.address,
      controller.address,
      interestRateModel.address,
      [config.riskParameter.initialExchangeRateMantissa],
      config.riskParameter.liquidationThreshold,
      [collateralNamePrefix + config.name],
      [collateralSymbolPrefix + config.symbol],
      config.decimals,
    ],
  })

  await sendTxWithPreview(
    priceOracle,
    'setFixedPrice',
    [token.address, config.price],
    option,
  )

  await sendTxWithPreview(
    controller,
    'supportMarketWithCollateralFactorMantissa',
    [pool.address, token.address, [config.riskParameter.collateralFactor]],
    option,
  )
  await sendTxWithPreview(
    pool,
    'setReserveFactorMantissa',
    [[config.riskParameter.reserveFactor]],
    option,
  )
  console.log(`---------------- Finished ----------------`)

  return { pool, interestRateModel }
}
