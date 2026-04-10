import "./styles/global.css";

export type DetectedOS = "windows" | "linux" | "macos" | "unknown";

export function detectOS(): DetectedOS {
  const ua = navigator.userAgent.toLowerCase();
  const platform = (navigator as Navigator & { userAgentData?: { platform?: string } }).userAgentData?.platform?.toLowerCase() ?? "";

  if (/win/.test(ua) || platform.includes("win")) return "windows";
  if (/mac|iphone|ipad|ipod/.test(ua) || platform.includes("mac")) return "macos";
  if (/linux|android/.test(ua) || platform.includes("linux")) return "linux";
  return "unknown";
}

async function loadVersion(): Promise<{ version: string } | null> {
  try {
    const r = await fetch("/version.json", { cache: "no-store" });
    if (!r.ok) return null;
    return await r.json();
  } catch {
    return null;
  }
}

function setActiveOS(os: DetectedOS): void {
  const tabs = document.querySelectorAll<HTMLButtonElement>(".os-tab");
  const panels = document.querySelectorAll<HTMLElement>(".os-panel");

  const map: Record<DetectedOS, string> = {
    windows: "panel-windows",
    linux: "panel-linux",
    macos: "panel-macos",
    unknown: "panel-windows",
  };

  const panelId = map[os] ?? "panel-windows";

  tabs.forEach((t) => {
    const active = t.dataset.os === os || (os === "unknown" && t.dataset.os === "windows");
    t.classList.toggle("active", !!active);
    t.setAttribute("aria-selected", active ? "true" : "false");
  });

  panels.forEach((p) => {
    p.classList.toggle("active", p.id === panelId);
  });
}

function wireCopyButtons(): void {
  document.querySelectorAll<HTMLButtonElement>(".copy-btn").forEach((btn) => {
    btn.addEventListener("click", async () => {
      const id = btn.dataset.copy;
      if (!id) return;
      const el = document.getElementById(id);
      if (!el || !(el instanceof HTMLElement)) return;
      const text = el.innerText;
      try {
        await navigator.clipboard.writeText(text);
        const prev = btn.textContent;
        btn.textContent = "¡Copiado!";
        setTimeout(() => {
          btn.textContent = prev;
        }, 1600);
      } catch {
        btn.textContent = "Error";
        setTimeout(() => {
          btn.textContent = "Copiar";
        }, 1600);
      }
    });
  });
}

async function main(): Promise<void> {
  const os = detectOS();
  const detectedEl = document.getElementById("os-detected");
  if (detectedEl) {
    const labels: Record<DetectedOS, string> = {
      windows: "Detectado: Windows — te mostramos los instaladores MSI/EXE.",
      linux: "Detectado: Linux — te mostramos .deb y comandos de compilación.",
      macos: "Detectado: macOS — te mostramos la ruta de compilación local.",
      unknown: "No pudimos detectar el SO — mostrando Windows por defecto.",
    };
    detectedEl.textContent = labels[os];
    detectedEl.hidden = false;
  }

  setActiveOS(os === "unknown" ? "windows" : os);

  document.querySelectorAll<HTMLButtonElement>(".os-tab").forEach((tab) => {
    tab.addEventListener("click", () => {
      const v = tab.dataset.os as DetectedOS | undefined;
      if (v) setActiveOS(v);
    });
  });

  const meta = await loadVersion();
  const ver = meta?.version ?? "…";
  const pill = document.getElementById("version-pill");
  if (pill) pill.textContent = `Versión actual del proyecto: ${ver}`;

  const tag = `v${ver}`;
  const base = `https://github.com/usetonet/tonet-browser/releases/download/${tag}`;
  const winMsi = document.getElementById("win-msi") as HTMLAnchorElement | null;
  const winExe = document.getElementById("win-exe") as HTMLAnchorElement | null;
  const linuxDeb = document.getElementById("linux-deb") as HTMLAnchorElement | null;
  if (meta?.version && winMsi && winExe && linuxDeb) {
    winMsi.href = `${base}/Tonet-${ver}-x64.msi`;
    winExe.href = `${base}/Tonet-Setup-${ver}-x64.exe`;
    linuxDeb.href = `${base}/tonet_${ver}_amd64.deb`;
  }

  wireCopyButtons();
}

main();
