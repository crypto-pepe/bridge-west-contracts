import { compile, ICompilationError } from '@waves/ride-js';
import { TRANSACTIONS } from '@wavesenterprise/transactions-factory';
import { ContractParam } from '@wavesenterprise/signature-generator';
import { Keypair, Signer } from '@wavesenterprise/signer';
import { readFile } from 'fs/promises';
import { NetworkConfig } from './network';
import { broadcastTx, ExecutedTxFor } from './helpers';
import { bytesToHex, sha256 } from '@wavesenterprise/crypto-utils';

export type ProofsGenerator = (
  tx: Uint8Array,
  txId: string
) => Promise<string[]>;

export type CompilationResult = {
  result: {
    base64: string;
    globalVariableComplexities: Record<string, number>;
    callableComplexities: Record<string, number>;
    bytes: Uint8Array;
    size: number;
    // ast: object;
    // complexity: number;
    // verifierComplexity?: number;
    // callableComplexities?: Record<string, number>;
    // userFunctionsComplexity?: Record<string, number>;
    // stateCallsComplexities?: Record<string, number>;
  };
};

export function estimateFeeForRideScript(scriptSize: number): number {
  const feePerKb = 100000;
  const extraFee = 400000;
  const maxScriptSize = 160 * 1024;
  if (scriptSize > maxScriptSize) {
    throw new Error('Max script size exceeded');
  }

  return Math.ceil(scriptSize / 1024) * feePerKb + extraFee;
}

export const compileRideScript = async (
  pathToScript: string,
  needCompaction = false,
  removeUnusedCode = false
) => {
  const rawScript = await readFile(pathToScript, 'utf8');
  const compiledScript = compile(
    rawScript,
    undefined,
    needCompaction,
    removeUnusedCode
  ) as CompilationResult | ICompilationError;

  const isICompilationError = (
    x: CompilationResult | ICompilationError
  ): x is ICompilationError => {
    return 'error' in x;
  };

  if (isICompilationError(compiledScript)) {
    throw new Error(compiledScript.error);
  }

  return compiledScript.result;
};

export const deployRideScript = async (
  script: string,
  deployerPrivateKey: string,
  network: NetworkConfig,
  proofsGenerator: ProofsGenerator | undefined = undefined,
  fee: number | undefined = undefined
) => {
  const keypair = await Keypair.fromString(deployerPrivateKey);

  const tx = TRANSACTIONS.SetScript.V1({
    script: script,
    chainId: network.chainID,
    fee: fee || network.setScriptFee,
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

export const deployWasmScript = async (
  contractName: string,
  pathToScript: string,
  constructorParams: ContractParam[],
  constructorPayments: { assetId?: string | undefined; amount: number }[],
  deployerPrivateKey: string,
  network: NetworkConfig,
  proofsGenerator: ProofsGenerator | undefined = undefined,
  fee: number | undefined = undefined
) => {
  const keypair = await Keypair.fromString(deployerPrivateKey);

  const script: Buffer = await readFile(pathToScript);
  const base64script = script.toString('base64');
  const wasm = {
    bytecode: base64script,
    bytecodeHash: bytesToHex(sha256(script)),
  };

  const tx = TRANSACTIONS.CreateContract.V7({
    contractName: contractName,
    params: constructorParams,
    payments: constructorPayments,
    storedContract: wasm,
    validationPolicy: { type: 'any' },
    isConfidential: false,
    groupOwners: [],
    groupParticipants: [],
    fee: fee || network.setWasmScriptFee,
    senderPublicKey: await keypair.publicKey(),
  });

  const signer = new Signer();
  const signedTx = await signer.getSignedTx(tx, keypair).catch((e) => {
    throw e;
  });

  if (proofsGenerator !== undefined) {
    const proofs = await proofsGenerator(
      await tx.getBytes(),
      await signedTx.getId()
    );
    proofs.forEach((proof) => signedTx.proofs.add(proof));
  }

  return (await broadcastTx(signedTx, network, true)) as ExecutedTxFor;
};

export const redeployWasmScript = async (
  contractAddress: string,
  pathToScript: string,
  deployerPrivateKey: string,
  network: NetworkConfig,
  proofsGenerator: ProofsGenerator | undefined = undefined,
  fee: number | undefined = undefined
) => {
  const keypair = await Keypair.fromString(deployerPrivateKey);

  const script: Buffer = await readFile(pathToScript);
  const base64script = script.toString('base64');
  const wasm = {
    bytecode: base64script,
    bytecodeHash: bytesToHex(sha256(script)),
  };

  const tx = TRANSACTIONS.UpdateContract.V6({
    contractId: contractAddress,
    storedContract: wasm,
    validationPolicy: { type: 'any' },
    groupOwners: [],
    groupParticipants: [],
    fee: fee || network.setWasmScriptFee,
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
