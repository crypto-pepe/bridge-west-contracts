import inquirer from 'inquirer';
import { WavesLedger } from '@waves/ledger';
import TransportNodeHid from '@ledgerhq/hw-transport-node-hid-singleton';
import { getEnvironmentByName } from 'relax-env-json';
import { exit } from 'process';
import { TRANSACTIONS } from '@wavesenterprise/transactions-factory';
import { SignedTx } from '@wavesenterprise/signer';
import { getTransactionId } from '@wavesenterprise/crypto-utils';
import { broadcastTx } from './helpers';
import { IUserData } from '@waves/ledger/lib/Waves';
import { NetworkConfig } from './network';

(async () => {
  console.log('Plug-in your ledger device and enter to WAVES application\n');

  let networkName: string = '';
  let dappToConfirm: string = '';
  let txToConfirm: string = '';

  await inquirer
    .prompt([
      {
        type: 'list',
        name: 'network',
        message: 'Select network for signing tx',
        waitUserInput: true,
        choices: ['mainnet', 'testnet'],
      },
      {
        type: 'string',
        name: 'dapp',
        message: 'Enter dapp to confirm tx in',
        waitUserInput: true,
      },
      {
        type: 'string',
        name: 'tx',
        message: 'Enter tx to confirm',
        waitUserInput: true,
      },
    ])
    .then((answers) => {
      networkName = answers.network;
      dappToConfirm = answers.dapp;
      txToConfirm = answers.tx;
      // switch (answers.network) {
      //   case 'mainnet':
      //     // switch (answers.chain) {
      //     //   case 'waves':
      //     //     networkByte = 87;
      //     //     break;
      //     //   case 'west':
      //     //     networkByte = 86;
      //     //     break;
      //     // }
      //     networkByte = 86;
      //     break;
      //   case 'testnet':
      //     networkByte = 84;
      //     break;
      // }
    })
    .catch((e) => {
      throw JSON.stringify(e);
    });

  let isWavesAppOpened = false;
  while (!isWavesAppOpened) {
    await inquirer
      .prompt([
        {
          type: 'confirm',
          name: 'isWavesAppOpened',
          message: 'Have you opened WAVES app on ledger?',
          waitUserInput: true,
        },
      ])
      .then((answers) => {
        isWavesAppOpened = answers.isWavesAppOpened;
      })
      .catch((e) => {
        throw JSON.stringify(e);
      });
  }

  const network: NetworkConfig = getEnvironmentByName(networkName).network;
  const multisigContractAddress =
    getEnvironmentByName(networkName).multisigContractAddress;

  let userId = -1;
  let userIdConfirmed = false;
  let user: IUserData = {
    publicKey: '',
    address: '',
    statusCode: '',
  };

  while (!userIdConfirmed) {
    const ledger = new WavesLedger({
      debug: true,
      openTimeout: 5000,
      listenTimeout: 30000,
      exchangeTimeout: 30000,
      networkCode: network.chainID,
      transport: TransportNodeHid,
    });

    await inquirer
      .prompt([
        {
          type: 'number',
          name: 'userId',
          message: 'Which user id need to use?',
          waitUserInput: true,
          validate: (input) => {
            return new Promise((resolve) => {
              const parsed = parseInt(input);
              resolve(
                parsed >= 0 && parsed <= 1000
                  ? true
                  : 'You can provide user id from 0 to 1000'
              );
            });
          },
          filter: (input) => {
            const parsed = parseInt(input);
            return isNaN(parsed) || parsed < 0 || parsed > 1000 ? '' : parsed;
          },
        },
      ])
      .then((answers) => {
        userId = answers.userId;
      })
      .catch((e) => {
        throw JSON.stringify(e);
      });

    const userData = await ledger.getUserDataById(userId);
    console.log(userData);
    user = userData;

    await inquirer
      .prompt([
        {
          type: 'confirm',
          name: 'userIdConfirmed',
          message: 'Confirm to use ' + userId + ' for signing',
          waitUserInput: true,
        },
      ])
      .then((answers) => {
        userIdConfirmed = answers.userIdConfirmed;
      })
      .catch((e) => {
        throw JSON.stringify(e);
      });
  }

  const ledger = new WavesLedger({
    debug: true,
    openTimeout: 5000,
    listenTimeout: 30000,
    exchangeTimeout: 30000,
    networkCode: network.chainID,
    transport: TransportNodeHid,
  });

  // const tx = TRANSACTIONS.Transfer.V3({
  //   recipient: '3MpHCHnKPqXVEbypoGFSQq3r2qNZVqfe7zY',
  //   amount: 100000000,
  //   assetId: undefined,
  //   attachment: '',
  //   fee: 1000000,
  //   feeAssetId: undefined,
  //   senderPublicKey: 'yMQKms5WvLvobErygwGjByEuNuebLMGXHndfVDsjMVD',
  // });

  // const tx = TRANSACTIONS.CallContract.V7({
  //   contractId: 'FzXKFUN1VKSr6D5LgxmHEaD9rPkK3uwnHsStMA3SrnSJ',
  //   contractVersion: 1,
  //   contractEngine: 'wasm',
  //   callFunc: 'set_completed',
  //   params: [{ type: 'integer', key: 'completed', value: 23 }],
  //   payments: [],
  //   fee: network.invokeFee,
  //   feeAssetId: undefined,
  //   senderPublicKey: user.publicKey,
  // });

  const tx = TRANSACTIONS.CallContract.V7({
    contractId: multisigContractAddress,
    contractVersion: 1,
    contractEngine: 'wasm',
    callFunc: 'confirm_transaction',
    params: [
      { type: 'string', key: 'dapp', value: dappToConfirm },
      { type: 'string', key: 'tx_id', value: txToConfirm },
    ],
    payments: [],
    fee: network.invokeFee,
    feeAssetId: undefined,
    senderPublicKey: user.publicKey,
  });

  const txBytes = await tx.getBytes();

  console.log(tx);
  console.log('VERIFY AND SIGN TX ON YOUR DEVICE!');

  const signature = await ledger.signSomeData(userId, { dataBuffer: txBytes });
  console.log('Your signature:', signature);

  const signedTx = new SignedTx(tx);
  const txId = getTransactionId(txBytes);

  signedTx.setId(txId);
  signedTx.proofs.add(signature);

  const broadcastedTx = await broadcastTx(signedTx, network, true).catch(
    (e) => {
      console.log(e);
      throw e;
    }
  );

  console.log(broadcastedTx);

  exit(0);
})().catch((e) => {
  throw e;
});
