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
  switch (network.name) {
    case 'mainnet':
      multisigContractAddress = '';
      throw 'todo'; // TODO: set
      break;
    case 'testnet':
      multisigContractAddress = '73YBejz5FnV1csCwErwC6CS5tt7VY1cpS8eRfgWfJd3F';
      break;
  }

  const tx = await deployWasmScript(
    'bridge_adapter',
    resolve(process.cwd(), './bin/bridge_adapter.wasm'),
    [
      { type: 'string', key: 'multisig', value: multisigContractAddress },
      { type: 'string', key: 'pauser', value: deployerAddress },
    ],
    [],
    deployerPrivateKey,
    network,
    proofsGenerator
  ).catch((e) => {
    throw e;
  });

  console.log('Bridge adapter contract deployed at contractId = ' + tx.tx.id);

  return true;
}
