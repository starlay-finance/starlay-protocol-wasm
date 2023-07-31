/* eslint-disable @typescript-eslint/naming-convention */
import { encodeAddress } from '@polkadot/keyring'
import { BN, BN_TEN } from '@polkadot/util'

export const ROLE = {
  DEFAULT_ADMIN_ROLE: 0,
  CONTROLLER_ADMIN: 2873677832,
  TOKEN_ADMIN: 937842313,
  BORROW_CAP_GUARDIAN: 181502825,
  PAUSE_GUARDIAN: 1332676982,
} as const

export const ZERO_ADDRESS = encodeAddress(
  '0x0000000000000000000000000000000000000000000000000000000000000000',
)

export const ONE_ETHER = BN_TEN.pow(new BN(18))
