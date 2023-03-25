import { ApiPromise } from '@polkadot/api'
import type { KeyringPair } from '@polkadot/keyring/types'
import { BN } from '@polkadot/util'
import PSP22Token from '../../types/contracts/psp22_token'
import { deployer, provider } from '../helper/wallet_helper'
import { DUMMY_TOKENS, ONE_ETHER, Token } from '../tokens'
import { ENV, Env } from './../env'
import {
  ROLE,
  ZERO_ADDRESS,
  defaultOption,
  deployController,
  deployDefaultInterestRateModel,
  deployFaucet,
  deployLens,
  deployManager,
  deployPSP22Token,
  deployPool,
  deployPriceOracle,
  waitForTx,
} from './../helper/deploy_helper'

const main = async () => {
  await deployContracts(ENV.testnet)
}
const deployContracts = async (env: Env) => {
  const api = await provider(env)
  const signer = await deployer(env)
  const option = defaultOption(api)
  const manager = await deployManager({
    api,
    signer,
    args: [ZERO_ADDRESS],
  })
  const controller = await deployController({
    api,
    signer,
    args: [manager.address],
  })
  const priceOracle = await deployPriceOracle({
    api,
    signer,
    args: [],
  })

  await waitForTx(await manager.tx.setController(controller.address, option))
  for (const key of Object.keys(ROLE)) {
    const role = ROLE[key]
    if (role === ROLE.DEFAULT_ADMIN_ROLE) continue
    await waitForTx(await manager.tx.grantRole(role, signer.address, option))
    console.log(`Role ${key} has been granted to ${signer.address}`)
  }

  await waitForTx(await manager.tx.setPriceOracle(priceOracle.address, option))
  for (const token of await deployDummyTokens(api, signer)) {
    const {
      baseRatePerYear,
      multiplierPerYearSlope1,
      multiplierPerYearSlope2,
      kink,
    } = token.token.rateModel
    const toParam = (m: BN) => [m.toString()]

    const rateModelContract = await deployDefaultInterestRateModel({
      api,
      signer,
      args: [
        toParam(baseRatePerYear),
        toParam(multiplierPerYearSlope1),
        toParam(multiplierPerYearSlope2),
        toParam(kink),
      ],
    })
    const pool = await deployPool({
      api,
      signer,
      args: [
        token.contract.address,
        controller.address,
        rateModelContract.address,
        [resolvePoolName(token.token.name)],
        [token.token.symbol],
        token.token.decimal,
      ],
    })
    await waitForTx(
      await priceOracle.tx.setFixedPrice(
        token.contract.address,
        ONE_ETHER,
        option,
      ),
    )
    await waitForTx(await manager.tx.supportMarket(pool.address, option))
  }
  await deployLens({ api, signer, args: [] })
  await deployFaucet({ api, signer, args: [] })
}

const resolvePoolName = (token: string) => {
  return `${token}Pool`
}
const deployDummyTokens = async (api: ApiPromise, signer: KeyringPair) => {
  const res: { contract: PSP22Token; token: Token }[] = []
  for (const token of DUMMY_TOKENS) {
    const deployed = await deployPSP22Token({
      api,
      signer,
      args: [
        token.totalSupply,
        token.name as unknown as string[],
        token.symbol as unknown as string[],
        token.decimal,
      ],
    })
    res.push({ contract: deployed, token: token })
  }
  return res
}

main()
  .then(() => {
    console.log('finish')
    process.exit(0)
  })
  .catch((e) => {
    console.log(e)
    process.exit(1)
  })
