import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { About } from "./components/About/About";
import { MonitorsProvider } from "./context/MonitorsContext";

function Root() {
  const [hash, setHash] = React.useState<string>(() => window.location.hash);
  React.useEffect(() => {
    const onHash = () => setHash(window.location.hash);
    window.addEventListener("hashchange", onHash);
    return () => window.removeEventListener("hashchange", onHash);
  }, []);

  const isAbout = hash === "#about";
  return (
    <React.StrictMode>
      <MonitorsProvider>{isAbout ? <About /> : <App />}</MonitorsProvider>
    </React.StrictMode>
  );
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <Root />
);
