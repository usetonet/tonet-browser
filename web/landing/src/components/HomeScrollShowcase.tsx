import type { CSSProperties } from "react";
import { useEffect, useRef, useState } from "react";
import { getLandingStrings, resolveSiteLang } from "../site-i18n";

function smoothstep(p: number, a: number, b: number): number {
  if (p <= a) return 0;
  if (p >= b) return 1;
  const t = (p - a) / (b - a);
  return t * t * (3 - 2 * t);
}

function layerStyle(t: number, tx: number, ty: number, dim: number): CSSProperties {
  const d = t * dim;
  return {
    opacity: d,
    transform: `translate3d(${tx * (1 - t)}px, ${ty * (1 - t)}px, 0) scale(${0.94 + 0.06 * t})`,
  };
}

export function HomeScrollShowcase() {
  const L = getLandingStrings(resolveSiteLang());
  const wrapRef = useRef<HTMLDivElement>(null);
  const [p, setP] = useState(0);
  const [reduced, setReduced] = useState(false);

  useEffect(() => {
    const mq = window.matchMedia("(prefers-reduced-motion: reduce)");
    setReduced(mq.matches);
    const h = () => setReduced(mq.matches);
    mq.addEventListener("change", h);
    return () => mq.removeEventListener("change", h);
  }, []);

  useEffect(() => {
    if (reduced) {
      setP(1);
      return;
    }
    const el = wrapRef.current;
    if (!el) return;
    const onScroll = () => {
      const rect = el.getBoundingClientRect();
      const vh = window.innerHeight;
      const range = Math.max(1, el.offsetHeight - vh);
      const scrolled = -rect.top;
      setP(Math.min(1, Math.max(0, scrolled / range)));
    };
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });
    window.addEventListener("resize", onScroll);
    return () => {
      window.removeEventListener("scroll", onScroll);
      window.removeEventListener("resize", onScroll);
    };
  }, [reduced]);

  const t1 = smoothstep(p, 0, 0.3);
  const t2 = smoothstep(p, 0.18, 0.48);
  const t3 = smoothstep(p, 0.36, 0.68);
  const tFinal = smoothstep(p, 0.74, 0.98);
  const dimLayers = 1 - smoothstep(p, 0.7, 0.94);
  /** At end of scroll range `dimLayers`→0; keep artwork visible in reduced-motion snapshot. */
  const dimActive = reduced ? 1 : dimLayers;
  const hintOpacity = 1 - smoothstep(p, 0.2, 0.5);

  const tPreface = reduced ? 0.95 : 1 - smoothstep(p, 0, 0.22);
  /** Fades badge copy before the final lockup; multiplied with parent layer opacity (via stage `layerStyle`). */
  const tBadgeDim = reduced ? 1 : 1 - smoothstep(p, 0.62, 0.88);
  /** Fade shield label as the card layer takes over so two messages never compete side‑by‑side. */
  const tShieldLabel = reduced ? 1 : 1 - smoothstep(t3, 0.22, 0.52);

  const lockup = (
    <div className="home-scroll-final-row flex flex-wrap items-center justify-center gap-2 sm:gap-3">
      <img src="/tonet.svg" width={40} height={40} className="size-10 rounded-[10px] shadow-lg" alt="Tonet" />
      <span className="home-scroll-final-mid text-balance">{L.scrollPoweredBy}</span>
      <img
        src="/servo-logo.webp"
        width={200}
        height={48}
        className="h-9 w-auto max-w-[min(78vw,240px)] object-contain sm:h-10"
        alt="Servo"
      />
    </div>
  );

  const viewportInner = (
    <>
      <div className="home-scroll-viewport-bg" style={{ opacity: 0.35 + 0.4 * p }} />

      {/* Layer stack (bottom → top): waves → preface → shield → card → final. Upper paints over lower. */}
      <div className="home-scroll-layer-wrap home-scroll-layer-wrap--z1">
        <div className="home-scroll-art-stage" style={layerStyle(t1, 0, 0, dimActive)}>
          <img
            src="/showcase/layer-1.svg"
            alt=""
            className="home-scroll-art-img"
            width={800}
            height={500}
            decoding="async"
          />
        </div>
      </div>

      <div className="home-scroll-preface-wrap" aria-hidden="true">
        <p
          className="home-scroll-preface"
          style={{
            opacity: tPreface,
            transform: `translateY(${(1 - tPreface) * 14}px)`,
          }}
        >
          {L.scrollIncoming}
        </p>
      </div>

      <div className="home-scroll-layer-wrap home-scroll-layer-wrap--z2">
        <div className="home-scroll-art-stage" style={layerStyle(t2, 0, 0, dimActive)}>
          <img
            src="/showcase/layer-2.svg"
            alt=""
            className="home-scroll-art-img"
            width={800}
            height={500}
            decoding="async"
          />
          <div
            className="home-scroll-embed home-scroll-embed--shield"
            aria-hidden="true"
            style={{ opacity: tBadgeDim * tShieldLabel }}
          >
            {L.scrollBadgeSecure}
          </div>
        </div>
      </div>

      <div className="home-scroll-layer-wrap home-scroll-layer-wrap--z3">
        <div className="home-scroll-art-stage" style={layerStyle(t3, 0, 45, dimActive)}>
          <img
            src="/showcase/layer-3.svg"
            alt=""
            className="home-scroll-art-img"
            width={800}
            height={500}
            decoding="async"
          />
          <div className="home-scroll-embed home-scroll-embed--card" aria-hidden="true" style={{ opacity: tBadgeDim }}>
            {L.scrollBadgePrivacy}
          </div>
        </div>
      </div>

      <div
        className="home-scroll-final"
        style={{
          opacity: tFinal,
          transform: `translate3d(0, ${(1 - tFinal) * 20}px, 0) scale(${0.97 + 0.03 * tFinal})`,
        }}
      >
        <div className="home-scroll-final-stack">
          <p className="home-scroll-final-engine" style={{ opacity: Math.min(1, tFinal * 1.15) }}>
            {L.scrollEngineTagline}
          </p>
          {lockup}
        </div>
      </div>
    </>
  );

  const frame = (
    <div className="home-browser-frame mx-auto w-full max-w-[920px]">
      <div className="home-browser-chrome">
        <span className="home-browser-dot bg-[#ff5f56]" />
        <span className="home-browser-dot bg-[#febc2e]" />
        <span className="home-browser-dot bg-[#28c840]" />
        <div className="home-browser-url font-mono">tonet://reading — Servo · lightweight surface</div>
      </div>
      <div className="home-browser-viewport home-browser-viewport--scroll">{viewportInner}</div>
    </div>
  );

  if (reduced) {
    return (
      <div className="mx-auto max-w-[1120px] px-6 py-2 sm:py-4" role="region" aria-label={L.homeScrollAria}>
        {frame}
      </div>
    );
  }

  return (
    <div
      ref={wrapRef}
      className="home-scroll-pin-wrap relative mx-auto max-w-[1120px] px-6"
      role="region"
      aria-label={L.homeScrollAria}
    >
      <div className="home-scroll-sticky">
        {frame}
        <p
          className="home-scroll-progress-hint mt-4 text-center text-[0.78rem] font-medium uppercase tracking-[0.14em] text-[#9aa3b5]/80 transition-opacity duration-300"
          style={{ opacity: hintOpacity }}
        >
          {L.homeScrollHint}
        </p>
      </div>
    </div>
  );
}
