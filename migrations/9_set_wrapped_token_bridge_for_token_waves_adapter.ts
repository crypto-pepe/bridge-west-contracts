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

  let tokenWavesAdapterContractAddress = '';
  let wrappedTokenBridgeOnWaves = '';
  switch (network.name) {
    case 'mainnet':
      tokenWavesAdapterContractAddress = '';
      throw 'todo'; // TODO: set
      break;
    case 'testnet':
      tokenWavesAdapterContractAddress =
        'HSNBZeJ858vG11a7fB1x65Y5ok4cTR6RhJzcpZo7tHTU';
      wrappedTokenBridgeOnWaves = '3MybRwk9oKz173iravaDXB3jJDqSZWUEGGM';
      break;
  }

  await invoke(
    {
      contractId: tokenWavesAdapterContractAddress,
      contractVersion: 1,
      callFunction: 'set_wrapped_token_bridge_contract',
      callPayments: [],
      callParams: [
        {
          type: 'string',
          key: 'contract',
          value: wrappedTokenBridgeOnWaves,
        },
      ],
    },
    deployerPrivateKey,
    network,
    proofsGenerator
  );

  return true;
}
