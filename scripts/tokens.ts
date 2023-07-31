import { RISK_PARAMETERS, RiskParameter } from './risk_parameters'
/* eslint-disable @typescript-eslint/naming-convention */
import { BN, BN_TEN } from '@polkadot/util'
import { ONE_ETHER } from './helper/constants'
import { InterestRateModel, RATE_MODELS } from './interest_rates'

export interface TokenConfig {
  symbol: string
  decimals: number
  name: string
  rateModel: InterestRateModel
  riskParameter: RiskParameter
  price: BN
}

export interface iAssetBase<T> {
  weth: T
  usdc: T
  usdt: T
  wbtc: T
  wastr: T
  wsdn: T
  dai: T
  busd: T
  matic: T
  bnb: T
  dot: T
}

export type DummyToken = TokenConfig & DummyTokenProp

interface DummyTokenProp {
  totalSupply: BN
}

const price = (val: number) => {
  return ONE_ETHER.mul(new BN(val))
}

export const SUPPORTED_TOKENS: iAssetBase<TokenConfig> = {
  weth: {
    symbol: 'WETH',
    name: 'Wrapped Ether',
    rateModel: RATE_MODELS.weth,
    riskParameter: RISK_PARAMETERS.weth,
    price: price(1792),
    decimals: 18,
  },
  bnb: {
    symbol: 'BNB',
    name: 'Binance Coin',
    rateModel: RATE_MODELS.bnb,
    riskParameter: RISK_PARAMETERS.bnb,
    price: price(313),
    decimals: 18,
  },
  dai: {
    symbol: 'DAI',
    name: 'Dai Stablecoin',
    rateModel: RATE_MODELS.dai,
    riskParameter: RISK_PARAMETERS.dai,
    price: ONE_ETHER,
    decimals: 18,
  },
  usdc: {
    symbol: 'USDC',
    name: 'USD Coin',
    rateModel: RATE_MODELS.usdc,
    riskParameter: RISK_PARAMETERS.usdc,
    decimals: 6,
    price: ONE_ETHER,
  },
  usdt: {
    symbol: 'USDT',
    name: 'Tether USD',
    rateModel: RATE_MODELS.usdt,
    riskParameter: RISK_PARAMETERS.usdt,
    price: ONE_ETHER,
    decimals: 6,
  },
  busd: {
    symbol: 'BUSD',
    name: 'Binance USD',
    rateModel: RATE_MODELS.busd,
    riskParameter: RISK_PARAMETERS.busd,
    price: ONE_ETHER,
    decimals: 18,
  },
  dot: {
    symbol: 'DOT',
    name: 'Polkadot',
    rateModel: RATE_MODELS.busd,
    riskParameter: RISK_PARAMETERS.dot,
    price: price(6),
    decimals: 10,
  },
  matic: {
    symbol: 'MATIC',
    name: 'Matic Token',
    rateModel: RATE_MODELS.matic,
    riskParameter: RISK_PARAMETERS.matic,
    price: ONE_ETHER.mul(new BN(111)).div(new BN(100)),
    decimals: 18,
  },
  wastr: {
    symbol: 'WASTR',
    name: 'Wrapped Astr',
    rateModel: RATE_MODELS.wastr,
    riskParameter: RISK_PARAMETERS.wastr,
    price: price(6).div(new BN(100)),
    decimals: 18,
  },
  wbtc: {
    symbol: 'WBTC',
    name: 'Wrapped Bitcoin',
    rateModel: RATE_MODELS.wbtc,
    riskParameter: RISK_PARAMETERS.wbtc,
    price: price(28363),
    decimals: 8,
  },
  wsdn: {
    symbol: 'WSDN',
    name: 'Wrapped SDN',
    rateModel: RATE_MODELS.wsdn,
    riskParameter: RISK_PARAMETERS.wsdn,
    price: price(3).div(new BN(100)),
    decimals: 18,
  },
}

export const DUMMY_TOKENS: DummyToken[] = Object.values(SUPPORTED_TOKENS).map(
  (t) => {
    return {
      ...t,
      totalSupply: BN_TEN.pow(new BN(18)).mul(new BN(100_000_000_000)),
    }
  },
)
