import "./styles/global.css";
import { applyLandingLocale, resolveSiteLang, wireLanguageSwitcher } from "./site-i18n";

const lang = resolveSiteLang();
applyLandingLocale(lang, { page: "guide" });
wireLanguageSwitcher(lang);
