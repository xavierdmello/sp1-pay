export default [
  {
    "type": "constructor",
    "inputs": [
      {
        "name": "_verifier",
        "type": "address",
        "internalType": "contract ISP1Verifier"
      },
      { "name": "_bonsaiPayVKey", "type": "bytes32", "internalType": "bytes32" }
    ],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "balanceOf",
    "inputs": [
      { "name": "claimId", "type": "bytes32", "internalType": "bytes32" }
    ],
    "outputs": [{ "name": "", "type": "uint256", "internalType": "uint256" }],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "bonsaiPayVKey",
    "inputs": [],
    "outputs": [{ "name": "", "type": "bytes32", "internalType": "bytes32" }],
    "stateMutability": "view"
  },
  {
    "type": "function",
    "name": "claim",
    "inputs": [
      { "name": "proof", "type": "bytes", "internalType": "bytes" },
      { "name": "publicValues", "type": "bytes", "internalType": "bytes" }
    ],
    "outputs": [],
    "stateMutability": "nonpayable"
  },
  {
    "type": "function",
    "name": "deposit",
    "inputs": [
      { "name": "claimId", "type": "bytes32", "internalType": "bytes32" }
    ],
    "outputs": [],
    "stateMutability": "payable"
  },
  {
    "type": "function",
    "name": "verifier",
    "inputs": [],
    "outputs": [
      { "name": "", "type": "address", "internalType": "contract ISP1Verifier" }
    ],
    "stateMutability": "view"
  },
  {
    "type": "event",
    "name": "Claimed",
    "inputs": [
      {
        "name": "recipient",
        "type": "address",
        "indexed": true,
        "internalType": "address"
      },
      {
        "name": "claimId",
        "type": "bytes32",
        "indexed": true,
        "internalType": "bytes32"
      },
      {
        "name": "amount",
        "type": "uint256",
        "indexed": false,
        "internalType": "uint256"
      }
    ],
    "anonymous": false
  },
  {
    "type": "event",
    "name": "Deposited",
    "inputs": [
      {
        "name": "claimId",
        "type": "bytes32",
        "indexed": true,
        "internalType": "bytes32"
      },
      {
        "name": "amount",
        "type": "uint256",
        "indexed": false,
        "internalType": "uint256"
      }
    ],
    "anonymous": false
  },
  {
    "type": "error",
    "name": "InvalidClaim",
    "inputs": [
      { "name": "message", "type": "string", "internalType": "string" }
    ]
  },
  {
    "type": "error",
    "name": "InvalidDeposit",
    "inputs": [
      { "name": "message", "type": "string", "internalType": "string" }
    ]
  },
  { "type": "error", "name": "TransferFailed", "inputs": [] }
] as const;
