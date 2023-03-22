import { BN } from '@polkadot/util'

export interface Token {
  symbol: string
  decimal: number
  name: string
  rateModel: InterestRateModel
}
export type DummyToken = Token & DummyTokenProp

export const ONE_ETHER = new BN(10).pow(new BN(18))

interface DummyTokenProp {
  totalSupply: BN
}

interface InterestRateModel {
  baseRatePerYear: BN
  multiplierPerYearSlope1: BN
  multiplierPerYearSlope2: BN
  kink: BN
}

export const SUPPORTED_TOKENS: Token[] = [
  {
    decimal: 18,
    symbol: 'WETH',
    name: 'Wrapped Ether',
    rateModel: {
      baseRatePerYear: new BN(100).mul(ONE_ETHER),
      multiplierPerYearSlope1: new BN(100).mul(ONE_ETHER),
      multiplierPerYearSlope2: new BN(100).mul(ONE_ETHER),
      kink: new BN(100).mul(ONE_ETHER),
    },
  },
]

export const DUMMY_TOKENS: DummyToken[] = SUPPORTED_TOKENS.map((t) => {
  return {
    ...t,
    totalSupply: new BN(10).pow(new BN(18)).mul(new BN(100_000_000_000)),
  }
})
