import "./styles/global.css";
import { mountSiteNav } from "./mount-nav";
import { detectOS, type DetectedOS } from "./detect-os";
import {
  applyLandingLocale,
  detectedOsLine,
  resolveSiteLang,
  versionPillPrefix,
  wireCopyButtons,
  wireLanguageSwitcher,
} from "./site-i18n";

mountSiteNav();

type DownloadMap = {
  windowsSetup: string;
  linuxSetup: string;
  macSetup: string;
  windowsMsi: string;
  windowsExe: string;
  windowsPortableZip: string;
  deb: string;
  macTarball: string;
};

type ReleaseEntry = {
  version: string;
  production: boolean;
  download: DownloadMap;
};

type ChannelInfo = {
  version: string;
  production?: boolean;
  recommended?: boolean;
  download: DownloadMap;
};

type VersionMeta = {
  schemaVersion?: number;
  version: string;
  repo?: string;
  releasesUrl?: string;
  updateManifestUrl?: string;
  cdnBaseUrl?: string;
  download?: DownloadMap;
  channels?: {
    stable?: ChannelInfo;
    development?: ChannelInfo;
  };
  releases?: ReleaseEntry[];
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

  panels.for((p) => {
    p.classList.toggle("active", p.id === panelId);
  });
}

function applyDownloadLinks(d: DownloadMap | undefined, fallback: string): void {
  const winSetup = document.getElementById("win-setup") as HTMLAnchorElement | null;
  const linuxSetup = document.getElementById("linux-setup") as HTMLAnchorElement | null;
  const macSetup = document.getElementById("mac-setup") as HTMLAnchorElement | null;
  const winMsi = document.getElementById("win-msi") as HTMLAnchorElement | null;
  const winExe = document.getElementById("win-exe") as HTMLAnchorElement | null;
  const linuxDeb = document.getElementById("linux-deb") as HTMLAnchorElement | null;

  if (!d) {
    if (winSetup) winSetup.href = fallback;
    if (linuxSetup) linuxSetup.href = fallback;
    if (macSetup) macSetup.href = fallback;
    if (winMsi) winMsi.href = fallback;
    if (winExe) winExe.href = fallback;
    if (linuxDeb) linuxDeb.href = fallback;
    return;
  }

  if (winSetup) winSetup.href = d.windowsSetup;
  if (linuxSetup) linuxSetup.href = d.linuxSetup;
  if (macSetup) macSetup.href = d.macSetup;
  if (winMsi) winMsi.href = d.windowsMsi;
  if (winExe) winExe.href = d.windowsExe;
  if (linuxDeb) linuxDeb.href = d.deb;
}

function stableVersionLabel(meta: VersionMeta): string {
  return meta.channels?.stable?.version ?? meta.version;
}

/** Primary installer URL for stable channel (short CDN names where applicable). */
function primaryInstallerUrl(os: DetectedOS, meta: VersionMeta): string | null {
  const d = meta.channels?.stable?.download ?? meta.download;
  if (!d) return null;
  const effective = os === "unknown" ? "windows" : os;
  if (effective === "windows") return d.windowsSetup ?? null;
  if (effective === "linux") return d.linuxSetup ?? null;
  if (effective === "macos") return d.macSetup ?? null;
  return null;
}

function configureHeroDownload(meta: VersionMeta | null, os: DetectedOS): void {
  const hero = document.getElementById("hero-download") as HTMLAnchorElement | null;
  if (!hero) return;
  if (!meta) {
    hero.href = "#download";
    return;
  }
  const url = primaryInstallerUrl(os, meta);
  if (url) {
    hero.href = url;
    return;
  }
  hero.href = "#download";
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
  configureHeroDownload(meta, os);

  const releasesFallback = meta?.releasesUrl ?? "https://dl.usetonet.com/";
  const pill = document.getElementById("version-pill");
  if (pill) {
    pill.textContent = meta
      ? `${versionPillPrefix(lang)} ${stableVersionLabel(meta)}`
      : `${versionPillPrefix(lang)} …`;
  }

  const channelSelect = document.getElementById("channel-select") as HTMLSelectElement | null;
  const versionRow = document.getElementById("version-row");
  const versionSelect = document.getElementById("version-select") as HTMLSelectElement | null;
  const channelHint = document.getElementById("channel-hint");

  const releases = meta?.releases?.length ? meta.releases : [];

  if (versionSelect && releases.length > 0) {
    versionSelect.innerHTML = "";
    for (const r of releases) {
      const opt = document.createElement("option");
      opt.value = r.version;
      opt.textContent = `${r.version}${r.production ? " (stable)" : " (preview)"}`;
      versionSelect.appendChild(opt);
    }
  }

  const devAvailable = !!(meta?.channels?.development?.download);

  if (channelSelect) {
    const devOpt = channelSelect.querySelector<HTMLOptionElement>('option[value="development"]');
    if (devOpt) {
      devOpt.disabled = !devAvailable;
    }
  }

  function refreshDownloads(): void {
    if (!meta) return;
    const mode = channelSelect?.value ?? "stable";

    if (mode === "specific") {
      versionRow?.removeAttribute("hidden");
      const v = versionSelect?.value;
      const entry = releases.find((r) => r.version === v);
      applyDownloadLinks(entry?.download, releasesFallback);
      if (channelHint) {
        channelHint.textContent =
          "Versioned filenames on the CDN (e.g. Tonet-Setup-x.y.z-x64.exe). Pick the build you need.";
      }
      return;
    }

    versionRow?.setAttribute("hidden", "true");

    if (mode === "development") {
      const d = meta.channels?.development?.download;
      applyDownloadLinks(d, releasesFallback);
      if (channelHint) {
        channelHint.textContent = devAvailable
          ? "Preview channel: may include unstable changes. Short filenames (e.g. Tonet-Setup-Preview.exe) track the latest preview."
          : "No preview release is published on the CDN yet.";
      }
      return;
    }

    const d = meta.channels?.stable?.download ?? meta.download;
    applyDownloadLinks(d, releasesFallback);
    if (channelHint) {
      channelHint.textContent =
        "Recommended production builds. Short filenames (Tonet-Setup.exe, tonet_amd64.deb, …) always point at the latest stable release.";
    }
  }

  channelSelect?.addEventListener("change", refreshDownloads);
  versionSelect?.addEventListener("change", refreshDownloads);

  if (meta) {
    refreshDownloads();
  } else {
    applyDownloadLinks(undefined, releasesFallback);
  }

  wireCopyButtons(lang);
}

main();
