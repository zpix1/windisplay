import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { MonitorsProvider } from "./context/MonitorsContext";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <MonitorsProvider>
      <App />
    </MonitorsProvider>
  </React.StrictMode>
);
