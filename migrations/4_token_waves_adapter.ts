import { resolve } from 'path';
import { NetworkConfig } from '../scripts/network';
import { ProofsGenerator, deployWasmScript } from '../scripts/script';
import { Keypair } from '@wavesenterprise/signer';
import { getAddressFromPrivateKey } from '../scripts/helpers';

export default async function (
  deployerSeed: string,
  network: NetworkConfig,
  proofsGenerator: ProofsGenerator
) {
  const deployerPrivateKey = await (
    await Keypair.fromExistingSeedPhrase(deployerSeed)
  ).privateKey();
  const deployerAddress = await getAddressFromPrivateKey(
    deployerPrivateKey,
    network.chainID
  );

  let multisigContractAddress = '';
  let bridgeAdapterAddress = '';
  let protocolCallerAddress = '';
  switch (network.name) {
    case 'mainnet':
      multisigContractAddress = '';
      bridgeAdapterAddress = '';
      protocolCallerAddress = '';
      throw 'todo'; // TODO: set
      break;
    case 'testnet':
      multisigContractAddress = '73YBejz5FnV1csCwErwC6CS5tt7VY1cpS8eRfgWfJd3F';
      bridgeAdapterAddress = '6eF3nNZhKc8ahA6NDpCXj7tVA6naXnoDuyiHV12KJmip';
      protocolCallerAddress = '59aJQqPYsH5nkVnXrqR7DhHDV91kmVhdvQXofQn72wmX';
      break;
  }

  const tx = await deployWasmScript(
    'token_waves_adapter',
    resolve(process.cwd(), './bin/token_waves_adapter.wasm'),
    [
      { type: 'string', key: 'multisig', value: multisigContractAddress },
      { type: 'string', key: 'protocol_caller', value: protocolCallerAddress },
      { type: 'string', key: 'root_adapter', value: bridgeAdapterAddress },
      { type: 'string', key: 'pauser', value: deployerAddress },
    ],
    [],
    deployerPrivateKey,
    network,
    proofsGenerator
  ).catch((e) => {
    throw e;
  });

  console.log(
    'Token Waves adapter contract deployed at contractId = ' + tx.tx.id
  );

  return true;
}
