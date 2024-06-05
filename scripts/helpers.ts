import { We } from '@wavesenterprise/sdk';
import { NetworkConfig } from './network';
import { SignedTx } from '@wavesenterprise/signer';
import {
  createAddress,
  fromBase58,
  toBase58,
} from '@wavesenterprise/crypto-utils';
import { Keypair } from '@wavesenterprise/signer';

export type BroadcastedTx = {
  id: string;
  senderPublicKey: string;
  type: number;
  version: number;
  fee: number;
  timestamp: string | number;
};

export type ExecutedTxFor = {
  id: string;
  senderPublicKey: string;
  type: number;
  version: number;
  fee: number;
  timestamp: string | number;
  errorMessage: string;
  statusCode: number;
  tx: BroadcastedTx;
};

export type CancellablePromise<T> = Promise<T> & { cancel: () => void };

const delay = (timeout: number): CancellablePromise<{}> => {
  const t: any = {};

  const p = new Promise((resolve, _) => {
    t.resolve = resolve;
    t.id = setTimeout(resolve, timeout);
  }) as any;

  (<any>p).cancel = () => {
    t.resolve();
    clearTimeout(t.id);
  };

  return p;
};

export async function waitForTx(
  txId: string,
  network: NetworkConfig
): Promise<unknown> {
  let expired = false;
  const to = delay(network.nodeTimeout);
  to.then(() => (expired = true));

  const we = new We(network.nodeAPI);
  const promise = (): Promise<unknown> =>
    we.transactions
      .info(txId)
      .then((x) => {
        to.cancel();
        return x as any;
      })
      .catch((_) =>
        delay(1000).then((_) =>
          expired
            ? Promise.reject(new Error('Tx wait stopped: timeout'))
            : promise()
        )
      );

  return promise();
}

export async function waitForExecutedTx(
  txId: string,
  network: NetworkConfig
): Promise<unknown> {
  let expired = false;
  const to = delay(network.nodeTimeout);
  to.then(() => (expired = true));

  const we = new We(network.nodeAPI);
  const promise = (): Promise<unknown> =>
    we.contracts
      .executedTxFor(txId)
      .then((x) => {
        to.cancel();
        return x as any;
      })
      .catch((_) =>
        delay(1000).then((_) =>
          expired
            ? Promise.reject(new Error('Tx wait stopped: timeout'))
            : promise()
        )
      );

  return promise();
}

export const broadcastTx = async (
  tx: SignedTx<any>,
  network: NetworkConfig,
  needToWaitForExecutedTx: boolean = false
): Promise<ExecutedTxFor | BroadcastedTx> => {
  const we = new We(network.nodeAPI);
  const txId = await tx.getId();

  await we.broadcast(tx).catch((e) => {
    throw e;
  });
  const txInfo = (await waitForTx(txId, network).catch((e) => {
    throw e;
  })) as BroadcastedTx;

  if (needToWaitForExecutedTx) {
    const resultTx = (await waitForExecutedTx(txId, network).catch((e) => {
      throw e;
    })) as ExecutedTxFor;

    if (resultTx.statusCode !== 0) {
      throw resultTx.errorMessage;
    }
    return resultTx;
  }

  return txInfo;
};

export const getPublicKeyFromPrivateKey = async (
  privateKey: string
): Promise<string> => {
  return (await Keypair.fromString(privateKey)).publicKey();
};

export const getAddressFromPrivateKey = async (
  privateKey: string,
  chainID: number
): Promise<string> => {
  return toBase58(
    createAddress(
      fromBase58(await (await Keypair.fromString(privateKey)).publicKey()),
      chainID
    )
  );
};

export const getAddressFromPublicKey = async (
  publicKey: string,
  chainID: number
): Promise<string> => {
  return toBase58(createAddress(fromBase58(publicKey), chainID));
};
