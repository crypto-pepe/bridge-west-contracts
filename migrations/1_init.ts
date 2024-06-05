import { resolve } from 'path';
import { NetworkConfig } from '../scripts/network';
import { ProofsGenerator, deployWasmScript } from '../scripts/script';
import { Keypair } from '@wavesenterprise/signer';

export default async function (
  deployerSeed: string,
  network: NetworkConfig,
  proofsGenerator: ProofsGenerator
) {
  const deployerKeyPair = await Keypair.fromExistingSeedPhrase(deployerSeed);
  const deployerPrivateKey = await deployerKeyPair.privateKey();

  const tx = await deployWasmScript(
    'migrations',
    resolve(process.cwd(), './bin/migration.wasm'),
    [],
    [],
    deployerPrivateKey,
    network,
    proofsGenerator
  ).catch((e) => {
    throw e;
  });

  console.log('Migration contract deployed at contractId = ' + tx.tx.id);

  return true;
}
