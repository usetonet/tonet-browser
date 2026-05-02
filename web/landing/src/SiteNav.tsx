import { FaGithub } from "react-icons/fa";

export type SitePage = "home" | "docs" | "compare" | "roadmap" | "handbook";

type Props = {
  current: SitePage;
};

function navCls(active: boolean): string {
  return active ? "nav-link nav-link-active" : "nav-link";
}

export function SiteNav({ current }: Props) {
  return (
    <header className="nav">
      <div className="nav-inner">
        <a className="brand" href="/">
          <span className="brand-mark" aria-hidden="true" />
          <span>Tonet</span>
        </a>
        <nav className="nav-links" id="site-nav-links" aria-label="Main">
          <a id="nav-download" className="nav-link" href="/#download">
            Download
          </a>
          <a id="nav-features" className="nav-link" href="/#features">
            Features
          </a>
          <a className={navCls(current === "compare")} href="/compare.html">
            Compare
          </a>
          <a className={navCls(current === "roadmap")} href="/roadmap.html">
            Roadmap
          </a>
          <a
            id="nav-docs"
            className={navCls(current === "docs" || current === "handbook")}
            href="/docs.html"
          >
            Documentation
          </a>
          <a
            className="nav-github"
            href="https://github.com/usetonet/tonet-browser"
            target="_blank"
            rel="noopener noreferrer"
          >
            <FaGithub aria-hidden title="GitHub" size={20} />
            <span className="nav-github-text">GitHub</span>
          </a>
        </nav>
      </div>
    </header>
  );
}
