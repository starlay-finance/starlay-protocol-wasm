import type { KeyringPair } from '@polkadot/keyring/types'
import { deployController, deployPSP22Token } from './testContractsHelper'
import { hexToUtf8, zeroAddress } from './testHelpers'

import Pool_factory from '../types/constructors/pool'
import Pool from '../types/contracts/pool'

import Controller from '../types/contracts/controller'
import PSP22Token from '../types/contracts/psp22_token'

describe('Pool spec', () => {
  const setup = async () => {
    const { api, alice: deployer } = globalThis.setup

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

    return { deployer, token, pool, controller }
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
    let controller: Controller

    beforeAll(async () => {
      ;({ deployer, token, pool, controller } = await setup())
    })

    it('preparations', async () => {
      await token.tx.mint(deployer.address, 10_000)
      const { value } = await token.query.balanceOf(deployer.address)
      expect(value.ok.toNumber()).toEqual(10_000)
    })

    it('execute', async () => {
      await token.tx.approve(pool.address, 3_000)
      await pool.tx.mint(3_000)

      expect(
        (await token.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(7000)
      expect(
        (await token.query.balanceOf(pool.address)).value.ok.toNumber(),
      ).toEqual(3000)
      expect(
        (await pool.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(3000)
    })
  })

  describe('.redeem', () => {
    let deployer: KeyringPair
    let token: PSP22Token
    let pool: Pool
    let controller: Controller

    beforeAll(async () => {
      ;({ deployer, token, pool, controller } = await setup())
    })

    it('preparations', async () => {
      await token.tx.mint(deployer.address, 10_000)
      const { value } = await token.query.balanceOf(deployer.address)
      expect(value.ok.toNumber()).toEqual(10_000)

      await token.tx.approve(pool.address, 10_000)
      await pool.tx.mint(10_000)
      expect(
        (await pool.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(10_000)
    })

    it('execute', async () => {
      await pool.tx.redeem(3_000)

      expect(
        (await token.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(3000)
      expect(
        (await token.query.balanceOf(pool.address)).value.ok.toNumber(),
      ).toEqual(7000)
      expect(
        (await pool.query.balanceOf(deployer.address)).value.ok.toNumber(),
      ).toEqual(7000)
    })
  })
})
