import { useEffect, useState } from "react";
import { getName, getVersion } from "@tauri-apps/api/app";
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
        Manage your display settings quickly from the tray <br />
        Created by <Link href="https://github.com/zpix1">@zpix1</Link> <br />
        Support me on <Link href="https://www.patreon.com/zpix1">Patreon</Link>
      </div>
    </div>
  );
}
