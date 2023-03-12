import { StrictMode } from "react";
import { createRoot } from 'react-dom/client';
import { ChakraProvider } from "@chakra-ui/react";
import init, { set_panic_hook } from "spc-wasm";
import App from "./App";
import "./index.css";

init().then(() => {
  set_panic_hook();
  createRoot(document.getElementById("root")!).render(
    <StrictMode>
      <ChakraProvider>
        <App />
      </ChakraProvider>
    </StrictMode>
  );
});
