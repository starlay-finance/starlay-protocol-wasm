import { Abi } from '@polkadot/api-contract'
import { TypechainPlugin } from '@starlay-finance/typechain-polkadot/src/types/interfaces'
import { writeFileSync } from '@starlay-finance/typechain-polkadot/src/utils/directories'
import { readFileSync } from 'fs'

const replaceRecursive = (obj: any, name: string, replaced: any) => {
  Object.keys(obj).forEach((key) => {
    if (!obj[key]) return
    if ('name' in obj[key] && obj[key].name === name) {
      obj[key] = replaced
      return
    }
    if ('body' in obj[key] && typeof obj[key].body === 'object')
      replaceRecursive(obj[key].body, name, replaced)
  })
}

export default class WrappedU256DataOverridePlugin implements TypechainPlugin {
  name = 'WrappedU256DataOverride'
  outputDir = 'data'
  ext = 'json'

  generate(
    _abi: Abi,
    fileName: string,
    _absPathToABIs: string,
    absPathToOutput: string,
  ): void {
    const path = `${absPathToOutput}/${this.outputDir}/${fileName}.${this.ext}`
    const json = JSON.parse(readFileSync(path).toString())
    replaceRecursive(json, 'WrappedU256', { name: 'ReturnNumber' })

    writeFileSync(
      absPathToOutput,
      `${this.outputDir}/${fileName}.${this.ext}`,
      JSON.stringify(json),
    )
  }
}
