import { useEffect } from "react";
import { detectOS, type DetectedOS } from "../detect-os";
import { loadVersionMeta, stableVersionLabel, type VersionMeta } from "../download-logic";
import {
  applyLandingLocale,
  resolveSiteLang,
  versionPillPrefix,
  wireLanguageSwitcher,
} from "../site-i18n";

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
    hero.href = "/download.html";
    return;
  }
  const url = primaryInstallerUrl(os, meta);
  hero.href = url ?? "/download.html";
}

export function HomeBoot(): null {
  useEffect(() => {
    const lang = resolveSiteLang();
    applyLandingLocale(lang);
    wireLanguageSwitcher(lang);

    const os = detectOS();
    void loadVersionMeta().then((meta) => {
      configureHeroDownload(meta, os);
      const pill = document.getElementById("version-pill");
      if (pill) {
        pill.textContent = meta
          ? `${versionPillPrefix(lang)} ${stableVersionLabel(meta)}`
          : `${versionPillPrefix(lang)} …`;
      }
    });
  }, []);
  return null;
}
