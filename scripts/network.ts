export type NetworkConfig = {
  name: string;
  nodeAPI: string;
  nodeTimeout: number;
  chainID: number;
  apiKey: string;
  transferFee: number;
  invokeFee: number;
  additionalFee: number;
  issueFee: number;
  setScriptFee: number;
  setWasmScriptFee: number;
};
