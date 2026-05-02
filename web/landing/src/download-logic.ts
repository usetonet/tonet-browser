import type { DetectedOS } from "./detect-os";

export type DownloadMap = {
  windowsSetup: string;
  linuxSetup: string;
  macSetup: string;
  windowsMsi: string;
  windowsExe: string;
  windowsPortableZip: string;
  deb: string;
  macTarball: string;
};

export type ReleaseEntry = {
  version: string;
  production: boolean;
  download: DownloadMap;
};

export type ChannelInfo = {
  version: string;
  production?: boolean;
  recommended?: boolean;
  download: DownloadMap;
};

export type VersionMeta = {
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

export async function loadVersionMeta(): Promise<VersionMeta | null> {
  try {
    const r = await fetch("/version.json", { cache: "no-store" });
    if (!r.ok) return null;
    return await r.json();
  } catch {
    return null;
  }
}

export function setActiveOS(os: DetectedOS): void {
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

export function applyDownloadLinks(d: DownloadMap | undefined, fallback: string): void {
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

export function stableVersionLabel(meta: VersionMeta): string {
  return meta.channels?.stable?.version ?? meta.version;
}
