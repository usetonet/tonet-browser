import { useEffect } from "react";
import {
  applyDownloadLinks,
  loadVersionMeta,
  setActiveOS,
  stableVersionLabel,
} from "../download-logic";
import {
  applyLandingLocale,
  detectedOsLine,
  resolveSiteLang,
  versionPillPrefix,
  wireCopyButtons,
  wireLanguageSwitcher,
} from "../site-i18n";
import type { DetectedOS } from "../detect-os";

function syncPrimaryCta(): void {
  const primary = document.getElementById("download-primary") as HTMLAnchorElement | null;
  if (!primary) return;
  const activeTab = document.querySelector<HTMLButtonElement>(".os-tab.active");
  const os = (activeTab?.dataset.os as DetectedOS | undefined) ?? "windows";
  const srcId = os === "linux" ? "linux-setup" : os === "macos" ? "mac-setup" : "win-setup";
  const src = document.getElementById(srcId) as HTMLAnchorElement | null;
  if (src?.href) primary.href = src.href;
}

function triggerFileDownload(url: string): void {
  const a = document.createElement("a");
  a.href = url;
  a.rel = "noopener";
  a.target = "_blank";
  document.body.appendChild(a);
  a.click();
  a.remove();
}

function openDownloadModal(): void {
  document.getElementById("download-modal")?.removeAttribute("hidden");
}

function closeDownloadModal(): void {
  document.getElementById("download-modal")?.setAttribute("hidden", "true");
}

function wireModal(): void {
  const modal = document.getElementById("download-modal");
  const primary = document.getElementById("download-primary") as HTMLAnchorElement | null;
  const retry = document.getElementById("modal-retry-link") as HTMLAnchorElement | null;
  const closeBtn = document.querySelector<HTMLButtonElement>(".modal-close");

  primary?.addEventListener("click", (e) => {
    e.preventDefault();
    const url = primary.href;
    if (url) triggerFileDownload(url);
    openDownloadModal();
  });

  retry?.addEventListener("click", (e) => {
    e.preventDefault();
    const url = primary?.href;
    if (url) triggerFileDownload(url);
  });

  closeBtn?.addEventListener("click", () => closeDownloadModal());
  modal?.addEventListener("click", (e) => {
    if (e.target === modal) closeDownloadModal();
  });
  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape") closeDownloadModal();
  });
}

export function DownloadBoot(): null {
  useEffect(() => {
    const lang = resolveSiteLang();
    applyLandingLocale(lang, { page: "download" });
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
        if (v) {
          setActiveOS(v);
          syncPrimaryCta();
        }
      });
    });

    let cancelled = false;
    void loadVersionMeta().then((meta) => {
      if (cancelled) return;
      const releasesFallback = meta?.releasesUrl ?? "https://dl.usetonet.com/";
      const pill = document.getElementById("version-pill");
      if (pill) {
        pill.textContent = meta
          ? `${versionPillPrefix(lang)} ${stableVersionLabel(meta)}`
          : `${versionPillPrefix(lang)} …`;
      }
      if (!meta) {
        applyDownloadLinks(undefined, releasesFallback);
        syncPrimaryCta();
        wireCopyButtons(lang);
        wireModal();
        return;
      }

      const channelSelect = document.getElementById("channel-select") as HTMLSelectElement | null;
      const versionRow = document.getElementById("version-row");
      const versionSelect = document.getElementById("version-select") as HTMLSelectElement | null;
      const channelHint = document.getElementById("channel-hint");

      const releases = meta.releases?.length ? meta.releases : [];

      if (versionSelect && releases.length > 0) {
        versionSelect.innerHTML = "";
        for (const r of releases) {
          const opt = document.createElement("option");
          opt.value = r.version;
          opt.textContent = `${r.version}${r.production ? " (stable)" : " (preview)"}`;
          versionSelect.appendChild(opt);
        }
      }

      const devAvailable = !!(meta.channels?.development?.download);

      if (channelSelect) {
        const devOpt = channelSelect.querySelector<HTMLOptionElement>('option[value="development"]');
        if (devOpt) devOpt.disabled = !devAvailable;
      }

      function refreshDownloads(): void {
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
          syncPrimaryCta();
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
          syncPrimaryCta();
          return;
        }

        const d = meta.channels?.stable?.download ?? meta.download;
        applyDownloadLinks(d, releasesFallback);
        if (channelHint) {
          channelHint.textContent =
            "Recommended production builds. Short filenames (Tonet-Setup.exe, tonet_amd64.deb, …) always point at the latest stable release.";
        }
        syncPrimaryCta();
      }

      channelSelect?.addEventListener("change", refreshDownloads);
      versionSelect?.addEventListener("change", refreshDownloads);

      refreshDownloads();
      wireCopyButtons(lang);
      wireModal();
    });

    return () => {
      cancelled = true;
    };
  }, []);

  return null;
}
