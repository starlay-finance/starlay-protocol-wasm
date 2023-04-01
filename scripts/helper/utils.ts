import { SignAndSendSuccessResponse } from '@727-ventures/typechain-types'
import { ApiPromise } from '@polkadot/api'
import type { KeyringPair } from '@polkadot/keyring/types'
import { WeightV2 } from '@polkadot/types/interfaces'
import { BN, BN_ONE } from '@polkadot/util'
import { LastArrayElement } from 'type-fest'
import { Config } from '../config'
import { ENV, getCurrentEnv } from '../env'
import { ONE_ETHER } from './constants'
import { ExcludeLastArrayElement } from './utilityTypes'

const WAIT_FINALIZED_SECONDS = 10000
const MAX_CALL_WEIGHT = new BN(2_000_000_000).isub(BN_ONE).mul(new BN(10))
const PROOFSIZE = new BN(2_000_000)

export const isTest = (): boolean => process.env.NODE_ENV === 'test'

export const percent = (val: number): BN => {
  return new BN(val).mul(ONE_ETHER).div(new BN(100))
}
export const waitForTx = async (
  result: SignAndSendSuccessResponse,
): Promise<void> => {
  if (isTest() || getCurrentEnv() === ENV.local) return

  while (!result.result.isFinalized) {
    await new Promise((resolve) => setTimeout(resolve, WAIT_FINALIZED_SECONDS))
  }
}

export const sendTxWithPreview = async <
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  C extends { tx: any; query: any; name: string },
  F extends keyof C['tx'],
  R extends ReturnType<C['tx'][F]>,
>(
  contract: C,
  fn: F,
  args: ExcludeLastArrayElement<Parameters<C['tx'][F]>>,
  option?: LastArrayElement<Parameters<C['tx'][F]>>,
): Promise<R> => {
  try {
    const preview = await contract.query[fn](...args, option)
    if (preview.value.err) throw preview.value.err
    if (preview.value.ok.err) throw preview.value.ok.err
  } catch (e) {
    throw new Error(
      `Failed to preview: ${toCalldata(
        contract,
        fn,
        ...args,
        option,
      )}): ${JSON.stringify(e)}`,
    )
  }
  const res = await contract.tx[fn](...args, option)
  await waitForTx(res)
  console.log(`Succeeded: ${toCalldata(contract, fn, ...args)}`)
  return res
}

const toCalldata = (
  contract: { name: string },
  fn: unknown,
  ...args: unknown[]
) => `${contract.name}.${fn}(${JSON.stringify(args)})`

export const defaultOption = (
  api: ApiPromise,
): {
  storageDepositLimit: BN
  gasLimit: WeightV2
} => {
  return {
    storageDepositLimit: new BN(10).pow(new BN(18)),
    gasLimit: getGasLimit(api),
  }
}

export const getGasLimit = (
  api: ApiPromise,
  refTime?: BN | number,
  proofSize?: BN | number,
): WeightV2 => {
  refTime = refTime || MAX_CALL_WEIGHT
  proofSize = proofSize || PROOFSIZE
  return api.registry.createType('WeightV2', {
    refTime: refTime,
    proofSize: proofSize,
  })
}

export const hexToUtf8 = (hexArray: number[]): string =>
  Buffer.from(hexArray.toString().replace('0x', ''), 'hex').toString('utf-8')

export const extractAddressDeep = (records: unknown) =>
  Object.keys(records).reduce((res, key) => {
    if ('address' in records[key])
      return {
        ...res,
        [key]: records[key].address,
      }
    if (typeof records[key] === 'object')
      return {
        ...res,
        [key]: extractAddressDeep(records[key]),
      }
    return res
  }, {})

export const mintNativeToken = async (
  api: ApiPromise,
  signer: KeyringPair,
  config: Config,
) => {
  if (!config.mintee) return
  const amount = config.mintAmount || '0xffffffffffffffffffff'
  for (const address of config.mintee) {
    const transfer = api.tx.balances.transfer(address, amount)
    await transfer.signAndSend(signer)
    console.log(`Native token minted: ${amount}@${address}`)
  }
}
