import { useEffect } from "react";
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
    wireLanguageSwitcher(lang);
    wireCopyButtons(lang);
  }, []);
  return null;
}
