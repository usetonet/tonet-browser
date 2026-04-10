import "./styles/global.css";
import {
  applyDocsLocale,
  resolveSiteLang,
  wireCopyButtons,
  wireLanguageSwitcher,
} from "./site-i18n";

const lang = resolveSiteLang();
applyDocsLocale(lang);
wireLanguageSwitcher(lang);
wireCopyButtons(lang);
