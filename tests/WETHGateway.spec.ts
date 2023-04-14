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
    const { weth, wethGateway } = await setup()
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
  })

  it('Deposit WETH', async () => {
    const { weth, wethGateway, pools, users, api } = await setup()
    const { pool } = pools.weth

    const {
      data: { free: beforeBalanceUser0 },
    } = await api.query.system.account(users[0].address)

    await wethGateway
      .withSigner(users[0])
      .tx.depositEth(pool.address, users[0].address, {
        value: ONE_ETHER,
      })

    const {
      data: { free: afterBalanceUser0 },
    } = await api.query.system.account(users[0].address)

    expect(
      (await weth.query.balanceOf(pool.address)).value.ok.toString(),
    ).toEqual(ONE_ETHER.toString())

    expect(beforeBalanceUser0.sub(afterBalanceUser0).gt(ONE_ETHER)).toEqual(
      true,
    )
    expect(
      beforeBalanceUser0.sub(afterBalanceUser0).lt(new BN(2).mul(ONE_ETHER)),
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

    await wethGateway
      .withSigner(users[0])
      .tx.depositEth(pool.address, users[0].address, {
        value: ONE_ETHER,
      })

    const withdrawAmount = ONE_ETHER.div(new BN(2))
    await pool
      .withSigner(users[0])
      .tx.approve(wethGateway.address, withdrawAmount)
    await wethGateway
      .withSigner(users[0])
      .tx.withdrawEth(pool.address, withdrawAmount, users[0].address)
    expect(
      (await weth.query.balanceOf(pool.address)).value.ok.toString(),
    ).toEqual(withdrawAmount.toString())
  })
})
