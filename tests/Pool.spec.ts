import type { KeyringPair } from '@polkadot/keyring/types'
import { deployController, deployPSP22Token } from './testContractsHelper'
import { hexToUtf8, zeroAddress } from './testHelpers'

import Pool_factory from '../types/constructors/pool'
import Pool from '../types/contracts/pool'

import PSP22Token from '../types/contracts/psp22_token'

describe('Pool spec', () => {
  const setup = async () => {
    const { api, alice: deployer, bob, charlie } = globalThis.setup

    const token = await deployPSP22Token({
      api,
      signer: deployer,
      args: [
        0,
        'Dai Stablecoin' as unknown as string[],
        'DAI' as unknown as string[],
        8,
      ],
    })

    const controller = await deployController({
      api,
      signer: deployer,
      args: [],
    })

    const poolFactory = new Pool_factory(api, deployer)
    const pool = new Pool(
      (
        await poolFactory.newFromAsset(token.address, controller.address)
      ).address,
      deployer,
      api,
    )
    const users = [bob, charlie]

    // initialize
    await controller.tx.supportMarket(pool.address)

    return { deployer, token, pool, controller, users }
  }

  it('instantiate', async () => {
    const { token, pool, controller } = await setup()
    expect(pool.address).not.toBe(zeroAddress)
    expect((await pool.query.underlying()).value.ok).toEqual(token.address)
    expect((await pool.query.controller()).value.ok).toEqual(controller.address)
    expect(hexToUtf8((await pool.query.tokenName()).value.ok)).toEqual(
      'Starlay Dai Stablecoin',
    )
    expect(hexToUtf8((await pool.query.tokenSymbol()).value.ok)).toEqual('sDAI')
    expect((await pool.query.tokenDecimals()).value.ok).toEqual(8)
  })

  describe('.mint', () => {
    let deployer: KeyringPair
    let token: PSP22Token
    let pool: Pool

    beforeAll(async () => {
      ;({ deployer, token, pool } = await setup())
    })

    it('preparations', async () => {
      await token.tx.mint(deployer.address, 10_000)
      expect(
        (await token.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(10_000)
    })

    it('execute', async () => {
      await token.tx.approve(pool.address, 3_000)
      const { events } = await pool.tx.mint(3_000)

      expect(
        (await token.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(7000)
      expect(
        (await token.query.balanceOf(pool.address)).value.ok.toNumber(),
      ).toEqual(3000)
      expect(
        (await pool.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(3000)

      const event = events[0]
      expect(event.name).toEqual('Mint')
      expect(event.args.minter).toEqual(deployer.address)
      expect(event.args.mintAmount.toNumber()).toEqual(3_000)
      expect(event.args.mintTokens.toNumber()).toEqual(3_000)
    })
  })

  describe('.redeem', () => {
    let deployer: KeyringPair
    let token: PSP22Token
    let pool: Pool

    beforeAll(async () => {
      ;({ deployer, token, pool } = await setup())
    })

    it('preparations', async () => {
      await token.tx.mint(deployer.address, 10_000)

      await token.tx.approve(pool.address, 10_000)
      await pool.tx.mint(10_000)
      expect(
        (await pool.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(10_000)
    })

    it('execute', async () => {
      const { events } = await pool.tx.redeem(3_000)

      expect(
        (await token.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(3000)
      expect(
        (await token.query.balanceOf(pool.address)).value.ok.toNumber(),
      ).toEqual(7000)
      expect(
        (await pool.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(7000)

      const event = events[0]
      expect(event.name).toEqual('Redeem')
      expect(event.args.redeemer).toEqual(deployer.address)
      expect(event.args.redeemAmount.toNumber()).toEqual(3_000)
      expect(event.args.redeemTokens.toNumber()).toEqual(3_000)
    })
  })

  describe('.borrow', () => {
    let deployer: KeyringPair
    let token: PSP22Token
    let pool: Pool
    let users: KeyringPair[]

    beforeAll(async () => {
      ;({ deployer, token, pool, users } = await setup())
    })

    it('preparations', async () => {
      await token.tx.mint(deployer.address, 10_000)
      await token.tx.approve(pool.address, 10_000)
      await pool.tx.mint(10_000)
      expect(
        (await pool.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(10_000)
    })

    it('execute', async () => {
      const [user1, user2] = users
      const { events: events1 } = await pool.withSigner(user1).tx.borrow(3_000)

      expect(
        (await token.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(0)
      expect(
        (await token.query.balanceOf(user1.address)).value.ok.toNumber(),
      ).toEqual(3_000)
      expect(
        (await token.query.balanceOf(user2.address)).value.ok.toNumber(),
      ).toEqual(0)
      expect(
        (await token.query.balanceOf(pool.address)).value.ok.toNumber(),
      ).toEqual(7_000)
      expect(
        (await pool.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(10_000)
      const event1 = events1[0]
      expect(event1.name).toEqual('Borrow')
      expect(event1.args.borrower).toEqual(user1.address)
      expect(event1.args.borrowAmount.toNumber()).toEqual(3_000)
      expect(event1.args.accountBorrows.toNumber()).toEqual(3_000)
      expect(event1.args.totalBorrows.toNumber()).toEqual(3_000)

      const { events: events2 } = await pool.withSigner(user2).tx.borrow(2_500)

      expect(
        (await token.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(0)
      expect(
        (await token.query.balanceOf(user1.address)).value.ok.toNumber(),
      ).toEqual(3_000)
      expect(
        (await token.query.balanceOf(user2.address)).value.ok.toNumber(),
      ).toEqual(2_500)
      expect(
        (await token.query.balanceOf(pool.address)).value.ok.toNumber(),
      ).toEqual(4_500)
      expect(
        (await pool.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(10_000)
      const event2 = events2[0]
      expect(event2.name).toEqual('Borrow')
      expect(event2.args.borrower).toEqual(user2.address)
      expect(event2.args.borrowAmount.toNumber()).toEqual(2_500)
      expect(event2.args.accountBorrows.toNumber()).toEqual(2_500)
      expect(event2.args.totalBorrows.toNumber()).toEqual(5_500)
    })
  })

  describe('.repay_borrow', () => {
    let deployer: KeyringPair
    let token: PSP22Token
    let pool: Pool
    let users: KeyringPair[]

    beforeAll(async () => {
      ;({ deployer, token, pool, users } = await setup())
    })

    it('preparations', async () => {
      await token.tx.mint(deployer.address, 10_000)
      await token.tx.approve(pool.address, 10_000)
      await pool.tx.mint(10_000)
      expect(
        (await pool.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(10_000)

      const [user1, _] = users
      await pool.withSigner(user1).tx.borrow(10_000)
      expect(
        (await token.query.balanceOf(user1.address)).value.ok.toNumber(),
      ).toEqual(10_000)
    })

    it('execute', async () => {
      const [user1, _] = users
      await token.withSigner(user1).tx.approve(pool.address, 4_500)
      const { events } = await pool.withSigner(user1).tx.repayBorrow(4_500)

      expect(
        (await token.query.balanceOf(user1.address)).value.ok.toNumber(),
      ).toEqual(5_500)
      expect(
        (await token.query.balanceOf(pool.address)).value.ok.toNumber(),
      ).toEqual(4_500)

      const event = events[0]
      expect(event.name).toEqual('RepayBorrow')
      expect(event.args.payer).toEqual(user1.address)
      expect(event.args.borrower).toEqual(user1.address)
      expect(event.args.repayAmount.toNumber()).toEqual(4_500)
      expect(event.args.accountBorrows.toNumber()).toEqual(5_500)
      expect(event.args.totalBorrows.toNumber()).toEqual(5_500)
    })
  })
})
