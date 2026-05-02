import { useEffect } from "react";
import { applyFooterLocale, resolveSiteLang, wireLanguageSwitcher } from "../site-i18n";

/** Footer + language select without overwriting page title/meta (legal / about pages). */
export function SiteChromeBoot(): null {
  useEffect(() => {
    const lang = resolveSiteLang();
    document.documentElement.lang = lang;
    applyFooterLocale(lang);
    wireLanguageSwitcher(lang);
  }, []);
  return null;
}
