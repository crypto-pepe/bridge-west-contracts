import { Keypair } from '@wavesenterprise/signer';
import { invoke } from '../scripts/transaction';
import { NetworkConfig } from '../scripts/network';
import { ProofsGenerator } from '../scripts/script';

export default async function (
  deployerSeed: string,
  network: NetworkConfig,
  proofsGenerator: ProofsGenerator
) {
  const deployerPrivateKey = await (
    await Keypair.fromExistingSeedPhrase(deployerSeed)
  ).privateKey();

  let coinBridgeContractAddress = '';
  let callerContractAddress = ''; // token_west_adapter
  switch (network.name) {
    case 'mainnet':
      throw 'todo'; // TODO: set
      break;
    case 'testnet':
      coinBridgeContractAddress =
        'Fm8go79jJY2oh86PGKqWvuHwYYFMp7HcLjyyPKG9KiGa';
      callerContractAddress = '3N7gP7bxevss5mVwkvMMJCnzPSP8c1YJCjw';
      break;
  }

  await invoke(
    {
      contractId: coinBridgeContractAddress,
      contractVersion: 1,
      callFunction: 'update_caller_contract',
      callPayments: [],
      callParams: [
        {
          type: 'string',
          key: 'caller_contract',
          value: callerContractAddress,
        },
      ],
    },
    deployerPrivateKey,
    network,
    proofsGenerator
  );

  return true;
}
