import { BN } from '@polkadot/util'
import { ONE_ETHER, ZERO_ADDRESS } from '../scripts/helper/constants'
import {
  deployController,
  deployDefaultInterestRateModel,
  deployPriceOracle,
  deployWETH,
  deployWETHGateway,
} from '../scripts/helper/deploy_helper'
import { hexToUtf8 } from '../scripts/helper/utils'
import { preparePoolsWithPreparedTokens } from './testContractHelper'

describe('WETHGateway spec', () => {
  const rateModelArg = new BN(100).mul(ONE_ETHER)

  const setup = async () => {
    const { api, alice: deployer, bob, charlie, django } = globalThis.setup
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

    const wethGateway = await deployWETHGateway({
      api,
      signer: deployer,
      args: [weth.address],
    })

    const pools = await preparePoolsWithPreparedTokens({
      api,
      controller,
      rateModel,
      manager: deployer,
      wethToken: weth,
      gateway: wethGateway.address,
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
    }
  }

  it('instantiate', async () => {
    const { weth, wethGateway, pools } = await setup()
    const { pool } = pools.weth
    expect(weth.address).not.toBe(ZERO_ADDRESS)
    expect(wethGateway.address).not.toBe(ZERO_ADDRESS)

    expect((await wethGateway.query.getWethAddress()).value.ok).toEqual(
      weth.address,
    )

    expect(hexToUtf8((await weth.query.tokenName()).value.ok)).toEqual(
      'Wrapped Astar',
    )
    expect(hexToUtf8((await weth.query.tokenSymbol()).value.ok)).toEqual(
      'WASTR',
    )
    expect((await weth.query.tokenDecimals()).value.ok).toEqual(18)
    expect((await pool.query.gateway()).value.ok).toEqual(wethGateway.address)
  })

  it('Deposit WETH', async () => {
    const { weth, wethGateway, pools, users, api } = await setup()
    const { pool } = pools.weth

    const {
      data: { free: beforeUserBalance },
    } = await api.query.system.account(users[0].address)

    await wethGateway.withSigner(users[0]).tx.depositEth(pool.address, {
      value: ONE_ETHER,
    })

    const {
      data: { free: afterUserBalance },
    } = await api.query.system.account(users[0].address)

    expect(
      (await weth.query.balanceOf(pool.address)).value.ok.toString(),
    ).toEqual(ONE_ETHER.toString())

    expect(beforeUserBalance.sub(afterUserBalance).gt(ONE_ETHER)).toEqual(true)
    expect(
      beforeUserBalance.sub(afterUserBalance).lt(new BN(2).mul(ONE_ETHER)),
    ).toEqual(true)

    expect(
      (await pool.query.balanceOf(users[0].address)).value.ok.toString(),
    ).toEqual(ONE_ETHER.toString())

    expect(
      (
        await pool.query.principalBalanceOf(users[0].address)
      ).value.ok.toString(),
    ).toEqual(ONE_ETHER.toString())
  })

  it('Withdraw WETH', async () => {
    const { weth, wethGateway, pools, users, api } = await setup()
    const { pool } = pools.weth

    const depositAmount = ONE_ETHER
    await wethGateway.withSigner(users[0]).tx.depositEth(pool.address, {
      value: depositAmount,
    })

    const withdrawAmount = ONE_ETHER.div(new BN(5))
    await pool
      .withSigner(users[0])
      .tx.approve(wethGateway.address, withdrawAmount)

    const {
      data: { free: beforeUserBalance },
    } = await api.query.system.account(users[0].address)
    await wethGateway
      .withSigner(users[0])
      .tx.withdrawEth(pool.address, withdrawAmount)
    expect(
      (await weth.query.balanceOf(pool.address)).value.ok.toString(),
    ).toEqual(depositAmount.sub(withdrawAmount).toString())

    const {
      data: { free: afterUserBalance },
    } = await api.query.system.account(users[0].address)

    expect(afterUserBalance.sub(beforeUserBalance).gt(new BN(0))).toEqual(true)
    expect(afterUserBalance.sub(beforeUserBalance).lt(withdrawAmount)).toEqual(
      true,
    )
  })

  it('Borrow WETH', async () => {
    const { wethGateway, pools, users, api } = await setup()
    const { pool } = pools.weth

    const depositAmount = ONE_ETHER.mul(new BN(2))
    await wethGateway.withSigner(users[0]).tx.depositEth(pool.address, {
      value: depositAmount,
    })
    await wethGateway.withSigner(users[1]).tx.depositEth(pool.address, {
      value: depositAmount,
    })

    const {
      data: { free: beforeUserBalance },
    } = await api.query.system.account(users[0].address)
    const borrowAmount = ONE_ETHER.div(new BN(5))
    await wethGateway
      .withSigner(users[0])
      .tx.borrowEth(pool.address, borrowAmount)
    const {
      data: { free: afterUserBalance },
    } = await api.query.system.account(users[0].address)

    expect(afterUserBalance.sub(beforeUserBalance).gt(new BN(0))).toEqual(true)

    expect(
      (
        await pool.query.borrowBalanceStored(users[0].address)
      ).value.ok.toString(),
    ).toEqual(borrowAmount.toString())
    expect((await pool.query.totalBorrows()).value.ok.toString()).toEqual(
      borrowAmount.toString(),
    )
  })

  it('Repay WETH', async () => {
    const { weth, wethGateway, pools, users, api } = await setup()
    const { pool } = pools.weth

    const depositAmount = ONE_ETHER.mul(new BN(2))
    await wethGateway.withSigner(users[0]).tx.depositEth(pool.address, {
      value: depositAmount,
    })
    await wethGateway.withSigner(users[1]).tx.depositEth(pool.address, {
      value: depositAmount,
    })

    const borrowAmount = ONE_ETHER.div(new BN(2))
    await wethGateway
      .withSigner(users[0])
      .tx.borrowEth(pool.address, borrowAmount)

    expect(
      (
        await pool.query.borrowBalanceStored(users[0].address)
      ).value.ok.toString(),
    ).toEqual(borrowAmount.toString())

    const repayAmount = ONE_ETHER.div(new BN(5))
    await weth.withSigner(users[0]).tx.approve(wethGateway.address, repayAmount)
    const {
      data: { free: beforeUserBalance },
    } = await api.query.system.account(users[0].address)
    await wethGateway
      .withSigner(users[0])
      .tx.repayEth(pool.address, repayAmount, {
        value: repayAmount,
      })
    const {
      data: { free: afterUserBalance },
    } = await api.query.system.account(users[0].address)

    expect(beforeUserBalance.sub(afterUserBalance).gt(repayAmount)).toEqual(
      true,
    )
    expect(
      (await weth.query.balanceOf(pool.address)).value.ok.toString(),
    ).toEqual(
      depositAmount
        .mul(new BN(2))
        .sub(borrowAmount)
        .add(repayAmount)
        .toString(),
    )

    // Consider Interest.
    expect(
      new BN((await pool.query.totalBorrows()).value.ok.toString()).gt(
        borrowAmount.sub(repayAmount),
      ),
    ).toEqual(true)
    expect(
      new BN(
        (
          await pool.query.borrowBalanceStored(users[0].address)
        ).value.ok.toString(),
      ).gt(borrowAmount.sub(repayAmount)),
    ).toEqual(true)
  })
})
