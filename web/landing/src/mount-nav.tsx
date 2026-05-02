import { createRoot } from "react-dom/client";
import { SiteNav, type SitePage } from "./SiteNav";

function readPage(): SitePage {
  const d = document.body?.dataset.page;
  if (d === "docs" || d === "compare" || d === "roadmap" || d === "handbook") return d;
  return "home";
}

export function mountSiteNav(): void {
  const el = document.getElementById("site-nav-root");
  if (!el) return;
  const root = createRoot(el);
  root.render(<SiteNav current={readPage()} />);
}
