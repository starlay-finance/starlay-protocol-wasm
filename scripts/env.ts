import dotenv from 'dotenv'

dotenv.config()

let currentEnv: Env

export interface EnvironmentParameter {
  rpc: string
}

export const ENV = {
  testnet: 0,
  test: 1,
} as const

export type Env = (typeof ENV)[keyof typeof ENV]

export const setEnv = (name: string): Env => {
  currentEnv = ENV[name] ?? ENV.test
  return currentEnv
}

export const getCurrentEnv = (): Env => currentEnv

export const valueOf = (env: Env): EnvironmentParameter =>
  ENV_PARAMS[env] || ENV_PARAMS[ENV.test]

const ENV_PARAMS: Record<Env, EnvironmentParameter> = {
  [ENV.testnet]: {
    rpc: 'wss://shibuya-rpc.dwellir.com',
  },
  [ENV.test]: {
    rpc: 'ws://127.0.0.1:9944',
  },
}

export const mnemonic = (): string => process.env.MNEMONIC
