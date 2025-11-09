import { useState, useCallback } from "react";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

type Status =
  | "idle"
  | "checking"
  | "downloading"
  | "installing"
  | "installed"
  | "up_to_date"
  | "error";

export function CheckUpdatesButton() {
  const [status, setStatus] = useState<Status>("idle");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [info, setInfo] = useState<{
    currentVersion: string;
    version: string;
    date?: string;
  } | null>(null);
  const [downloadTotal, setDownloadTotal] = useState<number | null>(null);
  const [downloadedBytes, setDownloadedBytes] = useState<number>(0);

  const handleClick = useCallback(async () => {
    try {
      if (status === "installed") {
        await relaunch();
        return;
      }
      setErrorMessage(null);
      setInfo(null);
      setDownloadTotal(null);
      setDownloadedBytes(0);
      setStatus("checking");
      const update = await check();
      if (!update) {
        setStatus("up_to_date");
        return;
      }
      setInfo({
        currentVersion: update.currentVersion,
        version: update.version,
        date: update.date,
      });
      setStatus("downloading");
      await update.downloadAndInstall((ev) => {
        if (ev.event === "Started") {
          setDownloadTotal(ev.data.contentLength ?? null);
          setDownloadedBytes(0);
        } else if (ev.event === "Progress") {
          setDownloadedBytes((prev) => prev + ev.data.chunkLength);
        } else if (ev.event === "Finished") {
          setDownloadedBytes((prev) => (downloadTotal ? downloadTotal : prev));
        }
      });
      setStatus("installing");
      await new Promise((r) => setTimeout(r, 300));
      setStatus("installed");
    } catch (e) {
      setStatus("error");
      setErrorMessage(String((e as any)?.message ?? e));
    }
  }, [status]);

  const disabled =
    status === "checking" ||
    status === "downloading" ||
    status === "installing";
  const label =
    status === "checking"
      ? "Checking..."
      : status === "downloading"
      ? "Downloading..."
      : status === "installing"
      ? "Installing..."
      : status === "installed"
      ? "Relaunch to update"
      : status === "up_to_date"
      ? "Up to date"
      : status === "error"
      ? "Error, retry"
      : "Check for updates";

  const progress =
    status === "downloading" && downloadTotal && downloadTotal > 0
      ? Math.max(
          0,
          Math.min(100, Math.round((downloadedBytes / downloadTotal) * 100))
        )
      : null;

  return (
    <>
      <button
        className="button"
        onClick={handleClick}
        disabled={disabled}
        title={errorMessage ?? (progress !== null ? `${progress}%` : undefined)}
        style={
          progress !== null
            ? { position: "relative", overflow: "hidden" }
            : undefined
        }
      >
        {progress !== null ? (
          <span
            aria-hidden
            style={{
              position: "absolute",
              left: 0,
              top: 0,
              bottom: 0,
              width: `${progress}%`,
              background: "rgba(43, 108, 176, 0.3)",
            }}
          />
        ) : null}
        <span
          style={
            progress !== null ? { position: "relative", zIndex: 1 } : undefined
          }
        >
          {progress !== null ? `Downloading... ${progress}%` : label}
        </span>
      </button>
      {info ? (
        <div className="settings-section-title-note">
          {info.currentVersion} {"->"} {info.version}
          {info.date ? ` (${info.date})` : ""}
        </div>
      ) : null}
    </>
  );
}
