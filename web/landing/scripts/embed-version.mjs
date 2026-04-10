import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.join(__dirname, "..", "..", "..");
const cargoPath = path.join(root, "crates", "tonet", "Cargo.toml");
const dist = path.join(__dirname, "..", "dist");

const cargo = fs.readFileSync(cargoPath, "utf8");
const m = cargo.match(/^version = "([^"]+)"/m);
const version = m ? m[1] : "0.0.0";

const payload = {
  version,
  repo: "usetonet/tonet-browser",
  releasesUrl: "https://github.com/usetonet/tonet-browser/releases/latest",
  download: {
    windowsMsi: `https://github.com/usetonet/tonet-browser/releases/latest/download/Tonet-${version}-x64.msi`,
    windowsExe: `https://github.com/usetonet/tonet-browser/releases/latest/download/Tonet-Setup-${version}-x64.exe`,
    deb: `https://github.com/usetonet/tonet-browser/releases/latest/download/tonet_${version}_amd64.deb`,
  },
};

fs.mkdirSync(dist, { recursive: true });
fs.writeFileSync(path.join(dist, "version.json"), JSON.stringify(payload, null, 2));
console.log("version.json →", payload.version);
