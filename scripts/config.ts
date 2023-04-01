import { BN } from '@polkadot/util'
import { ONE_ETHER, ROLE } from './helper/constants'

export type Config = {
  liquidationIncentive: BN
  closeFactor: BN
  collateralNamePrefix: string
  collateralSymbolPrefix: string
  roleGrantees?: Partial<
    Record<Exclude<keyof typeof ROLE, 'DEFAULT_ADMIN_ROLE'>, string>
  >
  // for local only
  mintee?: string[]
  mintAmount?: string
}

export const CONFIG: Config = {
  liquidationIncentive: ONE_ETHER.mul(new BN(10)).div(new BN(90)),
  closeFactor: ONE_ETHER,
  collateralNamePrefix: 'Starlay ',
  collateralSymbolPrefix: 'l',
}
