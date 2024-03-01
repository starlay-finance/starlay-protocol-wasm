import { BN } from '@polkadot/util'
import { ONE_ETHER } from './helper/constants'
import { percent } from './helper/utils'
import { iAssetBase } from './tokens'
export interface InterestRateModel {
  baseRatePerYear: () => BN
  multiplierPerYearSlope1: () => BN
  multiplierPerYearSlope2: () => BN
  kink: () => BN
}

class StarlayInterestRateModel implements InterestRateModel {
  constructor(
    baseRate: BN,
    slope1: BN,
    slope2: BN,
    optimalUtilizationRate: BN,
  ) {
    this.baseRate = baseRate
    this.slope1 = slope1
    this.slope2 = slope2
    this.optimalUtilRate = optimalUtilizationRate
  }
  private readonly baseRate: BN
  private readonly slope1: BN
  private readonly slope2: BN
  private readonly optimalUtilRate: BN
  baseRatePerYear: () => BN = () => {
    return this.baseRate
  }
  multiplierPerYearSlope1: () => BN = () => {
    return this.slope1.mul(ONE_ETHER).div(this.optimalUtilRate)
  }
  multiplierPerYearSlope2: () => BN = () => {
    return this.slope2.mul(ONE_ETHER).div(ONE_ETHER.sub(this.optimalUtilRate))
  }
  kink: () => BN = () => {
    return this.optimalUtilRate
  }
}

const model = (base: number, slope1: number, slope2: number, opt: number) => {
  return new StarlayInterestRateModel(
    percent(base),
    percent(slope1),
    percent(slope2),
    percent(opt),
  )
}

export const RATE_MODELS: iAssetBase<InterestRateModel> = {
  weth: model(0, 8, 100, 65),
  bnb: model(0, 7, 300, 45),
  dai: model(0, 4, 60, 90),
  dot: model(0, 7, 100, 65),
  matic: model(0, 7, 300, 45),
  usdc: model(0, 4, 60, 90),
  usdt: model(0, 4, 60, 90),
  wastr: model(0, 7, 300, 45),
}
