import "./styles/global.css";
import { mountSiteNav } from "./mount-nav";
import {
  applyDocsLocale,
  resolveSiteLang,
  wireCopyButtons,
  wireLanguageSwitcher,
} from "./site-i18n";

mountSiteNav();

const lang = resolveSiteLang();
applyDocsLocale(lang);
wireLanguageSwitcher(lang);
wireCopyButtons(lang);
