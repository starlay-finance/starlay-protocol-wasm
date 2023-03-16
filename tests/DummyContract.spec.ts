import DummyContract_factory from '../types/constructors/dummy_contract'
import DummyContract from '../types/contracts/dummy_contract'

describe('DummyContract spec', () => {
  const setup = async () => {
    const { api, alice: deployer } = globalThis.setup

    const factory = new DummyContract_factory(api, deployer)
    const contract = await factory.new()
    return new DummyContract(contract.address, deployer, api)
  }

  it('.result_ok', async () => {
    const contract = await setup()
    const res = await contract.query.resultOk()
    console.log(res.value)
  })
  it('.result_ng', async () => {
    const contract = await setup()
    const res = await contract.query.resultNg()
    console.log(res.value)
  })
  it('.result_panic', async () => {
    const contract = await setup()
    const res = await contract.query.resultPanic()
    console.log(res.value)
  })
})
