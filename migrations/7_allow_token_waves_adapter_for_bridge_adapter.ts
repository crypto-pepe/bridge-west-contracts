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
  let tokenWavesAdapterContractAddress = '';
  let executionChainId = 1;
  switch (network.name) {
    case 'mainnet':
      bridgeAdapterContractAddress = '';
      tokenWavesAdapterContractAddress = '';
      executionChainId = 1;
      throw 'todo'; // TODO: set
      break;
    case 'testnet':
      bridgeAdapterContractAddress =
        '5JJ9nFhnPdrwRm5d6ZT46BqwntXEgkjgXkrtrRo94kQk';
      tokenWavesAdapterContractAddress =
        'HSNBZeJ858vG11a7fB1x65Y5ok4cTR6RhJzcpZo7tHTU';
      executionChainId = 10001;
      break;
  }

  const tx = await invoke(
    {
      contractId: bridgeAdapterContractAddress,
      contractVersion: 1,
      callFunction: 'set_adapter',
      callPayments: [],
      callParams: [
        {
          type: 'integer',
          key: 'execution_chain_id',
          value: executionChainId,
        },
        {
          type: 'string',
          key: 'adapter',
          value: tokenWavesAdapterContractAddress,
        },
      ],
    },
    deployerPrivateKey,
    network,
    proofsGenerator
  );

  return true;
}
