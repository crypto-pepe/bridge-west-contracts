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
        '5JJ9nFhnPdrwRm5d6ZT46BqwntXEgkjgXkrtrRo94kQk';
      coinBridgeContractAddress =
        '3n3h2mZcwZeymdE1qwTtmaXpSB4MsyZQA2W1LB3Tr9ir';
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
