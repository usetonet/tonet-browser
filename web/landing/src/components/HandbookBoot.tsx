import { useEffect } from "react";
import { applyHandbookLocale } from "../i18n/extra-pages";
import { resolveSiteLang, wireLanguageSwitcher } from "../site-i18n";

export function HandbookBoot(): null {
  useEffect(() => {
    const lang = resolveSiteLang();
    applyHandbookLocale(lang);
    wireLanguageSwitcher(lang);
  }, []);
  return null;
}
