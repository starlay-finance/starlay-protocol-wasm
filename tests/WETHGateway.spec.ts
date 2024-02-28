import type { KeyringPair } from '@polkadot/keyring/types'
import { WeightV2 } from '@polkadot/types/interfaces'
import { BN, BN_ONE, BN_TEN } from '@polkadot/util'
import { ONE_ETHER, ZERO_ADDRESS } from '../scripts/helper/constants'
import {
  deployController,
  deployDefaultInterestRateModel,
  deployIncentivesController,
  deployPriceOracle,
  deployWETH,
  deployWETHGateway,
} from '../scripts/helper/deploy_helper'
import { getGasLimit } from '../scripts/helper/utils'

import WETH from '../types/contracts/weth'
import WETHGateway from '../types/contracts/weth_gateway'

import { Pools, preparePoolsWithPreparedTokens } from './testContractHelper'
import { shouldNotRevert } from './testHelpers'

const MAX_CALL_WEIGHT = new BN(100_000_000_000).isub(BN_ONE).mul(BN_TEN)
const PROOFSIZE = new BN(2_000_000)
describe('WETHGateway spec', () => {
  const rateModelArg = new BN(100).mul(ONE_ETHER)

  let api
  let deployer: KeyringPair
  let pools: Pools
  // let rateModel: DefaultInterestRateModel
  // let controller: Controller
  // let priceOracle: PriceOracle
  // let users: KeyringPair[]
  let weth: WETH
  let wethGateway: WETHGateway
  let gasLimit: WeightV2

  const setup = async () => {
    const { api, alice: deployer, bob, charlie, django } = globalThis.setup
    gasLimit = getGasLimit(api, MAX_CALL_WEIGHT, PROOFSIZE)
    const controller = await deployController({
      api,
      signer: deployer,
      args: [deployer.address],
    })
    const priceOracle = await deployPriceOracle({
      api,
      signer: deployer,
      args: [],
    })

    // temp: declare params for rate_model
    const rateModel = await deployDefaultInterestRateModel({
      api,
      signer: deployer,
      args: [[rateModelArg], [rateModelArg], [rateModelArg], [rateModelArg]],
    })

    // WETH and WETHGateway
    const weth = await deployWETH({
      api,
      signer: deployer,
      args: [],
    })

    const incentivesController = await deployIncentivesController({
      api,
      signer: deployer,
      args: [],
    })

    const pools = await preparePoolsWithPreparedTokens({
      api,
      controller,
      rateModel,
      signer: deployer,
      wethToken: weth,
      incentivesController,
      manager: deployer.address,
    })

    const wethGateway = await deployWETHGateway({
      api,
      signer: deployer,
      args: [weth.address, pools.weth.pool.address],
    })

    const users = [bob, charlie, django]

    // initialize
    await controller.tx.setPriceOracle(priceOracle.address)
    await controller.tx.setCloseFactorMantissa([ONE_ETHER])
    //// for pool
    for (const sym of [pools.weth]) {
      await priceOracle.tx.setFixedPrice(sym.token.address, ONE_ETHER)
      await controller.tx.supportMarketWithCollateralFactorMantissa(
        sym.pool.address,
        sym.token.address,
        [ONE_ETHER.mul(new BN(90)).div(new BN(100))],
      )
    }

    return {
      api,
      deployer,
      pools,
      rateModel,
      controller,
      priceOracle,
      users,
      weth,
      wethGateway,
      incentivesController,
    }
  }

  it('instantiate', async () => {
    ;({
      weth,
      wethGateway,
      api,
      deployer,
      pools,
      // rateModel,
      // controller,
      // priceOracle,
      // users,
    } = await setup())

    expect(weth.address).not.toBe(ZERO_ADDRESS)
    expect(wethGateway.address).not.toBe(ZERO_ADDRESS)

    expect((await wethGateway.query.getWethAddress()).value.ok).toEqual(
      weth.address,
    )

    expect((await weth.query.tokenName()).value.ok).toEqual('Wrapped Astar')
    expect((await weth.query.tokenSymbol()).value.ok).toEqual('WASTR')
    expect((await weth.query.tokenDecimals()).value.ok).toEqual(18)
  })

  const depositAmount = 3000
  it('Deposit WETH', async () => {
    const { pool } = pools.weth
    const {
      data: { free: beforeWethContractBalance },
    } = await api.query.system.account(weth.address)

    await shouldNotRevert(wethGateway, 'depositEth', [
      {
        value: depositAmount,
        gasLimit,
      },
    ])

    expect(
      (await weth.query.balanceOf(pool.address)).value.ok.toString(),
    ).toEqual(depositAmount.toString())

    const {
      data: { free: afterWethContractBalance },
    } = await api.query.system.account(weth.address)
    expect(
      afterWethContractBalance.sub(beforeWethContractBalance).toNumber(),
    ).toEqual(depositAmount)

    expect(
      (await pool.query.balanceOf(deployer.address)).value.ok.toString(),
    ).toEqual(depositAmount.toString())

    expect(
      (
        await pool.query.balanceOfUnderlying(deployer.address)
      ).value.ok.toString(),
    ).toEqual(depositAmount.toString())
  })

  const borrowAmount = 2000
  describe('Borrow WETH', () => {
    it('Should Fail', async () => {
      const result = await wethGateway.query.borrowEth(borrowAmount)

      expect(result.value.ok.err).toStrictEqual({
        pool: { insufficientDelegateAllowance: null },
      })
    })

    it('Success', async () => {
      const { pool } = pools.weth

      await shouldNotRevert(pool, 'approveDelegate', [
        wethGateway.address,
        borrowAmount,
        { gasLimit },
      ])
      const {
        data: { free: beforeWethContractBalance },
      } = await api.query.system.account(weth.address)
      await shouldNotRevert(wethGateway, 'borrowEth', [
        borrowAmount,
        { gasLimit },
      ])
      const {
        data: { free: afterWethContractBalance },
      } = await api.query.system.account(weth.address)

      expect(
        beforeWethContractBalance.sub(afterWethContractBalance).toNumber(),
      ).toEqual(borrowAmount)

      expect(
        (
          await pool.query.borrowBalanceStored(deployer.address)
        ).value.ok.toString(),
      ).toEqual(borrowAmount.toString())
      expect((await pool.query.totalBorrows()).value.ok.toString()).toEqual(
        borrowAmount.toString(),
      )
    })
  })

  const repayAmount = 2000
  it('Repay WETH', async () => {
    const { pool } = pools.weth
    await shouldNotRevert(weth, 'approve', [
      wethGateway.address,
      repayAmount,
      { gasLimit },
    ])
    const {
      data: { free: beforeWethContractBalance },
    } = await api.query.system.account(weth.address)
    await shouldNotRevert(wethGateway, 'repayEth', [
      repayAmount,
      {
        value: repayAmount,
        gasLimit,
      },
    ])
    const {
      data: { free: afterWethContractBalance },
    } = await api.query.system.account(weth.address)

    expect(
      afterWethContractBalance.sub(beforeWethContractBalance).toNumber(),
    ).toEqual(repayAmount)
    expect(
      (await weth.query.balanceOf(pool.address)).value.ok.toString(),
    ).toEqual((depositAmount - borrowAmount + repayAmount).toString())

    // Consider Interest.
    expect((await pool.query.totalBorrows()).value.ok.toNumber()).toEqual(
      borrowAmount - repayAmount,
    )
    expect(
      (
        await pool.query.borrowBalanceStored(deployer.address)
      ).value.ok.toNumber(),
    ).toEqual(borrowAmount - repayAmount)
  })

  const withdrawAmount = 1000
  it('Withdraw WETH', async () => {
    const { pool } = pools.weth

    await shouldNotRevert(pool, 'approve', [
      wethGateway.address,
      withdrawAmount * 3,
      { gasLimit },
    ])

    const {
      data: { free: beforeWethContractBalance },
    } = await api.query.system.account(weth.address)

    await shouldNotRevert(wethGateway, 'withdrawEth', [
      withdrawAmount,
      { gasLimit },
    ])

    expect(
      (await weth.query.balanceOf(pool.address)).value.ok.toNumber(),
    ).toEqual(depositAmount - borrowAmount + repayAmount - withdrawAmount)

    const {
      data: { free: afterWethContractBalance },
    } = await api.query.system.account(weth.address)

    expect(
      beforeWethContractBalance.sub(afterWethContractBalance).toNumber(),
    ).toEqual(withdrawAmount)
  })
})
