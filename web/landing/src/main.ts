import "./styles/global.css";
import {
  applyLandingLocale,
  resolveSiteLang,
  versionPillPrefix,
  wireLanguageSwitcher,
} from "./site-i18n";
import { loadVersionMeta, stableVersionLabel } from "./download-logic";

async function main(): Promise<void> {
  const lang = resolveSiteLang();
  applyLandingLocale(lang);
  wireLanguageSwitcher(lang);

  const meta = await loadVersionMeta();
  const pill = document.getElementById("version-pill");
  if (pill) {
    pill.textContent = meta
      ? `${versionPillPrefix(lang)} ${stableVersionLabel(meta)}`
      : `${versionPillPrefix(lang)} …`;
  }
}

main();
