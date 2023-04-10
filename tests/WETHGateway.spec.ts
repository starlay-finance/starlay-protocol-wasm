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
    const rateModelArg = new BN(100).mul(ONE_ETHER)
    const rateModel = await deployDefaultInterestRateModel({
      api,
      signer: deployer,
      args: [[rateModelArg], [rateModelArg], [rateModelArg], [rateModelArg]],
    })

    const pools = await preparePoolsWithPreparedTokens({
      api,
      controller,
      rateModel,
      manager: deployer,
    })

    const users = [bob, charlie, django]

    // initialize
    await controller.tx.setPriceOracle(priceOracle.address)
    await controller.tx.setCloseFactorMantissa([ONE_ETHER])
    //// for pool
    for (const sym of [pools.dai, pools.usdc, pools.usdt]) {
      await priceOracle.tx.setFixedPrice(sym.token.address, ONE_ETHER)
      await controller.tx.supportMarketWithCollateralFactorMantissa(
        sym.pool.address,
        [ONE_ETHER.mul(new BN(90)).div(new BN(100))],
      )
    }

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
      'Wrapped Ether',
    )
    expect(hexToUtf8((await weth.query.tokenSymbol()).value.ok)).toEqual('WETH')
    expect((await weth.query.tokenDecimals()).value.ok).toEqual(18)
  })

  it('Deposit WETH', async () => {
    const { weth, wethGateway, pools } = await setup()
    const { alice, bob, api } = globalThis.setup
    const { pool } = pools.dai

    const beforeBalanceAlice = api.query.system.account(alice)
    console.log(beforeBalanceAlice)
    await wethGateway.query.depositEth(pool.address, alice, {
      value: ONE_ETHER,
    })
    const TWO_ETHER = new BN(2).mul(ONE_ETHER)
    await wethGateway.query.depositEth(pool.address, bob, {
      value: TWO_ETHER,
    })

    expect((await weth.query.balanceOf(alice)).value.ok).toEqual(0)
    expect((await pool.query.balanceOf(alice)).value.ok).toEqual(ONE_ETHER)
    expect((await weth.query.balanceOf(bob)).value.ok).toEqual(0)
    expect((await pool.query.balanceOf(bob)).value.ok).toEqual(TWO_ETHER)
  })

  it('Withdraw WETH', async () => {
    const { weth, wethGateway, pools } = await setup()
    const { alice, bob } = globalThis.setup
    const { pool } = pools.dai

    await wethGateway.query.depositEth(pool.address, alice, {
      value: ONE_ETHER,
    })
    const TWO_ETHER = new BN(2).mul(ONE_ETHER)
    await wethGateway.query.depositEth(pool.address, bob, {
      value: TWO_ETHER,
    })

    expect((await weth.query.balanceOf(alice)).value.ok).toEqual(0)
    expect((await pool.query.balanceOf(alice)).value.ok).toEqual(ONE_ETHER)
    expect((await weth.query.balanceOf(bob)).value.ok).toEqual(0)
    expect((await pool.query.balanceOf(bob)).value.ok).toEqual(TWO_ETHER)

    const HALF_ETHER = new BN(0.5).mul(ONE_ETHER)
    await wethGateway.query.withdrawEth(pool.address, HALF_ETHER, alice)
    await wethGateway.query.withdrawEth(pool.address, ONE_ETHER, bob)
    expect((await weth.query.balanceOf(alice)).value.ok).toEqual(HALF_ETHER)
    expect((await pool.query.balanceOf(alice)).value.ok).toEqual(HALF_ETHER)
    expect((await weth.query.balanceOf(bob)).value.ok).toEqual(ONE_ETHER)
    expect((await pool.query.balanceOf(bob)).value.ok).toEqual(ONE_ETHER)
  })
})
