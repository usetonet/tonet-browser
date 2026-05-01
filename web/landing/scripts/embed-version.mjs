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
const cdnBase = (process.env.TONET_CDN_BASE_URL || "https://downloads.usetonet.com").replace(/\/+$/, "");
const versionPath = process.env.TONET_CDN_VERSION_PATH || "/version.json";
const downloadsPage = process.env.TONET_SITE_DOWNLOADS_PAGE || `${cdnBase}/`;

const payload = {
  version,
  repo: "usetonet/tonet-browser",
  releasesUrl: downloadsPage,
  updateManifestUrl: `${cdnBase}${versionPath.startsWith("/") ? versionPath : `/${versionPath}`}`,
  cdnBaseUrl: cdnBase,
  download: {
    windowsSetup: `${cdnBase}/Tonet-Setup-${version}-x64.exe`,
    linuxSetup: `${cdnBase}/tonet_${version}_amd64.deb`,
    macSetup: `${cdnBase}/Tonet-${version}-macos.tar.gz`,
    windowsMsi: `${cdnBase}/Tonet-${version}-x64.msi`,
    windowsExe: `${cdnBase}/Tonet-Setup-${version}-x64.exe`,
    windowsPortableZip: `${cdnBase}/Tonet-${version}-windows-x64-portable.zip`,
    deb: `${cdnBase}/tonet_${version}_amd64.deb`,
    macTarball: `${cdnBase}/Tonet-${version}-macos.tar.gz`,
  },
};

fs.mkdirSync(dist, { recursive: true });
fs.writeFileSync(path.join(dist, "version.json"), JSON.stringify(payload, null, 2));
console.log("version.json →", payload.version);
