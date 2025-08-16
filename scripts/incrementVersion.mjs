import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";

function isValidSemver(version) {
  return /^\d+\.\d+\.\d+$/.test(version);
}

function bumpSemver(version, kind) {
  if (!isValidSemver(version)) {
    throw new Error(`Invalid semver: ${version}`);
  }
  const [major, minor, patch] = version.split(".").map((n) => Number(n));
  if (kind === "major") {
    return `${major + 1}.0.0`;
  }
  if (kind === "minor") {
    return `${major}.${minor + 1}.0`;
  }
  return `${major}.${minor}.${patch + 1}`;
}

function parseArgs(argv) {
  const args = { mode: "patch", to: null };
  for (const arg of argv.slice(2)) {
    if (arg === "major" || arg === "minor" || arg === "patch") {
      args.mode = arg;
      continue;
    }
    if (arg.startsWith("--to=")) {
      args.to = arg.slice("--to=".length);
      continue;
    }
    if (arg === "-t") {
      const idx = argv.indexOf("-t");
      const next = argv[idx + 1];
      if (next) args.to = next;
      continue;
    }
  }
  return args;
}

async function updateJsonFile(filePath, updater) {
  const jsonText = await readFile(filePath, "utf8");
  const data = JSON.parse(jsonText);
  const updated = updater(data);
  await writeFile(filePath, JSON.stringify(updated, null, 2) + "\n", "utf8");
}

async function updateCargoToml(filePath, nextVersion) {
  let content = await readFile(filePath, "utf8");

  const startIdx = content.indexOf("[package]");
  if (startIdx === -1) {
    throw new Error("[package] section not found in Cargo.toml");
  }
  const nextSectionIdx = content.indexOf("[", startIdx + 1);
  const endIdx = nextSectionIdx === -1 ? content.length : nextSectionIdx;
  const packageSection = content.slice(startIdx, endIdx);

  const newPackageSection = packageSection.replace(
    /^version\s*=\s*"[^"]+"/m,
    `version = "${nextVersion}"`
  );

  if (newPackageSection === packageSection) {
    throw new Error("version line not found in Cargo.toml [package] section");
  }

  content =
    content.slice(0, startIdx) + newPackageSection + content.slice(endIdx);
  await writeFile(filePath, content, "utf8");
}

async function main() {
  const root = process.cwd();
  const args = parseArgs(process.argv);

  const packageJsonPath = path.join(root, "package.json");
  const tauriConfPath = path.join(root, "src-tauri", "tauri.conf.json");
  const cargoTomlPath = path.join(root, "src-tauri", "Cargo.toml");

  const pkg = JSON.parse(await readFile(packageJsonPath, "utf8"));
  const current = pkg.version;
  if (!isValidSemver(current)) {
    throw new Error(`Invalid current version in package.json: ${current}`);
  }

  const nextVersion = args.to ? args.to : bumpSemver(current, args.mode);
  if (!isValidSemver(nextVersion)) {
    throw new Error(`Invalid target version: ${nextVersion}`);
  }

  // package.json
  await updateJsonFile(packageJsonPath, (data) => ({
    ...data,
    version: nextVersion,
  }));

  // src-tauri/tauri.conf.json
  await updateJsonFile(tauriConfPath, (data) => ({
    ...data,
    version: nextVersion,
  }));

  // src-tauri/Cargo.toml
  await updateCargoToml(cargoTomlPath, nextVersion);

  console.log(`Version updated: ${current} -> ${nextVersion}`);
  console.log("Updated files:");
  console.log(" - package.json");
  console.log(" - src-tauri/tauri.conf.json");
  console.log(" - src-tauri/Cargo.toml");
}

main().catch((err) => {
  console.error(err.message || err);
  process.exit(1);
});
