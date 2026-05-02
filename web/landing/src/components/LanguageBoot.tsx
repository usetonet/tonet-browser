import { useEffect } from "react";
import { resolveSiteLang, wireLanguageSwitcher } from "../site-i18n";

/** Footer language switcher only (compare, roadmap, handbook). */
export function LanguageBoot(): null {
  useEffect(() => {
    wireLanguageSwitcher(resolveSiteLang());
  }, []);
  return null;
}
