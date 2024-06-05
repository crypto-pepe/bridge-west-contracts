import { Keypair } from '@wavesenterprise/signer';
import { getAddressFromPrivateKey } from './helpers';

(async () => {
  const seed = '';
  const networkByte: number = 'T'.charCodeAt(0);
  console.log(networkByte);

  const kp = await Keypair.fromExistingSeedPhrase(seed);
  const address = await getAddressFromPrivateKey(
    await kp.privateKey(),
    networkByte
  );

  console.log(address);
})();
