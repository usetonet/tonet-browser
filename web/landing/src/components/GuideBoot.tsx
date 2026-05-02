import { useEffect } from "react";
import { applyLandingLocale, resolveSiteLang, wireLanguageSwitcher } from "../site-i18n";

export function GuideBoot(): null {
  useEffect(() => {
    const lang = resolveSiteLang();
    applyLandingLocale(lang, { page: "guide" });
    wireLanguageSwitcher(lang);
  }, []);
  return null;
}
