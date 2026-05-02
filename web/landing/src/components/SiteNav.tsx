import { useEffect, useRef, useState } from "react";
import { FaGithub } from "react-icons/fa";
import { IoChevronDown } from "react-icons/io5";
import type { SitePage } from "../types/site-page";
import { getNavLabels, resolveSiteLang, type SiteLang } from "../site-i18n";

const GITHUB_REPO_API = "https://api.github.com/repos/usetonet/tonet-browser";

type Props = {
  current: SitePage;
};

type StarState = "loading" | { stars: number } | "fallback";

function navCls(active: boolean): string {
  return active
    ? "font-semibold text-[#e8ecf4] no-underline hover:text-[#e8ecf4]"
    : "text-[0.95rem] text-[#9aa3b5] no-underline hover:text-[#e8ecf4]";
}

function formatStarCount(n: number): string {
  if (n >= 1000) {
    const k = n / 1000;
    if (k >= 10) return `${Math.round(k)}k`;
    return `${k.toFixed(1).replace(/\.0$/, "")}k`;
  }
  return String(n);
}

export default function SiteNav({ current }: Props) {
  const lang = resolveSiteLang() as SiteLang;
  const n = getNavLabels(lang);
  const [open, setOpen] = useState(false);
  const [starState, setStarState] = useState<StarState>("loading");
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

  useEffect(() => {
    let cancelled = false;
    setStarState("loading");
    void fetch(GITHUB_REPO_API, {
      headers: { Accept: "application/vnd.github+json" },
    })
      .then((r) => (r.ok ? r.json() : Promise.reject(new Error("github api"))))
      .then((data: { stargazers_count?: unknown }) => {
        if (cancelled) return;
        const c = data.stargazers_count;
        const nStars = typeof c === "number" && Number.isFinite(c) ? c : 0;
        if (nStars >= 1) {
          setStarState({ stars: nStars });
        } else {
          setStarState("fallback");
        }
      })
      .catch(() => {
        if (!cancelled) setStarState("fallback");
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const repoHref = "https://github.com/usetonet/tonet-browser";
  const resolvedStars =
    starState !== "loading" && starState !== "fallback" ? starState.stars : null;
  const hasStars = resolvedStars !== null && resolvedStars >= 1;
  const ariaForLink =
    starState === "loading"
      ? "Tonet on GitHub — loading star count"
      : hasStars && resolvedStars !== null
        ? `Tonet on GitHub — ${resolvedStars} stars`
        : `Tonet on GitHub — ${n.github}`;

  return (
    <header className="sticky top-0 z-50 border-b border-white/[0.08] bg-[#070a10]/72 backdrop-blur-md">
      <div className="mx-auto flex max-w-[1120px] flex-wrap items-center justify-between gap-4 px-6 py-3.5">
        <a className="flex items-center gap-2.5 text-lg font-bold text-[#e8ecf4] no-underline hover:opacity-90" href="/">
          <span
            className="size-8 shrink-0 rounded-[9px] bg-[url('/tonet.svg')] bg-contain bg-center bg-no-repeat shadow-[0_8px_28px_rgba(90,153,255,0.35)]"
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
            className="inline-flex min-h-[28px] items-center gap-2 text-[0.95rem] text-[#9aa3b5] no-underline hover:text-[#e8ecf4]"
            href={repoHref}
            target="_blank"
            rel="noopener noreferrer"
            aria-label={ariaForLink}
          >
            <FaGithub aria-hidden size={20} className="shrink-0 text-[#e8ecf4]/90" />
            <span className="inline-flex min-w-[4.25rem] items-center gap-1 sm:min-w-[4.75rem]" aria-live="polite">
              {starState === "loading" ? (
                <span
                  className="inline-block h-[1.05rem] w-[3.25rem] rounded bg-white/[0.08] motion-safe:animate-pulse sm:w-[3.75rem]"
                  aria-hidden
                />
              ) : hasStars && resolvedStars !== null ? (
                <>
                  <span className="text-[0.85rem] text-[#e6c35c]" aria-hidden>
                    ★
                  </span>
                  <span className="font-semibold tabular-nums tracking-tight text-[#e8ecf4]">
                    {formatStarCount(resolvedStars)}
                  </span>
                </>
              ) : (
                <span className="font-medium text-[#9aa3b5]">{n.github}</span>
              )}
            </span>
          </a>
        </nav>
      </div>
    </header>
  );
}
