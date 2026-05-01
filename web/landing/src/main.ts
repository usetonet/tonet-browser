import "./styles/global.css";
import { detectOS, type DetectedOS } from "./detect-os";
import {
  applyLandingLocale,
  detectedOsLine,
  resolveSiteLang,
  versionPillPrefix,
  wireCopyButtons,
  wireLanguageSwitcher,
} from "./site-i18n";

type VersionDownload = {
  windowsSetup: string;
  linuxSetup: string;
  macSetup: string;
  windowsMsi: string;
  windowsExe: string;
  windowsPortableZip: string;
  deb: string;
  macTarball: string;
};

type VersionMeta = {
  version: string;
  repo?: string;
  releasesUrl?: string;
  updateManifestUrl?: string;
  cdnBaseUrl?: string;
  download?: VersionDownload;
};

async function loadVersion(): Promise<VersionMeta | null> {
  try {
    const r = await fetch("/version.json", { cache: "no-store" });
    if (!r.ok) return null;
    return await r.json();
  } catch {
    return null;
  }
}

function setActiveOS(os: DetectedOS): void {
  const tabs = document.querySelectorAll<HTMLButtonElement>(".os-tab");
  const panels = document.querySelectorAll<HTMLElement>(".os-panel");

  const map: Record<DetectedOS, string> = {
    windows: "panel-windows",
    linux: "panel-linux",
    macos: "panel-macos",
    unknown: "panel-windows",
  };

  const panelId = map[os] ?? "panel-windows";

  tabs.forEach((t) => {
    const active = t.dataset.os === os || (os === "unknown" && t.dataset.os === "windows");
    t.classList.toggle("active", !!active);
    t.setAttribute("aria-selected", active ? "true" : "false");
  });

  panels.forEach((p) => {
    p.classList.toggle("active", p.id === panelId);
  });
}

async function main(): Promise<void> {
  const lang = resolveSiteLang();
  applyLandingLocale(lang);
  wireLanguageSwitcher(lang);

  const os = detectOS();
  const detectedEl = document.getElementById("os-detected");
  if (detectedEl) {
    detectedEl.textContent = detectedOsLine(lang, os);
    detectedEl.hidden = false;
  }

  setActiveOS(os === "unknown" ? "windows" : os);

  document.querySelectorAll<HTMLButtonElement>(".os-tab").forEach((tab) => {
    tab.addEventListener("click", () => {
      const v = tab.dataset.os as DetectedOS | undefined;
      if (v) setActiveOS(v);
    });
  });

  const meta = await loadVersion();
  const ver = meta?.version ?? "…";
  const pill = document.getElementById("version-pill");
  if (pill) pill.textContent = `${versionPillPrefix(lang)} ${ver}`;

  const releasesLatest = meta?.releasesUrl ?? "https://downloads.usetonet.com/";
  const winSetup = document.getElementById("win-setup") as HTMLAnchorElement | null;
  const linuxSetup = document.getElementById("linux-setup") as HTMLAnchorElement | null;
  const macSetup = document.getElementById("mac-setup") as HTMLAnchorElement | null;
  const d = meta?.download;
  if (winSetup) {
    winSetup.href = d?.windowsSetup ?? releasesLatest;
  }
  if (linuxSetup) {
    linuxSetup.href = d?.linuxSetup ?? releasesLatest;
  }
  if (macSetup) {
    macSetup.href = d?.macSetup ?? releasesLatest;
  }

  const base = meta?.cdnBaseUrl ?? "https://downloads.usetonet.com";
  const winMsi = document.getElementById("win-msi") as HTMLAnchorElement | null;
  const winExe = document.getElementById("win-exe") as HTMLAnchorElement | null;
  const linuxDeb = document.getElementById("linux-deb") as HTMLAnchorElement | null;
  if (meta?.version && ver !== "…" && winMsi && winExe && linuxDeb) {
    winMsi.href = `${base}/Tonet-${ver}-x64.msi`;
    winExe.href = `${base}/Tonet-Setup-${ver}-x64.exe`;
    linuxDeb.href = `${base}/tonet_${ver}_amd64.deb`;
  }

  wireCopyButtons(lang);
}

main();
