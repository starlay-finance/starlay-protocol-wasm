import { TypechainPlugin } from '@727-ventures/typechain-polkadot/src/types/interfaces'
import { writeFileSync } from '@727-ventures/typechain-polkadot/src/utils/directories'
import { Abi } from '@polkadot/api-contract'
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
    const maybeJson = readFileSync(path).toString()
    const shouldBeJson =
      maybeJson.lastIndexOf(',') == maybeJson.length - 3
        ? maybeJson.slice(0, maybeJson.length - 3).concat('}')
        : maybeJson
    const json = JSON.parse(shouldBeJson)

    writeFileSync(
      absPathToOutput,
      `${this.outputDir}/${fileName}.${this.ext}`,
      JSON.stringify(json),
    )
  }
}
