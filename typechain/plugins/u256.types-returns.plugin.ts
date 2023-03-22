import { TypechainPlugin } from '@727-ventures/typechain-polkadot/src/types/interfaces'
import { writeFileSync } from '@727-ventures/typechain-polkadot/src/utils/directories'
import { Abi } from '@polkadot/api-contract'
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
