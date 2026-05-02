import { useEffect } from "react";
import { applyRoadmapLocale } from "../i18n/extra-pages";
import { applyFooterLocale, resolveSiteLang, wireLanguageSwitcher } from "../site-i18n";

export function RoadmapBoot(): null {
  useEffect(() => {
    const lang = resolveSiteLang();
    applyRoadmapLocale(lang);
    applyFooterLocale(lang);
    wireLanguageSwitcher(lang);
  }, []);
  return null;
}
