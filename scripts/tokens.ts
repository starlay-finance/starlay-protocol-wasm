import { RiskParameter, RISK_PARAMETERS } from './risk_parameters'
/* eslint-disable @typescript-eslint/naming-convention */
import { BN } from '@polkadot/util'
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

const TOKEN_BASE = {
  decimals: 18,
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
    ...TOKEN_BASE,
  },
  bnb: {
    symbol: 'BNB',
    name: 'Binance Coin',
    rateModel: RATE_MODELS.bnb,
    riskParameter: RISK_PARAMETERS.bnb,
    price: price(313),
    ...TOKEN_BASE,
  },
  dai: {
    symbol: 'DAI',
    name: 'Dai Stablecoin',
    rateModel: RATE_MODELS.dai,
    riskParameter: RISK_PARAMETERS.dai,
    price: ONE_ETHER,
    ...TOKEN_BASE,
  },
  usdc: {
    symbol: 'USDC',
    name: 'USD Coin',
    rateModel: RATE_MODELS.usdc,
    riskParameter: RISK_PARAMETERS.usdc,
    decimals: 6,
    price: ONE_ETHER,
    ...TOKEN_BASE,
  },
  usdt: {
    symbol: 'USDT',
    name: 'Tether USD',
    rateModel: RATE_MODELS.usdt,
    riskParameter: RISK_PARAMETERS.usdt,
    price: ONE_ETHER,
    ...TOKEN_BASE,
    decimals: 6,
  },
  busd: {
    symbol: 'BUSD',
    name: 'Binance USD',
    rateModel: RATE_MODELS.busd,
    riskParameter: RISK_PARAMETERS.busd,
    price: ONE_ETHER,
    ...TOKEN_BASE,
  },
  dot: {
    symbol: 'DOT',
    name: 'Polkadot',
    rateModel: RATE_MODELS.busd,
    riskParameter: RISK_PARAMETERS.dot,
    ...TOKEN_BASE,
    price: price(6),
    decimals: 10,
  },
  matic: {
    symbol: 'MATIC',
    name: 'Matic Token',
    rateModel: RATE_MODELS.matic,
    riskParameter: RISK_PARAMETERS.matic,
    price: ONE_ETHER.mul(new BN(111)).div(new BN(100)),
    ...TOKEN_BASE,
  },
  wastr: {
    symbol: 'WASTR',
    name: 'Wrapped Astr',
    rateModel: RATE_MODELS.wastr,
    riskParameter: RISK_PARAMETERS.wastr,
    price: price(6).div(new BN(100)),
    ...TOKEN_BASE,
  },
  wbtc: {
    symbol: 'WBTC',
    name: 'Wrapped Bitcoin',
    rateModel: RATE_MODELS.wbtc,
    riskParameter: RISK_PARAMETERS.wbtc,
    ...TOKEN_BASE,
    price: price(28363),
    decimals: 8,
  },
  wsdn: {
    symbol: 'WSDN',
    name: 'Wrapped SDN',
    rateModel: RATE_MODELS.wsdn,
    riskParameter: RISK_PARAMETERS.wsdn,
    price: price(3).div(new BN(100)),
    ...TOKEN_BASE,
  },
}

export const DUMMY_TOKENS: DummyToken[] = Object.values(SUPPORTED_TOKENS).map(
  (t) => {
    return {
      ...t,
      totalSupply: new BN(10).pow(new BN(18)).mul(new BN(100_000_000_000)),
    }
  },
)
