import { useEffect } from "react";
import { applyDocsExtraLocale } from "../i18n/extra-pages";
import {
  applyDocsLocale,
  resolveSiteLang,
  wireCopyButtons,
  wireLanguageSwitcher,
} from "../site-i18n";

export function DocsBoot(): null {
  useEffect(() => {
    const lang = resolveSiteLang();
    applyDocsLocale(lang);
    applyDocsExtraLocale(lang);
    wireLanguageSwitcher(lang);
    wireCopyButtons(lang);
  }, []);
  return null;
}
