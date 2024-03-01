import { BN } from '@polkadot/util'
import { ONE_ETHER } from './helper/constants'
import { percent } from './helper/utils'
import { iAssetBase } from './tokens'
export interface RiskParameter {
  collateralFactor: BN
  reserveFactor: BN
  initialExchangeRateMantissa: BN
  liquidationThreshold: BN
}

const param = (col: number, res: number, threshold: number): RiskParameter => {
  return {
    collateralFactor: percent(col),
    initialExchangeRateMantissa: ONE_ETHER,
    reserveFactor: percent(res),
    liquidationThreshold: new BN(threshold * 100),
  }
}

export const RISK_PARAMETERS: iAssetBase<RiskParameter> = {
  wastr: param(40, 20, 55),
  dot: param(65, 20, 70),
  usdc: param(80, 10, 85),
  usdt: param(80, 10, 85),
  dai: param(80, 10, 85),
  weth: param(80, 10, 85),
  matic: param(40, 20, 55),
  bnb: param(40, 20, 55),
}
