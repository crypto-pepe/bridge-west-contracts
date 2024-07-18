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

  let coinBridgeContractAddress = '';
  let executionChainId = 1;
  let executionAsset = '';
  switch (network.name) {
    case 'mainnet':
      coinBridgeContractAddress = '';
      executionChainId = 1;
      executionAsset = '';
      throw 'todo'; // TODO: set
      break;
    case 'testnet':
      coinBridgeContractAddress =
        'Fm8go79jJY2oh86PGKqWvuHwYYFMp7HcLjyyPKG9KiGa';
      executionChainId = 10001;
      executionAsset = '3MtgBm7ZQPKWvxURxT16Vujk1iEdoefbv8o';
      break;
  }

  await invoke(
    {
      contractId: coinBridgeContractAddress,
      contractVersion: 1,
      callFunction: 'update_execution_chain',
      callPayments: [],
      callParams: [
        {
          type: 'integer',
          key: 'execution_chain_id',
          value: executionChainId,
        },
        {
          type: 'boolean',
          key: 'enabled',
          value: true,
        },
      ],
    },
    deployerPrivateKey,
    network,
    proofsGenerator
  );

  await invoke(
    {
      contractId: coinBridgeContractAddress,
      contractVersion: 1,
      callFunction: 'update_binding_info',
      callPayments: [],
      callParams: [
        {
          type: 'integer',
          key: 'execution_chain_id',
          value: executionChainId,
        },
        {
          type: 'string',
          key: 'execution_asset',
          value: executionAsset,
        },
        {
          type: 'integer',
          key: 'min_amount',
          value: 15000000000,
        },
        {
          type: 'integer',
          key: 'min_fee',
          value: 1000000000,
        },
        {
          type: 'integer',
          key: 'threshold_fee',
          value: 33000000000000,
        },
        {
          type: 'integer',
          key: 'before_percent_fee',
          value: 500,
        },
        {
          type: 'integer',
          key: 'after_percent_fee',
          value: 0,
        },
        {
          type: 'boolean',
          key: 'enabled',
          value: true,
        },
      ],
    },
    deployerPrivateKey,
    network,
    proofsGenerator
  );

  return true;
}
