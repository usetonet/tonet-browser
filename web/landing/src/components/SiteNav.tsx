import { useEffect, useRef, useState } from "react";
import { FaGithub } from "react-icons/fa";
import { IoChevronDown } from "react-icons/io5";
import type { SitePage } from "../types/site-page";
import { getNavLabels, resolveSiteLang, type SiteLang } from "../site-i18n";

type Props = {
  current: SitePage;
};

function navCls(active: boolean): string {
  return active
    ? "font-semibold text-[#e8ecf4] no-underline hover:text-[#e8ecf4]"
    : "text-[0.95rem] text-[#9aa3b5] no-underline hover:text-[#e8ecf4]";
}

export default function SiteNav({ current }: Props) {
  const lang = resolveSiteLang() as SiteLang;
  const n = getNavLabels(lang);
  const [open, setOpen] = useState(false);
  const wrapRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function onDocClick(e: MouseEvent) {
      if (!wrapRef.current?.contains(e.target as Node)) setOpen(false);
    }
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") setOpen(false);
    }
    document.addEventListener("click", onDocClick);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("click", onDocClick);
      document.removeEventListener("keydown", onKey);
    };
  }, []);

  return (
    <header className="sticky top-0 z-50 border-b border-white/[0.08] bg-[#070a10]/72 backdrop-blur-md">
      <div className="mx-auto flex max-w-[1120px] flex-wrap items-center justify-between gap-4 px-6 py-3.5">
        <a className="flex items-center gap-2.5 text-lg font-bold text-[#e8ecf4] no-underline hover:opacity-90" href="/">
          <span
            className="size-8 shrink-0 rounded-[9px] bg-[url('/tonet.svg')] bg-contain bg-center bg-no-repeat shadow-[0_8px_28px_rgba(79,140,255,0.35)]"
            aria-hidden
          />
          <span>Tonet</span>
        </a>
        <nav className="flex flex-wrap items-center gap-5 md:gap-6" id="site-nav-links" aria-label={n.ariaMain}>
          <a className={navCls(false)} href="/download.html">
            {n.download}
          </a>
          <a className={navCls(current === "roadmap")} href="/roadmap.html">
            {n.roadmap}
          </a>
          <div className={`relative ${open ? "" : ""}`} ref={wrapRef}>
            <button
              type="button"
              className={`inline-flex cursor-pointer items-center gap-1 border-0 bg-transparent p-0 font-[inherit] ${navCls(false)}`}
              aria-expanded={open}
              aria-haspopup="true"
              onClick={(e) => {
                e.stopPropagation();
                setOpen((v) => !v);
              }}
            >
              {n.more}
              <IoChevronDown className={`shrink-0 transition-transform ${open ? "rotate-180" : ""}`} aria-hidden size={18} />
            </button>
            {open ? (
              <div
                className="absolute right-0 top-[calc(100%+10px)] z-[80] min-w-[240px] rounded-[14px] border border-white/[0.08] bg-[#0d111c]/96 p-3 pb-3.5 shadow-[0_18px_48px_rgba(0,0,0,0.45)] backdrop-blur-md"
                role="menu"
              >
                <div className="mb-2 ml-1 text-[0.7rem] uppercase tracking-[0.08em] text-[#9aa3b5] opacity-85">
                  {n.dropdownExplore}
                </div>
                <a className={`block rounded-lg px-1.5 py-2 ${navCls(false)}`} role="menuitem" href="/#features" onClick={() => setOpen(false)}>
                  {n.features}
                </a>
                <a
                  className={`block rounded-lg px-1.5 py-2 ${navCls(current === "guide")}`}
                  role="menuitem"
                  href="/guide.html"
                  onClick={() => setOpen(false)}
                >
                  {n.guide}
                </a>
                <a
                  className={`block rounded-lg px-1.5 py-2 ${navCls(current === "handbook")}`}
                  role="menuitem"
                  href="/handbook.html"
                  onClick={() => setOpen(false)}
                >
                  {n.handbook}
                </a>
                <a
                  className={`block rounded-lg px-1.5 py-2 ${navCls(current === "docs")}`}
                  role="menuitem"
                  href="/docs.html"
                  onClick={() => setOpen(false)}
                >
                  {n.technicalDocs}
                </a>
                <a
                  className={`block rounded-lg px-1.5 py-2 ${navCls(current === "compare")}`}
                  role="menuitem"
                  href="/compare.html"
                  onClick={() => setOpen(false)}
                >
                  {n.compare}
                </a>
              </div>
            ) : null}
          </div>
          <a
            className="inline-flex items-center gap-1.5 text-[0.95rem] text-[#9aa3b5] no-underline hover:text-[#e8ecf4] md:inline-flex"
            href="https://github.com/usetonet/tonet-browser"
            target="_blank"
            rel="noopener noreferrer"
          >
            <FaGithub aria-hidden title="GitHub" size={20} />
            <span className="hidden md:inline">GitHub</span>
          </a>
        </nav>
      </div>
    </header>
  );
}
