/**
 * Writes web/landing/public/version.json for local `vite` dev (same merge as embed-version).
 * Production builds still use scripts/embed-version.mjs → dist/version.json after vite build.
 */
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";
import { execFileSync } from "child_process";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.join(__dirname, "..", "..", "..");
const cargoPath = path.join(root, "crates", "tonet", "Cargo.toml");
const publicDir = path.join(__dirname, "..", "public");
const mergeScript = path.join(root, "scripts", "cdn-merge-manifest.mjs");

const cargo = fs.readFileSync(cargoPath, "utf8");
const m = cargo.match(/^version = "([^"]+)"/m);
const version = m ? m[1] : "0.0.0";
const cdnBase = (process.env.TONET_CDN_BASE_URL || "https://dl.usetonet.com").replace(/\/+$/, "");
const versionPath = process.env.TONET_CDN_VERSION_PATH || "/version.json";
const downloadsPage = process.env.TONET_SITE_DOWNLOADS_PAGE || `${cdnBase}/`;
const production = (process.env.TONET_RELEASE_PRODUCTION || "true").toLowerCase() !== "false";

fs.mkdirSync(publicDir, { recursive: true });
const existingManifestPath = path.join(publicDir, "version.json");
let prevPath = existingManifestPath;
let cleanupTemp = null;
if (!fs.existsSync(existingManifestPath)) {
  cleanupTemp = path.join(publicDir, ".embed-prev-manifest.json");
  fs.writeFileSync(
    cleanupTemp,
    JSON.stringify({ releases: [], repo: "usetonet/tonet-browser" }),
    "utf8"
  );
  prevPath = cleanupTemp;
}

const env = {
  ...process.env,
  CDN_BASE: cdnBase,
  VERSION: version,
  TONET_RELEASE_PRODUCTION: production ? "true" : "false",
  PREV_PATH: prevPath,
};

const merged = execFileSync(process.execPath, [mergeScript], {
  env,
  encoding: "utf8",
});

const payload = JSON.parse(merged);
payload.releasesUrl = downloadsPage;
payload.updateManifestUrl = `${cdnBase}${versionPath.startsWith("/") ? versionPath : `/${versionPath}`}`;

fs.writeFileSync(path.join(publicDir, "version.json"), JSON.stringify(payload, null, 2));
if (cleanupTemp) fs.rmSync(cleanupTemp, { force: true });
console.log("public/version.json →", payload.version, "(local dev)");
