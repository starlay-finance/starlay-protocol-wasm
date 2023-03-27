import { Result, ReturnNumber } from '@727-ventures/typechain-types'
import { encodeAddress } from '@polkadot/keyring'
import { waitForTx } from '../scripts/helper/deploy_helper'
import { ReplacedType } from './utilityTypes'

export const zeroAddress = encodeAddress(
  '0x0000000000000000000000000000000000000000000000000000000000000000',
)

export function parseUnits(amount: bigint | number, decimals = 18): bigint {
  return BigInt(amount) * 10n ** BigInt(decimals)
}

export const hexToUtf8 = (hexArray: number[]): string =>
  Buffer.from(hexArray.toString().replace('0x', ''), 'hex').toString('utf-8')

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

export function revertedWith(
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  result: { value: { err?: any } },
  // eslint-disable-next-line @typescript-eslint/no-explicit-any,@typescript-eslint/explicit-module-boundary-types
  errorTitle: any,
): void {
  if (result.value instanceof Result) {
    result.value = result.value.ok
  }
  if (typeof errorTitle === 'object') {
    expect(result.value).toHaveProperty('err', errorTitle)
  } else {
    expect(result.value.err).toHaveProperty(errorTitle)
  }
}

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
  try {
    const preview = await contract.query[fn](...args)
    expect(preview.value.ok.err).toBeUndefined()
  } catch (e) {
    throw new Error(
      `failed to preview ${contract.name}.${
        fn as string
      }(${args}): ${JSON.stringify(e)}`,
    )
  }
  const res = contract.tx[fn](...args)
  await waitForTx(res)
  return res
}
