import { NetworkConfig } from '../scripts/network';
import { ProofsGenerator } from '../scripts/script';
import { Keypair } from '@wavesenterprise/signer';
import { invoke } from '../scripts/transaction';

export default async function (
  deployerSeed: string,
  network: NetworkConfig,
  proofsGenerator: ProofsGenerator
) {
  const deployerPrivateKey = await (
    await Keypair.fromExistingSeedPhrase(deployerSeed)
  ).privateKey();

  let bridgeAdapterContractAddress = '';
  let coinBridgeContractAddress = '';
  switch (network.name) {
    case 'mainnet':
      bridgeAdapterContractAddress = '';
      coinBridgeContractAddress = '';
      throw 'todo'; // TODO: set
      break;
    case 'testnet':
      bridgeAdapterContractAddress =
        '6eF3nNZhKc8ahA6NDpCXj7tVA6naXnoDuyiHV12KJmip';
      coinBridgeContractAddress =
        'Fm8go79jJY2oh86PGKqWvuHwYYFMp7HcLjyyPKG9KiGa';
      break;
  }

  const tx = await invoke(
    {
      contractId: bridgeAdapterContractAddress,
      contractVersion: 1,
      callFunction: 'allow',
      callPayments: [],
      callParams: [
        {
          type: 'string',
          key: 'caller',
          value: coinBridgeContractAddress,
        },
      ],
    },
    deployerPrivateKey,
    network,
    proofsGenerator
  );

  return true;
}
