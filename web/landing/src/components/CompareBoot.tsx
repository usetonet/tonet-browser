import { useEffect } from "react";
import { applyComparePageLocale } from "../i18n/extra-pages";
import { resolveSiteLang, wireLanguageSwitcher } from "../site-i18n";

export function CompareBoot(): null {
  useEffect(() => {
    const lang = resolveSiteLang();
    applyComparePageLocale(lang);
    wireLanguageSwitcher(lang);
  }, []);
  return null;
}
