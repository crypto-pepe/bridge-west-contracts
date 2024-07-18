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
  let feeChainId = 1;
  let callerContractAddress = '';
  switch (network.name) {
    case 'mainnet':
      multisigContractAddress = '';
      executorContractAddress = '';
      bridgeAdapterContractAddress = '';
      feeRecipientAddress = '';
      throw 'todo'; // TODO: set
      break;
    case 'testnet':
      multisigContractAddress = '73YBejz5FnV1csCwErwC6CS5tt7VY1cpS8eRfgWfJd3F';
      executorContractAddress = 'GMRbgPfkih3aXMpxUcPZ2XpSwBqVTKNoURs4vE1ewASK';
      bridgeAdapterContractAddress =
        '6eF3nNZhKc8ahA6NDpCXj7tVA6naXnoDuyiHV12KJmip';
      feeRecipientAddress = '3N1VhCMKNh2SBgw9mhdLRBjyucnv6fkMNBA';
      feeChainId = 10001;
      callerContractAddress = '3N7gP7bxevss5mVwkvMMJCnzPSP8c1YJCjw';
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
      { type: 'integer', key: 'fee_chain_id', value: feeChainId },
      { type: 'string', key: 'caller_contract', value: callerContractAddress },
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
