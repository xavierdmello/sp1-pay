import React, { useState, useCallback, useEffect } from "react";
import Account from "./Account";
import { useBonsaiPayClaimedEvent, useBonsaiPayBalanceOf } from "../generated";
import { sha256 } from "@noble/hashes/sha256";
import { toHex } from "viem";
import { useContractWrite, usePrepareContractWrite } from "wagmi";
import SP1PayABI from "../abi/SP1Pay";
interface ProveProps {
  disabled: boolean;
  email: string | null;
}

let bonsaiPayAddress = import.meta.env.VITE_CUSTODY_ADDRESS;

const Prove: React.FC<ProveProps> = ({ disabled, email }) => {
  const [isLoading, setIsLoading] = useState(false);
  const [isClaimed, setIsClaimed] = useState(false);
  const [isNonZeroBalance, setIsNonZeroBalance] = useState(false);
  const [proof, setProof] = useState<string | null>(null);
  const [publicValues, setPublicValues] = useState<string | null>(null);

  const { config } = usePrepareContractWrite({
    address: bonsaiPayAddress,
    abi: SP1PayABI,
    functionName: "claim",
    args: [proof ? `${proof}` : null, publicValues ? `${publicValues}` : null],
  });

  const {
    data,
    isLoading: isTxLoading,
    isSuccess,
    write,
  } = useContractWrite(config);

  useEffect(() => {
    if (proof && publicValues) {
      write?.();
    }
  }, [proof, publicValues, write]);

  useBonsaiPayClaimedEvent({
    listener: () => {
      setIsClaimed(true);
    },
  });

  const { data: balance } = useBonsaiPayBalanceOf({
    args: [toHex(sha256(email ?? ""))],
  });

  useEffect(() => {
    setIsNonZeroBalance(balance !== 0n);
  }, [balance]);

  const { VITE_API_HOST } = import.meta.env;

  const handleClick = useCallback(async () => {
    setIsLoading(true);

    const jwtCookie = document.cookie
      .split("; ")
      .find((row) => row.startsWith("jwt="));
    const jwt = jwtCookie?.split("=")[1];

    if (!jwt) {
      console.error("JWT not found");
      setIsLoading(false);
      return;
    }

    try {
      const response = await fetch(`${VITE_API_HOST}/api/prove`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "X-Auth-Token": jwt,
        },
        body: JSON.stringify({ jwt }),
      });

      if (response.ok) {
        const data = await response.json();
        setProof(data.proof);
        setPublicValues(data.publicValues);
      } else {
        throw new Error("Response not OK");
      }
    } catch (error) {
      console.error("Error fetching data:", error);
    } finally {
      setIsLoading(false);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <>
      <Account email={email} disabled={disabled} hideClaim={true} />
      <button
        onClick={handleClick}
        disabled={isLoading || disabled || isClaimed || !isNonZeroBalance}
      >
        {isClaimed ? "Claimed" : isLoading ? "Proving..." : "Prove with SP1"}
      </button>
      {isLoading ? <p>This will take a few minutes...</p> : <p></p>}
    </>
  );
};

export default Prove;
