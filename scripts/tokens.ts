import { BN } from '@polkadot/util'
import { ONE_ETHER } from './helper/constants'

export interface TokenConfig {
  symbol: string
  decimals: number
  name: string
  rateModel: InterestRateModel
  collateralFactor: BN
  reserveFactor: BN
  initialExchangeRateMantissa: BN
  price: BN
}

export type DummyToken = TokenConfig & DummyTokenProp

interface DummyTokenProp {
  totalSupply: BN
}

interface InterestRateModel {
  baseRatePerYear: BN
  multiplierPerYearSlope1: BN
  multiplierPerYearSlope2: BN
  kink: BN
}

const TOKEN_BASE: Omit<TokenConfig, 'symbol' | 'name'> = {
  decimals: 18,
  rateModel: {
    baseRatePerYear: new BN(100).mul(ONE_ETHER),
    multiplierPerYearSlope1: new BN(100).mul(ONE_ETHER),
    multiplierPerYearSlope2: new BN(100).mul(ONE_ETHER),
    kink: new BN(100).mul(ONE_ETHER),
  },
  collateralFactor: ONE_ETHER.mul(new BN(90)).div(new BN(100)),
  reserveFactor: ONE_ETHER.mul(new BN(10)).div(new BN(100)),
  initialExchangeRateMantissa: ONE_ETHER,
  price: ONE_ETHER,
}

export const SUPPORTED_TOKENS: TokenConfig[] = [
  { symbol: 'WETH', name: 'Wrapped Ether', ...TOKEN_BASE },
  // { symbol: 'WBTC', name: 'Wrapped Bitcoin', ...TOKEN_BASE },
  // { symbol: 'DAI', name: 'Dai Stablecoin', ...TOKEN_BASE },
  // { symbol: 'USDC', name: 'USD Coin', ...TOKEN_BASE, decimal: 6 },
  // { symbol: 'ceUSDT', name: 'Tether USD', ...TOKEN_BASE, decimal: 6 },
  // { symbol: 'USDT', name: 'Native Tether USD', ...TOKEN_BASE, decimal: 6 },
]

export const DUMMY_TOKENS: DummyToken[] = SUPPORTED_TOKENS.map((t) => {
  return {
    ...t,
    totalSupply: new BN(10).pow(new BN(18)).mul(new BN(100_000_000_000)),
  }
})
