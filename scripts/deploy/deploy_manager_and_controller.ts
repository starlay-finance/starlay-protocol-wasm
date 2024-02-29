import { ApiPromise } from '@polkadot/api'
import type { KeyringPair } from '@polkadot/keyring/types'
import Controller from '../../types/contracts/controller'
import Manager from '../../types/contracts/manager'
import PriceOracle from '../../types/contracts/price_oracle'
import { Config } from '../config'
import { ROLE, ZERO_ADDRESS } from '../helper/constants'
import { defaultOption, sendTxWithPreview } from '../helper/utils'
import { deployController, deployManager } from './../helper/deploy_helper'

type DeployManagerAndController = (args: {
  api: ApiPromise
  signer: KeyringPair
  priceOracle: PriceOracle
  option: ReturnType<typeof defaultOption>
  config: Config
}) => Promise<{
  manager: Manager
  controller: Controller
}>
export const deployManagerAndController: DeployManagerAndController = async ({
  api,
  signer,
  priceOracle,
  config: { roleGrantees, closeFactor, liquidationIncentive },
  option,
}) => {
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

  await sendTxWithPreview(
    manager,
    'setController',
    [controller.address],
    option,
  )

  for (const key of Object.keys(ROLE)) {
    const role = ROLE[key]
    if (role === ROLE.DEFAULT_ADMIN_ROLE) continue
    const grantee = (roleGrantees && roleGrantees[role]) || signer.address
    await sendTxWithPreview(manager, 'grantRole', [role, grantee], option)
  }

  await sendTxWithPreview(
    manager,
    'setPriceOracle',
    [priceOracle.address],
    option,
  )
  await sendTxWithPreview(
    manager,
    'setCloseFactorMantissa',
    [[closeFactor]],
    option,
  )
  await sendTxWithPreview(
    manager,
    'setLiquidationIncentiveMantissa',
    [[liquidationIncentive]],
    option,
  )
  return { manager, controller }
}
