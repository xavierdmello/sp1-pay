// Copyright 2024 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.20;

import {Script} from "forge-std/Script.sol";
import {console2} from "forge-std/console2.sol";
import {ISP1Verifier} from "@sp1-contracts/ISP1Verifier.sol";
import {SP1MockVerifier} from "@sp1-contracts/SP1MockVerifier.sol";
import {BonsaiPay} from "../src/BonsaiPay.sol";

/// @notice Deployment script for the RISC Zero starter project.
/// @dev Use the following environment variable to control the deployment:
///     * ETH_WALLET_PRIVATE_KEY private key of the wallet to be used for deployment.
///
/// See the Foundry documentation for more information about Solidity scripts.
/// https://book.getfoundry.sh/tutorials/solidity-scripting
contract BonsaiPayDeploy is Script {
    function run() external {
        uint256 deployerKey = uint256(vm.envBytes32("ETH_WALLET_PRIVATE_KEY"));

        vm.startBroadcast(deployerKey);

        ISP1Verifier verifier;
        // Detect if the SP1_PROVER is set to mock, and pick the correct verifier.
        string memory mockStr = "mock";
        if (
            keccak256(abi.encodePacked(vm.envString("SP1_PROVER")))
                == keccak256(abi.encodePacked(mockStr))
        ) {
            verifier = ISP1Verifier(address(new SP1MockVerifier()));
        } else {
            verifier = ISP1Verifier(address(vm.envAddress("SP1_VERIFIER_ADDRESS")));
        }
        
        BonsaiPay bonsaiPay = new BonsaiPay(verifier, vm.envBytes32("SP1_PAY_PROGRAM_VKEY"), vm.envBytes("CERT"));
        console2.log("Deployed BonsaiPay to", address(bonsaiPay));

        vm.stopBroadcast();
    }
}
