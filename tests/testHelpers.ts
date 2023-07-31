import { ReturnNumber } from '@727-ventures/typechain-types'
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
      `failed to preview ${contract.name}.${fn as string}(${JSON.stringify(
        args,
      )}): ${JSON.stringify(e)}`,
    )
  }

  const res = await contract.tx[fn](...args)
  await waitForTx(res)
  return res
}
