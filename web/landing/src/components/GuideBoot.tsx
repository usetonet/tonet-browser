import { useEffect } from "react";
import { applyGuidePageLocale } from "../i18n/extra-pages";
import { applyLandingLocale, resolveSiteLang, wireLanguageSwitcher } from "../site-i18n";

export function GuideBoot(): null {
  useEffect(() => {
    const lang = resolveSiteLang();
    applyLandingLocale(lang, { page: "guide" });
    applyGuidePageLocale(lang);
    wireLanguageSwitcher(lang);
  }, []);
  return null;
}
