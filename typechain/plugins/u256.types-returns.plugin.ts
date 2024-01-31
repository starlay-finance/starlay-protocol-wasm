import { Abi } from '@polkadot/api-contract'
import { TypechainPlugin } from '@starlay-finance/typechain-polkadot/src/types/interfaces'
import { writeFileSync } from '@starlay-finance/typechain-polkadot/src/utils/directories'
import { readFileSync } from 'fs'

export default class U256TypesReturnsOverridePlugin implements TypechainPlugin {
  name = 'U256TypesReturnsOverride'
  outputDir = 'types-returns'
  ext = 'ts'

  generate(
    _abi: Abi,
    fileName: string,
    _absPathToABIs: string,
    absPathToOutput: string,
  ): void {
    const path = `${absPathToOutput}/${this.outputDir}/${fileName}.${this.ext}`
    const file = readFileSync(path)
    writeFileSync(
      absPathToOutput,
      `${this.outputDir}/${fileName}.${this.ext}`,
      file
        .toString()
        .replace(
          'export type U256 = Array<number>;',
          'export type U256 = ReturnNumber;',
        ),
    )
  }
}
