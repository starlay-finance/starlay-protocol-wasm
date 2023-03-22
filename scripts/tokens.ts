import { BN } from '@polkadot/util'
export interface Token {
  symbol: string
  decimal: number
  name: string
}
export type DummyToken = Token & DummyTokenProp

interface DummyTokenProp {
  totalSupply: BN
}

export const SUPPORTED_TOKENS: Token[] = [
  {
    decimal: 18,
    symbol: 'WETH',
    name: 'Wrapped Ether',
  },
]

export const DUMMY_TOKENS: DummyToken[] = SUPPORTED_TOKENS.map((t) => {
  return {
    ...t,
    totalSupply: new BN(10).pow(new BN(18)).mul(new BN(100_000_000_000)),
  }
})
