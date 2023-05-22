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

  const depositedDai = 3_000_000
  const depositedUsdc = 3_000_000
  const depositedUsdt = 3_000_000
  it('Deployer deposit assets for liquidity', async () => {
    await shouldNotRevert(dai.token, 'mint', [deployer.address, depositedDai])
    await shouldNotRevert(dai.token, 'approve', [
      dai.pool.address,
      depositedDai,
    ])
    await shouldNotRevert(dai.pool, 'mint', [depositedDai])

    await shouldNotRevert(usdc.token, 'mint', [deployer.address, depositedUsdc])
    await shouldNotRevert(usdc.token, 'approve', [
      usdc.pool.address,
      depositedUsdc,
    ])
    await shouldNotRevert(usdc.pool, 'mint', [depositedUsdc])

    await shouldNotRevert(usdt.token, 'mint', [deployer.address, depositedUsdt])
    await shouldNotRevert(usdt.token, 'approve', [
      usdt.pool.address,
      depositedUsdt,
    ])
    await shouldNotRevert(usdt.pool, 'mint', [depositedUsdt])
  })

  it('User 0 try to Flash Loan with mode 0 (return funds)', async () => {
    const initialUserBalance = 100_000
    await shouldNotRevert(dai.token, 'mint', [
      users[0].address,
      initialUserBalance,
    ])
    const premiumTotal = (
      await flashloanGateway.query.flashloanPremiumTotal()
    ).value.ok.toNumber()

    const flashLoanAmount = 200_000
    const premiumAmount = (flashLoanAmount * premiumTotal) / 10000
    await shouldNotRevert(dai.token.withSigner(users[0]), 'approve', [
      flashloanReceiver.address,
      premiumAmount,
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
      premium: premiumAmount,
    })

    const user0Balance = (
      await dai.token.query.balanceOf(users[0].address)
    ).value.ok.toNumber()
    expect(user0Balance).toEqual(initialUserBalance - premiumAmount)

    const poolBalance = (
      await dai.token.query.balanceOf(dai.pool.address)
    ).value.ok.toNumber()
    expect(poolBalance).toEqual(depositedDai + premiumAmount)
  })

  it('User 0 try to Flash Loan with mod 1 (revert expected for insufficient liquidity)', async () => {
    const premiumTotal = (
      await flashloanGateway.query.flashloanPremiumTotal()
    ).value.ok.toNumber()

    const flashLoanAmount = 200_000
    const premiumAmount = (flashLoanAmount * premiumTotal) / 10000
    await shouldNotRevert(dai.token.withSigner(users[0]), 'approve', [
      flashloanReceiver.address,
      premiumAmount,
    ])

    const result = (
      await flashloanGateway
        .withSigner(users[0])
        .query.flashloan(
          flashloanReceiver.address,
          [dai.token.address],
          [flashLoanAmount],
          [1],
          users[0].address,
          [],
        )
    ).value.ok
    expect(result.err).toStrictEqual({
      pool: { controller: 'InsufficientLiquidity' },
    })
  })

  it('Set mock receiver as excution revert. User 0 try to Flash Loan (revert expected)', async () => {
    await flashloanReceiver.tx.setFailExecutionTransfer(true)

    const premiumTotal = (
      await flashloanGateway.query.flashloanPremiumTotal()
    ).value.ok.toNumber()

    const flashLoanAmount = 200_000
    const premiumAmount = (flashLoanAmount * premiumTotal) / 10000
    await shouldNotRevert(dai.token.withSigner(users[0]), 'approve', [
      flashloanReceiver.address,
      premiumAmount,
    ])

    const result = (
      await flashloanGateway
        .withSigner(users[0])
        .query.flashloan(
          flashloanReceiver.address,
          [dai.token.address],
          [flashLoanAmount],
          [0],
          users[0].address,
          [],
        )
    ).value.ok
    expect(result.err).toStrictEqual({
      invalidFlashloanExecutorReturn: null,
    })
  })

  it('Caller deposits 100_000 USDC as collateral, Takes DAI flashloan with mode = 1, does not return the funds. A variable loan for caller is created', async () => {
    const deposited = 100_000
    await shouldNotRevert(usdc.token, 'mint', [users[0].address, deposited])
    await shouldNotRevert(usdc.token.withSigner(users[0]), 'approve', [
      usdc.pool.address,
      deposited,
    ])
    await shouldNotRevert(usdc.pool.withSigner(users[0]), 'mint', [deposited])

    await flashloanReceiver.tx.setFailExecutionTransfer(false)

    const premiumTotal = (
      await flashloanGateway.query.flashloanPremiumTotal()
    ).value.ok.toNumber()

    const flashLoanAmount = 80_000
    const premiumAmount = (flashLoanAmount * premiumTotal) / 10000
    const { events } = await shouldNotRevert(
      flashloanGateway.withSigner(users[0]),
      'flashloan',
      [
        flashloanReceiver.address,
        [dai.token.address],
        [flashLoanAmount],
        [1],
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
      premium: premiumAmount,
    })

    const borrowBalance = (
      await dai.pool.query.borrowBalanceStored(users[0].address)
    ).value.ok.toNumber()

    expect(borrowBalance).toEqual(flashLoanAmount)
  })

  it('tries to take a flashloan that is not listed in the market', async () => {
    const flashLoanAmount = 100_000
    const result = (
      await flashloanGateway.query.flashloan(
        flashloanReceiver.address,
        [dai.pool.address],
        [flashLoanAmount],
        [0],
        users[0].address,
        [],
      )
    ).value.ok

    expect(result.err).toStrictEqual({ marketNotListed: null })
  })
})
