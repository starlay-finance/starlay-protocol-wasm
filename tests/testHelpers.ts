import { ReturnNumber } from '@727-ventures/typechain-types'
import type { WeightV2 } from '@polkadot/types/interfaces'
import { BN } from '@polkadot/util'
import { ReplacedType } from '../scripts/helper/utilityTypes'
import { waitForTx } from '../scripts/helper/utils'

export const mantissa = () => pow10(18)
export const toDec6 = (value: number): BN => toDec(value, 6)
export const toDec18 = (value: number): BN => toDec(value, 18)

const pow10 = (exponent: number) => new BN(10).pow(new BN(exponent))
const toDec = (value: number, decimals: number): BN =>
  new BN(value).mul(pow10(decimals))

export const expectToEmit = <T = unknown>(
  event: { name: string; args: T },
  name: string,
  args: ReplacedType<T, ReturnNumber, number>,
): void => {
  expect(event.name).toBe(name)
  Object.keys(event.args).forEach((key) => {
    if (event.args[key] instanceof ReturnNumber)
      expect(event.args[key].toNumber()).toBe(args[key])
    else expect(event.args[key]).toBe(args[key])
  })
}

const generateNewArgsWithGasLimit = <
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  C extends { tx: any; query: any; name: string },
  F extends keyof C['tx'],
>(
  contract: C,
  fn: F,
  args: Parameters<C['tx'][F]>,
  gasLimit: WeightV2,
): Parameters<C['tx'][F]> => {
  if (args.length == contract.tx[fn].length) {
    const option: object = args[args.length - 1] as object
    return [
      ...args.slice(0, args.length - 1),
      { ...option, gasLimit, storageDepositLimit: null },
    ] as Parameters<C['tx'][F]>
  } else {
    return [...args, { gasLimit, storageDepositLimit: null }] as Parameters<
      C['tx'][F]
    >
  }
}

// const MAX_CALL_WEIGHT = new BN(5_000_000_000_000_000).isub(BN_ONE)
// const PROOFSIZE = new BN(1_000_000_000)
export const shouldNotRevert = async <
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  C extends { tx: any; query: any; name: string },
  F extends keyof C['tx'],
  R extends ReturnType<C['tx'][F]>,
>(
  contract: C,
  fn: F,
  args: Parameters<C['tx'][F]>,
): Promise<R> => {
  let gasRequired: WeightV2
  const { api } = globalThis.setup
  try {
    const preview = await contract.query[fn](...args)
    expect(preview.value.ok.err).toBeUndefined()
    gasRequired = preview.gasRequired as WeightV2
  } catch (e) {
    throw new Error(
      `failed to preview ${contract.name}.${fn as string}(${JSON.stringify(
        args,
      )}): ${JSON.stringify(e)}`,
    )
  }
  console.log(fn, 'gasRequired.refTime', gasRequired.refTime.toString())
  console.log(fn, 'gasRequired.proofSize', gasRequired.proofSize.toString())
  // const gasLimit = api?.registry.createType('WeightV2', {
  //   refTime: MAX_CALL_WEIGHT,
  //   proofSize: PROOFSIZE,
  // }) as WeightV2
  const gasLimit = api?.registry.createType('WeightV2', gasRequired) as WeightV2
  const newArgs = generateNewArgsWithGasLimit(contract, fn, args, gasLimit)
  const res = await contract.tx[fn](...newArgs)
  await waitForTx(res)
  return res
}
