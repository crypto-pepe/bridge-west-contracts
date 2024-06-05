import { TRANSACTIONS } from '@wavesenterprise/transactions-factory';
import { ContractParam } from '@wavesenterprise/signature-generator';
import { NetworkConfig } from './network';
import { broadcastTx } from './helpers';
import { ProofsGenerator } from './script';
import { Keypair, Signer } from '@wavesenterprise/signer';
import { We } from '@wavesenterprise/sdk';

export const data = async (
  params: {
    data: ContractParam[];
    fee?: number | undefined;
    feeAssetId?: string | undefined;
  },
  senderPrivateKey: string,
  network: NetworkConfig,
  proofsGenerator: ProofsGenerator | undefined = undefined
): Promise<any> => {
  const keypair = await Keypair.fromString(senderPrivateKey);

  const tx = TRANSACTIONS.Data.V3({
    data: params.data,
    fee: params.fee || network.transferFee,
    feeAssetId: params.feeAssetId,
    senderPublicKey: await keypair.publicKey(),
  });

  const signer = new Signer();
  const signedTx = await signer.getSignedTx(tx, keypair);

  if (proofsGenerator !== undefined) {
    const proofs = await proofsGenerator(
      await tx.getBytes(),
      await signedTx.getId()
    );
    proofs.forEach((proof) => signedTx.proofs.add(proof));
  }

  return broadcastTx(signedTx, network);
};

export const transfer = async (
  params: {
    recipient: string;
    amount: number;
    assetId?: string | undefined;
    attachment?: string | undefined;
    fee?: number | undefined;
    feeAssetId?: string | undefined;
  },
  senderPrivateKey: string,
  network: NetworkConfig,
  proofsGenerator: ProofsGenerator | undefined = undefined
): Promise<any> => {
  const keypair = await Keypair.fromString(senderPrivateKey);

  const tx = TRANSACTIONS.Transfer.V3({
    recipient: params.recipient,
    amount: params.amount,
    assetId: params.assetId,
    attachment: params.attachment || '',
    fee: params.fee || network.transferFee,
    feeAssetId: params.feeAssetId,
    senderPublicKey: await keypair.publicKey(),
  });

  const signer = new Signer();
  const signedTx = await signer.getSignedTx(tx, keypair);

  if (proofsGenerator !== undefined) {
    const proofs = await proofsGenerator(
      await tx.getBytes(),
      await signedTx.getId()
    );
    proofs.forEach((proof) => signedTx.proofs.add(proof));
  }

  return broadcastTx(signedTx, network);
};

export const issue = async (
  params: {
    name: string;
    description: string;
    quantity: number;
    decimals: number;
    reissuable: boolean;
    fee?: number | undefined;
  },
  senderPrivateKey: string,
  network: NetworkConfig,
  proofsGenerator: ProofsGenerator | undefined = undefined
): Promise<any> => {
  const keypair = await Keypair.fromString(senderPrivateKey);

  const tx = TRANSACTIONS.Issue.V3({
    name: params.name,
    description: params.description,
    quantity: params.quantity,
    decimals: params.decimals,
    reissuable: params.reissuable,
    fee: params.fee || network.issueFee,
    chainId: network.chainID,
    senderPublicKey: await keypair.publicKey(),
  });

  const signer = new Signer();
  const signedTx = await signer.getSignedTx(tx, keypair);

  if (proofsGenerator !== undefined) {
    const proofs = await proofsGenerator(
      await tx.getBytes(),
      await signedTx.getId()
    );
    proofs.forEach((proof) => signedTx.proofs.add(proof));
  }

  return broadcastTx(signedTx, network);
};

export const invoke = async (
  params: {
    contractId: string;
    contractVersion: number;
    callFunction: string;
    callParams: ContractParam[];
    callPayments: {
      assetId?: string;
      amount: number;
    }[];
    fee?: number | undefined;
    feeAssetId?: string | undefined;
  },
  senderPrivateKey: string,
  network: NetworkConfig,
  proofsGenerator: ProofsGenerator | undefined = undefined
): Promise<any> => {
  const keypair = await Keypair.fromString(senderPrivateKey);

  const tx = TRANSACTIONS.CallContract.V7({
    contractId: params.contractId,
    contractVersion: params.contractVersion,
    contractEngine: 'wasm',
    callFunc: params.callFunction,
    params: params.callParams,
    payments: params.callPayments,
    fee: params.fee || network.invokeFee,
    feeAssetId: params.feeAssetId,
    senderPublicKey: await keypair.publicKey(),
  });

  const signer = new Signer();
  const signedTx = await signer.getSignedTx(tx, keypair);

  if (proofsGenerator !== undefined) {
    const proofs = await proofsGenerator(
      await tx.getBytes(),
      await signedTx.getId()
    );
    proofs.forEach((proof) => signedTx.proofs.add(proof));
  }

  return broadcastTx(signedTx, network, true);
};

const getContractValue = async <T>(
  dApp: string,
  key: string,
  network: NetworkConfig
): Promise<T> => {
  const we = new We(network.nodeAPI);
  const { value } = (await we.contracts.getKey(dApp, key)) as ContractParam;

  return value as T;
};

export const getIntegerContractValue = async (
  dApp: string,
  key: string,
  network: NetworkConfig
): Promise<number> => getContractValue<number>(dApp, key, network);

export const getStringContractValue = async (
  dApp: string,
  key: string,
  network: NetworkConfig
): Promise<string> => getContractValue<string>(dApp, key, network);

export const getContractInfo = async (
  dApp: string,
  network: NetworkConfig
): Promise<unknown> => {
  const we = new We(network.nodeAPI);

  return await we.contracts.info(dApp);
};
