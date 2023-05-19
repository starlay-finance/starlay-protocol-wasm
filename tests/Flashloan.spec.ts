import type { KeyringPair } from '@polkadot/keyring/types'
import { BN } from '@polkadot/util'
import { ONE_ETHER } from '../scripts/helper/constants'
import {
  deployController,
  deployDefaultInterestRateModel,
  deployFlashLoanGateway,
  deployFlashLoanReceiver,
  deployPriceOracle,
} from '../scripts/helper/deploy_helper'
import Controller from '../types/contracts/controller'
import Contract from '../types/contracts/default_interest_rate_model'
import FlashloanGateway from '../types/contracts/flashloan_gateway'
import FlashloanReceiver from '../types/contracts/flashloan_receiver'
import { FlashLoan } from '../types/event-types/flashloan_gateway'
import {
  PoolContracts,
  Pools,
  preparePoolsWithPreparedTokens,
} from './testContractHelper'
import { expectToEmit, shouldNotRevert } from './testHelpers'

describe('Controller spec', () => {
  let deployer: KeyringPair
  let users: KeyringPair[]
  let flashloanGateway: FlashloanGateway
  let flashloanReceiver: FlashloanReceiver
  let controller: Controller
  let pools: Pools
  let usdt: PoolContracts
  let usdc: PoolContracts
  let dai: PoolContracts

  const setup = async (model?: Contract) => {
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

    const rateModel = model
      ? model
      : await deployDefaultInterestRateModel({
          api,
          signer: deployer,
          args: [[0], [0], [0], [0]],
        })

    const pools = await preparePoolsWithPreparedTokens({
      api,
      controller,
      rateModel,
      manager: deployer,
    })

    const users = [bob, charlie, django]

    const flashloanGateway = await deployFlashLoanGateway({
      api,
      signer: deployer,
      args: [controller.address],
    })

    const flashloanReceiver = await deployFlashLoanReceiver({
      api,
      signer: deployer,
      args: [flashloanGateway.address],
    })

    // initialize
    await controller.tx.setPriceOracle(priceOracle.address)
    await controller.tx.setCloseFactorMantissa([ONE_ETHER])
    await controller.tx.setFlashloanGateway(flashloanGateway.address)
    //// for pool
    for (const sym of [pools.dai, pools.usdc, pools.usdt]) {
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
      flashloanGateway,
      flashloanReceiver,
    }
  }

  beforeAll(async () => {
    ;({
      controller,
      flashloanGateway,
      pools,
      flashloanReceiver,
      deployer,
      users,
    } = await setup())
    ;({ usdt, usdc, dai } = pools)
  })

  it('instantiate', async () => {
    expect((await controller.query.flashloanGateway()).value.ok).toEqual(
      flashloanGateway.address,
    )
    expect((await flashloanGateway.query.controller()).value.ok).toEqual(
      controller.address,
    )
  })

  it('Deployer deposit USDT into the pool and User 0 try to Flash Loan with mode 0 (return funds)', async () => {
    // Make Liquidity
    const deposited = 3_000_000
    await shouldNotRevert(dai.token, 'mint', [deployer.address, deposited])
    await shouldNotRevert(dai.token, 'approve', [dai.pool.address, deposited])
    await shouldNotRevert(dai.pool, 'mint', [deposited])

    const initialUserBalance = 100_000
    await shouldNotRevert(dai.token, 'mint', [
      users[0].address,
      initialUserBalance,
    ])
    const premiumTotal = (
      await flashloanGateway.query.flashloanPremiumTotal()
    ).value.ok.toNumber()

    const flashLoanAmount = 200_000
    const approveAmount = (flashLoanAmount * premiumTotal) / 10000
    await shouldNotRevert(dai.token.withSigner(users[0]), 'approve', [
      flashloanReceiver.address,
      approveAmount,
    ])

    const { events } = await shouldNotRevert(
      flashloanGateway.withSigner(users[0]),
      'flashloan',
      [
        flashloanReceiver.address,
        [dai.token.address],
        [flashLoanAmount],
        [0],
        users[0].address,
        [],
      ],
    )

    expect(events).toHaveLength(1)
    expectToEmit<FlashLoan>(events[0], 'FlashLoan', {
      target: flashloanReceiver.address,
      initiator: users[0].address,
      asset: dai.token.address,
      amount: flashLoanAmount,
      premium: approveAmount,
    })

    const user0Balance = (
      await dai.token.query.balanceOf(users[0].address)
    ).value.ok.toNumber()
    expect(user0Balance).toEqual(initialUserBalance - approveAmount)

    const poolBalance = (
      await dai.token.query.balanceOf(dai.pool.address)
    ).value.ok.toNumber()
    expect(poolBalance).toEqual(deposited + approveAmount)
  })
})
