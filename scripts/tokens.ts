import { BN } from '@polkadot/util'
import { ONE_ETHER } from './helper/constants'

export interface Token {
  symbol: string
  decimal: number
  name: string
  rateModel: InterestRateModel
  collateralFator: BN
  price: BN
}
export type DummyToken = Token & DummyTokenProp


interface DummyTokenProp {
  totalSupply: BN
}

interface InterestRateModel {
  baseRatePerYear: BN
  multiplierPerYearSlope1: BN
  multiplierPerYearSlope2: BN
  kink: BN
}

const TOKEN_BASE: Omit<Token, 'symbol' | 'name'> = {
  decimal: 18,
  rateModel: {
    baseRatePerYear: new BN(100).mul(ONE_ETHER),
    multiplierPerYearSlope1: new BN(100).mul(ONE_ETHER),
    multiplierPerYearSlope2: new BN(100).mul(ONE_ETHER),
    kink: new BN(100).mul(ONE_ETHER),
  },
  collateralFator: ONE_ETHER.mul(new BN(90)).div(new BN(100)),
  price: ONE_ETHER,
}

export const SUPPORTED_TOKENS: Token[] = [
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
