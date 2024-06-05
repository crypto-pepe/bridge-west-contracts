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
  let executorContractAddress = '';
  let bridgeAdapterContractAddress = '';
  let feeRecipientAddress = '';
  switch (network.name) {
    case 'mainnet':
      multisigContractAddress = '';
      executorContractAddress = '';
      bridgeAdapterContractAddress = '';
      feeRecipientAddress = '';
      throw 'todo'; // TODO: set
      break;
    case 'testnet':
      multisigContractAddress = '9Zr9mUTwe62HBCpe7XTpJsJteLT4CpMrrQMaG4ry1VPE';
      executorContractAddress = '5uzd6TigKZVggtHWmqzGHyif2Cqojxnx1aq9YhhMZ4hP';
      bridgeAdapterContractAddress =
        '5JJ9nFhnPdrwRm5d6ZT46BqwntXEgkjgXkrtrRo94kQk';
      feeRecipientAddress = '3N1VhCMKNh2SBgw9mhdLRBjyucnv6fkMNBA';
      break;
  }

  const tx = await deployWasmScript(
    'coin_bridge',
    resolve(process.cwd(), './bin/coin_bridge.wasm'),
    [
      { type: 'string', key: 'multisig', value: multisigContractAddress },
      { type: 'string', key: 'executor', value: executorContractAddress },
      { type: 'string', key: 'adapter', value: bridgeAdapterContractAddress },
      { type: 'string', key: 'pauser', value: deployerAddress },
      { type: 'string', key: 'fee_recipient', value: feeRecipientAddress },
    ],
    [],
    deployerPrivateKey,
    network,
    proofsGenerator
  ).catch((e) => {
    throw e;
  });

  console.log('Coin Bridge contract deployed at contractId = ' + tx.tx.id);

  return true;
}
