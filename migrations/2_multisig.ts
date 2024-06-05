import { resolve } from 'path';
import { NetworkConfig } from '../scripts/network';
import { ProofsGenerator, deployWasmScript } from '../scripts/script';
import { Keypair } from '@wavesenterprise/signer';

export default async function (
  deployerSeed: string,
  network: NetworkConfig,
  proofsGenerator: ProofsGenerator
) {
  const deployerPrivateKey = await (
    await Keypair.fromExistingSeedPhrase(deployerSeed)
  ).privateKey();

  const publicKeys =
    'yMQKms5WvLvobErygwGjByEuNuebLMGXHndfVDsjMVD__BN9meJdnaezqtUK7iGhWC9a6TvgU51ESc69wT8x7AnN8__ENV5mvh5GsDNHhqwYt1BzxfZew1M3rRRzXub5vaGxY3C__nobcGCfJ1ZG1J6g8T9dRLoUnBCgQ6DM5H8Hy78sAmSN__Hv2T217jAFbgjXiqrz2CKQkbFH9CJc9dFAgwcQmi3Q83';

  let defaultQuorum = 3;
  switch (network.name) {
    case 'mainnet':
      defaultQuorum = 3;
      break;
    case 'testnet':
      defaultQuorum = 1; // TODO: set to 2
      break;
  }

  const tx = await deployWasmScript(
    'multisig',
    resolve(process.cwd(), './bin/multisig.wasm'),
    [
      { type: 'string', key: 'publicKeys', value: publicKeys },
      { type: 'integer', key: 'quorum', value: defaultQuorum },
    ],
    [],
    deployerPrivateKey,
    network,
    proofsGenerator
  ).catch((e) => {
    throw e;
  });

  console.log('Multisig contract deployed at contractId = ' + tx.tx.id);

  return true;
}
