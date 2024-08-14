import { WagmiConfig, createConfig } from "wagmi";
import {
  ConnectKitProvider,
  ConnectKitButton,
  getDefaultConfig,
} from "connectkit";
import { ReactNode, useEffect, useState } from "react";
import { ToastContainer } from "react-toastify";
import Claim from "./components/Claim";
import Deposit from "./components/Deposit";
import { sepolia } from "wagmi/chains";
import Modal from "./components/Modal";
import "react-toastify/dist/ReactToastify.css";
import { clearCookies } from "./libs/utils";

const { VITE_ALCHEMY_ID, VITE_WALLET_CONNECT_ID } = import.meta.env;

const config = createConfig(
  getDefaultConfig({
    alchemyId: VITE_ALCHEMY_ID,
    walletConnectProjectId: VITE_WALLET_CONNECT_ID,
    appName: "Bonsai Pay",
    chains: [sepolia],
  })
);

function App() {
  useEffect(() => {
    clearCookies();
  });

  return (
    <WagmiConfig config={config}>
      <ConnectKitProvider>
        <ToastContainer />
        <div className="app-container">
          <h2 className="title">SP1 Pay Demo</h2>

          <ConnectKitButton mode="light" />
          <ViewSelection />
        </div>
        <ToastContainer />
      </ConnectKitProvider>
    </WagmiConfig>
  );
}

function ViewSelection() {
  const [showComponent, setShowComponent] = useState<"deposit" | "claim">(
    "claim"
  );
  const handleRadioChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setShowComponent(e.target.value as "deposit" | "claim");
  };
  return (
    <div>
      <div className="radio-container">
        <label>
          <input
            className="radio-input"
            type="radio"
            value="deposit"
            checked={showComponent === "deposit"}
            onChange={handleRadioChange}
          />
          Send
        </label>
        <label>
          <input
            className="radio-input"
            type="radio"
            value="claim"
            checked={showComponent === "claim"}
            onChange={handleRadioChange}
          />
          Claim
        </label>
      </div>
      <div className="card">
        {showComponent === "deposit" ? <Deposit /> : <Claim />}
      </div>
    </div>
  );
}

export default App;
