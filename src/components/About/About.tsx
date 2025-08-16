import { useEffect, useState } from "react";
import { getName, getVersion } from "@tauri-apps/api/app";
import { path } from "@tauri-apps/api";
import { openPath } from "@tauri-apps/plugin-opener";
import { Link } from "../ui/Link/Link";

import "./About.css";

export function About() {
  const [name, setName] = useState<string>("WinDisplay");
  const [version, setVersion] = useState<string>("");

  useEffect(() => {
    Promise.all([
      getName().catch(() => "WinDisplay"),
      getVersion().catch(() => ""),
    ])
      .then(([n, v]) => {
        setName(n);
        setVersion(v);
      })
      .catch(() => {});
  }, []);

  return (
    <div className="about-container">
      <span className="about-title">
        <span className="about-name">{name}</span>
        <span className="about-version">{version && `v${version}`}</span>
        <Link
          className="about-github"
          target="_blank"
          href="https://github.com/zpix1/windisplay"
        >
          GitHub
        </Link>
      </span>
      <div className="about-description">
        Manage your display settings quickly from the tray. <br />
        Report issues{" "}
        <Link href="https://github.com/zpix1/windisplay/issues">here</Link>.
        When reporting an issue, please attach the log file.
        <div style={{ marginTop: 8 }}>
          <button
            onClick={async () => {
              try {
                const logDir = await path.appLogDir();
                await openPath(logDir);
              } catch (error) {
                console.error("Error opening log directory:", error);
              }
            }}
          >
            Open log directory
          </button>
        </div>
        <br />
        Created by <Link href="https://github.com/zpix1">@zpix1</Link>. <br />
        If you liked this app, support me{" "}
        <Link href="https://zpix1.github.io/donate/">here</Link>.
      </div>
    </div>
  );
}
