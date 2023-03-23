export interface EnvironmentParameter {
  rpc: string
}

const testnetParam: EnvironmentParameter = {
  rpc: 'wss://shibuya.public.blastapi.io',
}

const testParam: EnvironmentParameter = {
  rpc: 'ws://127.0.0.1:9944',
}

export const ENV = {
  testnet: 0,
  test: 1,
} as const

export type Env = (typeof ENV)[keyof typeof ENV]

export const valueOf = (env: Env): EnvironmentParameter => {
  switch (env) {
    case ENV.testnet:
      return testnetParam
    case ENV.test:
      return testParam
    default:
      return testParam
  }
}
