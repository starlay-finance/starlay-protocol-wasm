export interface EnvironmentParameter {
  rpc: string
}

const testnetParam: EnvironmentParameter = {
  rpc: 'wss://shibuya.public.blastapi.io',
}

const ENV = {
  testnet: 0,
} as const

export type Env = (typeof ENV)[keyof typeof ENV]

export const valueOf = (env: Env): EnvironmentParameter => {
  switch (env) {
    case ENV.testnet:
      return testnetParam
    default:
      return testnetParam
  }
}
