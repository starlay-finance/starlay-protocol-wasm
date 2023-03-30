import { BN } from '@polkadot/util'
import { ONE_ETHER } from './helper/constants'
import { percent } from './helper/utils'
import { iAssetBase } from './tokens'
export interface RiskParameter {
  collateralFactor: BN
  reserveFactor: BN
  initialExchangeRateMantissa: BN
}

const param = (col: number, res: number): RiskParameter => {
  return {
    collateralFactor: percent(col),
    initialExchangeRateMantissa: ONE_ETHER,
    reserveFactor: percent(res),
  }
}

export const RISK_PARAMETERS: iAssetBase<RiskParameter> = {
  wastr: param(40, 20),
  dot: param(65, 20),
  usdc: param(80, 10),
  usdt: param(80, 10),
  dai: param(80, 10),
  busd: param(80, 10),
  weth: param(80, 10),
  wbtc: param(70, 10),
  matic: param(40, 20),
  bnb: param(40, 20),
  wsdn: param(40, 20),
}
