/**
 * Merge CDN version.json with a new release entry.
 * Env: CDN_BASE, VERSION, TONET_RELEASE_PRODUCTION (true|false), PREV_PATH (default prev.json)
 * Writes merged manifest to stdout.
 */
import fs from "fs";

const CDN_BASE = (process.env.CDN_BASE || "").replace(/\/+$/, "");
const VERSION = (process.env.VERSION || "").trim();
const PRODUCTION =
  (process.env.TONET_RELEASE_PRODUCTION || "true").toLowerCase() !== "false";
const PREV_PATH = process.env.PREV_PATH || "prev.json";

if (!CDN_BASE || !VERSION) {
  console.error("cdn-merge-manifest: CDN_BASE and VERSION are required");
  process.exit(1);
}

/** Per-file URLs including version in filename (archival / picker). */
function versionedDownloads(v) {
  const base = CDN_BASE;
  return {
    windowsSetup: `${base}/Tonet-Setup-${v}-x64.exe`,
    linuxSetup: `${base}/tonet_${v}_amd64.deb`,
    macSetup: `${base}/Tonet-${v}-macos.tar.gz`,
    windowsMsi: `${base}/Tonet-${v}-x64.msi`,
    windowsExe: `${base}/Tonet-Setup-${v}-x64.exe`,
    windowsPortableZip: `${base}/Tonet-${v}-windows-x64-portable.zip`,
    deb: `${base}/tonet_${v}_amd64.deb`,
    macTarball: `${base}/Tonet-${v}-macos.tar.gz`,
  };
}

/** Stable channel: fixed filenames overwritten each stable production release. */
function stableAliasDownloads() {
  const base = CDN_BASE;
  return {
    windowsSetup: `${base}/Tonet-Setup.exe`,
    linuxSetup: `${base}/tonet_amd64.deb`,
    macSetup: `${base}/Tonet-macos.tar.gz`,
    windowsMsi: `${base}/Tonet-x64.msi`,
    windowsExe: `${base}/Tonet-Setup.exe`,
    windowsPortableZip: `${base}/Tonet-windows-x64-portable.zip`,
    deb: `${base}/tonet_amd64.deb`,
    macTarball: `${base}/Tonet-macos.tar.gz`,
  };
}

/** Development / preview channel: separate aliases (non-production builds). */
function developmentAliasDownloads() {
  const base = CDN_BASE;
  return {
    windowsSetup: `${base}/Tonet-Setup-Preview.exe`,
    linuxSetup: `${base}/tonet_amd64-preview.deb`,
    macSetup: `${base}/Tonet-macos-preview.tar.gz`,
    windowsMsi: `${base}/Tonet-x64-preview.msi`,
    windowsExe: `${base}/Tonet-Setup-Preview.exe`,
    windowsPortableZip: `${base}/Tonet-windows-x64-portable-preview.zip`,
    deb: `${base}/tonet_amd64-preview.deb`,
    macTarball: `${base}/Tonet-macos-preview.tar.gz`,
  };
}

function semverCompare(a, b) {
  const pa = a
    .replace(/^v/, "")
    .split(/[.+_-]/)
    .map((x) => {
      const n = parseInt(x, 10);
      return Number.isFinite(n) ? n : x;
    });
  const pb = b
    .replace(/^v/, "")
    .split(/[.+_-]/)
    .map((x) => {
      const n = parseInt(x, 10);
      return Number.isFinite(n) ? n : x;
    });
  const len = Math.max(pa.length, pb.length);
  for (let i = 0; i < len; i++) {
    const x = pa[i] ?? 0;
    const y = pb[i] ?? 0;
    if (typeof x === typeof y) {
      if (x < y) return -1;
      if (x > y) return 1;
    } else {
      const sx = String(x);
      const sy = String(y);
      if (sx < sy) return -1;
      if (sx > sy) return 1;
    }
  }
  return 0;
}

function sortReleasesDesc(releases) {
  return [...releases].sort((r1, r2) => semverCompare(r2.version, r1.version));
}

let prev = {};
try {
  const raw = fs.readFileSync(PREV_PATH, "utf8");
  prev = JSON.parse(raw || "{}");
} catch {
  prev = {};
}

let releases = Array.isArray(prev.releases) ? prev.releases : [];

const entry = {
  version: VERSION,
  production: PRODUCTION,
  download: versionedDownloads(VERSION),
};

const idx = releases.findIndex((r) => r.version === VERSION);
if (idx >= 0) releases[idx] = entry;
else releases.push(entry);

releases = sortReleasesDesc(releases);

const prodReleases = releases.filter((r) => r.production);
const devReleases = releases.filter((r) => !r.production);

const latestStable = prodReleases[0] ?? null;
const latestDev = devReleases[0] ?? null;

const channels = {};

if (latestStable) {
  channels.stable = {
    version: latestStable.version,
    production: true,
    recommended: true,
    download: stableAliasDownloads(),
  };
}

if (latestDev) {
  channels.development = {
    version: latestDev.version,
    production: false,
    recommended: false,
    download: developmentAliasDownloads(),
  };
}

const topVersion = latestStable?.version ?? VERSION;

const out = {
  schemaVersion: 2,
  version: topVersion,
  repo: prev.repo ?? "usetonet/tonet-browser",
  releasesUrl: `${CDN_BASE}/`,
  cdnBaseUrl: CDN_BASE,
  updateManifestUrl: `${CDN_BASE}/version.json`,
  download:
    channels.stable?.download ?? versionedDownloads(VERSION),
  channels,
  releases,
};

process.stdout.write(JSON.stringify(out, null, 2));
