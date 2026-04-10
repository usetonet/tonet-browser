/** Landing + docs: English-first HTML; runtime locale from `navigator.language` or user override. */

export type SiteLang = "en" | "es" | "de" | "fr";

const STORAGE_KEY = "tonet-site-lang";

export function getStoredSiteLang(): SiteLang | null {
  try {
    const v = localStorage.getItem(STORAGE_KEY);
    if (v === "en" || v === "es" || v === "de" || v === "fr") return v;
  } catch {
    /* private mode */
  }
  return null;
}

export function setStoredSiteLang(lang: SiteLang | "clear"): void {
  try {
    if (lang === "clear") localStorage.removeItem(STORAGE_KEY);
    else localStorage.setItem(STORAGE_KEY, lang);
  } catch {
    /* ignore */
  }
}

/** Primary subtag from `navigator.languages` / `navigator.language`. */
export function resolveSiteLang(): SiteLang {
  const stored = getStoredSiteLang();
  if (stored) return stored;

  const candidates = navigator.languages?.length
    ? navigator.languages
    : [navigator.language || "en"];
  for (const raw of candidates) {
    const primary = raw.split(/[-_]/)[0]?.toLowerCase() ?? "en";
    if (primary === "es" || primary === "de" || primary === "fr") return primary;
  }
  return "en";
}

export interface CopyUi {
  copy: string;
  copied: string;
  error: string;
}

export const copyUi: Record<SiteLang, CopyUi> = {
  en: { copy: "Copy", copied: "Copied!", error: "Error" },
  es: { copy: "Copiar", copied: "¡Copiado!", error: "Error" },
  de: { copy: "Kopieren", copied: "Kopiert!", error: "Fehler" },
  fr: { copy: "Copier", copied: "Copié !", error: "Erreur" },
};

import type { DetectedOS } from "./detect-os";

export function detectedOsLine(lang: SiteLang, os: DetectedOS): string {
  const m = {
    en: {
      windows: "Detected: Windows — showing MSI/EXE installers.",
      linux: "Detected: Linux — showing .deb and build commands.",
      macos: "Detected: macOS — showing local build steps.",
      unknown: "Could not detect your OS — defaulting to Windows.",
    },
    es: {
      windows: "Detectado: Windows — te mostramos los instaladores MSI/EXE.",
      linux: "Detectado: Linux — te mostramos .deb y comandos de compilación.",
      macos: "Detectado: macOS — te mostramos la ruta de compilación local.",
      unknown: "No pudimos detectar el SO — mostrando Windows por defecto.",
    },
    de: {
      windows: "Erkannt: Windows — MSI/EXE-Installer werden angezeigt.",
      linux: "Erkannt: Linux — .deb und Build-Befehle werden angezeigt.",
      macos: "Erkannt: macOS — lokale Build-Schritte werden angezeigt.",
      unknown: "Betriebssystem unbekannt — Standard: Windows.",
    },
    fr: {
      windows: "Détecté : Windows — installateurs MSI/EXE affichés.",
      linux: "Détecté : Linux — .deb et commandes de compilation affichés.",
      macos: "Détecté : macOS — étapes de compilation locales affichées.",
      unknown: "Système inconnu — affichage Windows par défaut.",
    },
  }[lang];
  return m[os];
}

export function versionPillPrefix(lang: SiteLang): string {
  const map: Record<SiteLang, string> = {
    en: "Current project version:",
    es: "Versión actual del proyecto:",
    de: "Aktuelle Projektversion:",
    fr: "Version actuelle du projet :",
  };
  return map[lang];
}

interface LandingStrings {
  metaDescription: string;
  title: string;
  navAria: string;
  navDownload: string;
  navFeatures: string;
  navDocs: string;
  heroTitle: string;
  heroLead: string;
  heroDownload: string;
  heroDocs: string;
  featuresTitle: string;
  featuresLead: string;
  c1t: string;
  c1p: string;
  c2t: string;
  c2p: string;
  c3t: string;
  c3p: string;
  c4t: string;
  c4p: string;
  downloadTitle: string;
  downloadLead: string;
  panelWinTitle: string;
  panelWinP1: string;
  panelWinFoot: string;
  winMsi: string;
  winExe: string;
  panelLinuxTitle: string;
  panelLinuxP1: string;
  linuxDeb: string;
  linuxH4src: string;
  linuxH4user: string;
  panelMacTitle: string;
  panelMacP1: string;
  footer1: string;
  footer2: string;
  langSwitcher: string;
}

const landing: Record<SiteLang, LandingStrings> = {
  en: {
    metaDescription:
      "Tonet — a minimal from-scratch browser. Light, fast, and intentional. Downloads for Windows, Linux, and docs.",
    title: "Tonet — Browse light",
    navAria: "Main",
    navDownload: "Download",
    navFeatures: "Features",
    navDocs: "Documentation",
    heroTitle: "Browse without the weight.<br />Push back on web bloat.",
    heroLead:
      "Tonet is a browser built with a clear goal: extreme speed, minimal weight, and an in-house engine for essential content. No Blink, WebKit, or CEF — you control what comes in.",
    heroDownload: "Download Tonet",
    heroDocs: "Read the docs",
    featuresTitle: "Built to get to the point",
    featuresLead:
      "An honest MVP: network + parser + minimal renderer. Tonet does not try to render today’s industrial web; it tries to make reading and search feel human again.",
    c1t: "Purity filter",
    c1p: "Pages over 1 MB are rejected. Tonet only loads what fits a lightweight standard.",
    c2t: "Custom parser",
    c2p: "Manual extraction of title and essential text — without dragging in a full HTML engine.",
    c3t: "Updates on your terms",
    c3p: "Check on launch, every 24 hours, or only when you want. One click opens official downloads.",
    c4t: "Open source",
    c4p: "Full transparency on GitHub. Signed installers and documented pipelines for operators.",
    downloadTitle: "Download Tonet",
    downloadLead:
      "We highlight the option that matches your system. You can always switch tabs manually.",
    panelWinTitle: "Windows",
    panelWinP1: "CI-built installers: MSI (enterprise / Intune) and EXE via Inno Setup.",
    panelWinFoot:
      "After a release, files appear as Tonet-&lt;version&gt;-x64.msi and Tonet-Setup-&lt;version&gt;-x64.exe. The “latest” release page always lists current binaries.",
    winMsi: "Download MSI (x64)",
    winExe: "Download EXE (x64)",
    panelLinuxTitle: "Linux",
    panelLinuxP1: ".deb package for amd64 (when the release workflow publishes it) or build from source.",
    linuxDeb: "Open releases (.deb)",
    linuxH4src: "Build from the repository",
    linuxH4user: "Per-user install (desktop + binary)",
    panelMacTitle: "macOS",
    panelMacP1:
      "A signed, notarized .app bundle is on the roadmap. Until then, build with stable Rust and Xcode Command Line Tools.",
    footer1: "usetonet.com — project",
    footer2: "Landing on Cloudflare Workers · Tonet engine under active development",
    langSwitcher: "Site language",
  },
  es: {
    metaDescription:
      "Tonet — navegador minimalista desde cero. Ligero, rápido y con intención. Descargas para Windows, Linux y documentación.",
    title: "Tonet — Navega ligero",
    navAria: "Principal",
    navDownload: "Descargar",
    navFeatures: "Características",
    navDocs: "Documentación",
    heroTitle: "Navega sin peso.<br />Rechaza la basura web.",
    heroLead:
      "Tonet es un navegador en construcción con una filosofía clara: velocidad extrema, ligereza absoluta y un motor propio para contenido esencial. Sin Blink, WebKit ni CEF — tú controlas qué entra.",
    heroDownload: "Descargar Tonet",
    heroDocs: "Ver documentación",
    featuresTitle: "Diseñado para ir al grano",
    featuresLead:
      "Un MVP honesto: red + parser + render mínimos. Tonet no pretende renderizar la web industrial de hoy; pretende devolver la lectura y la búsqueda a algo humano.",
    c1t: "Filtro de pureza",
    c1p: "Páginas mayores de 1 MB se rechazan. Tonet solo carga lo que cabe en un estándar de ligereza.",
    c2t: "Parser propio",
    c2p: "Extracción manual de título y texto esencial — sin arrastrar un motor HTML completo.",
    c3t: "Actualizaciones bajo tu control",
    c3p: "Comprueba novedades al iniciar, cada 24 h o solo cuando tú quieras. Un clic abre las descargas oficiales.",
    c4t: "Código abierto",
    c4p: "Transparencia total en GitHub. Instaladores firmados y pipelines documentados para quien despliega.",
    downloadTitle: "Descargar Tonet",
    downloadLead:
      "Detectamos tu sistema para resaltar la opción adecuada. Siempre puedes cambiar de pestaña manualmente.",
    panelWinTitle: "Windows",
    panelWinP1: "Instaladores generados en CI: paquete MSI (empresas / InTune) y EXE con Inno Setup.",
    panelWinFoot:
      "Tras publicar un release, los archivos aparecen como Tonet-&lt;versión&gt;-x64.msi y Tonet-Setup-&lt;versión&gt;-x64.exe. La página de «latest» lista siempre los binarios vigentes.",
    winMsi: "Descargar MSI (x64)",
    winExe: "Descargar EXE (x64)",
    panelLinuxTitle: "Linux",
    panelLinuxP1:
      "Paquete .deb para amd64 (cuando el workflow de release lo publique) o compilación desde fuente.",
    linuxDeb: "Abrir releases (.deb)",
    linuxH4src: "Compilar desde el repositorio",
    linuxH4user: "Instalación de usuario (desktop + binario)",
    panelMacTitle: "macOS",
    panelMacP1:
      "El bundle .app firmado y notarizado está en la hoja de ruta. Mientras tanto puedes compilar con Rust estable y Xcode Command Line Tools.",
    footer1: "usetonet.com — proyecto",
    footer2: "Landing servida con Cloudflare Workers · Motor Tonet en desarrollo activo",
    langSwitcher: "Idioma del sitio",
  },
  de: {
    metaDescription:
      "Tonet — ein minimales Browser-Projekt von Grund auf. Leicht, schnell, bewusst. Downloads für Windows, Linux und Dokumentation.",
    title: "Tonet — Leicht surfen",
    navAria: "Hauptnavigation",
    navDownload: "Download",
    navFeatures: "Funktionen",
    navDocs: "Dokumentation",
    heroTitle: "Surfen ohne Ballast.<br />Web-Bloat zurückweisen.",
    heroLead:
      "Tonet wird mit klarem Ziel entwickelt: hohe Geschwindigkeit, geringes Gewicht und eine eigene Engine für Wesentliches. Kein Blink, WebKit oder CEF — Sie entscheiden, was reinkommt.",
    heroDownload: "Tonet herunterladen",
    heroDocs: "Dokumentation",
    featuresTitle: "Auf den Punkt gebaut",
    featuresLead:
      "Ein ehrliches MVP: Netzwerk + Parser + minimaler Renderer. Tonet soll nicht das heutige Industrie-Web rendern, sondern Lesen und Suchen wieder menschlich machen.",
    c1t: "Reinheitsfilter",
    c1p: "Seiten über 1 MB werden abgelehnt. Tonet lädt nur, was in ein leichtgewichtiges Budget passt.",
    c2t: "Eigener Parser",
    c2p: "Manuelle Extraktion von Titel und Kerntext — ohne eine komplette HTML-Engine.",
    c3t: "Updates nach Ihren Regeln",
    c3p: "Beim Start, alle 24 Stunden oder nur manuell prüfen. Ein Klick öffnet offizielle Downloads.",
    c4t: "Open Source",
    c4p: "Volle Transparenz auf GitHub. Signierte Installer und dokumentierte Pipelines für Betreiber.",
    downloadTitle: "Tonet herunterladen",
    downloadLead:
      "Wir heben die passende Option für Ihr System hervor. Sie können die Registerkarten jederzeit manuell wechseln.",
    panelWinTitle: "Windows",
    panelWinP1: "CI-erstellte Installer: MSI (Unternehmen / Intune) und EXE mit Inno Setup.",
    panelWinFoot:
      "Nach einem Release erscheinen Dateien als Tonet-&lt;Version&gt;-x64.msi und Tonet-Setup-&lt;Version&gt;-x64.exe. Die „latest“-Seite listet stets aktuelle Binärdateien.",
    winMsi: "MSI laden (x64)",
    winExe: "EXE laden (x64)",
    panelLinuxTitle: "Linux",
    panelLinuxP1: ".deb für amd64 (wenn der Release-Workflow es veröffentlicht) oder aus dem Quellcode bauen.",
    linuxDeb: "Releases öffnen (.deb)",
    linuxH4src: "Aus dem Repository bauen",
    linuxH4user: "Benutzerinstallation (Desktop + Binary)",
    panelMacTitle: "macOS",
    panelMacP1:
      "Ein signiertes, notarisiertes .app-Bundle ist geplant. Bis dahin: Build mit stable Rust und Xcode Command Line Tools.",
    footer1: "usetonet.com — Projekt",
    footer2: "Landing auf Cloudflare Workers · Tonet-Engine in aktiver Entwicklung",
    langSwitcher: "Sprache der Website",
  },
  fr: {
    metaDescription:
      "Tonet — navigateur minimal créé from scratch. Léger, rapide et volontaire. Téléchargements Windows, Linux et documentation.",
    title: "Tonet — Naviguer léger",
    navAria: "Principal",
    navDownload: "Télécharger",
    navFeatures: "Fonctionnalités",
    navDocs: "Documentation",
    heroTitle: "Naviguez sans le poids.<br />Rejetez le superflu du web.",
    heroLead:
      "Tonet est un navigateur construit autour d’un objectif clair : vitesse, légèreté et moteur maison pour l’essentiel. Pas de Blink, WebKit ni CEF — vous contrôlez ce qui entre.",
    heroDownload: "Télécharger Tonet",
    heroDocs: "Lire la documentation",
    featuresTitle: "Conçu pour aller à l’essentiel",
    featuresLead:
      "Un MVP honnête : réseau + parseur + rendu minimal. Tonet ne vise pas le web industriel d’aujourd’hui ; il veut rendre lecture et recherche plus humaines.",
    c1t: "Filtre de pureté",
    c1p: "Les pages de plus de 1 Mo sont refusées. Tonet ne charge que ce qui respecte un budget léger.",
    c2t: "Parseur dédié",
    c2p: "Extraction manuelle du titre et du texte utile — sans embarquer un moteur HTML complet.",
    c3t: "Mises à jour selon vos règles",
    c3p: "Vérification au lancement, toutes les 24 h ou uniquement à la demande. Un clic ouvre les téléchargements officiels.",
    c4t: "Open source",
    c4p: "Transparence sur GitHub. Installateurs signés et pipelines documentés pour les opérateurs.",
    downloadTitle: "Télécharger Tonet",
    downloadLead:
      "Nous mettons en avant l’option adaptée à votre système. Vous pouvez toujours changer d’onglet manuellement.",
    panelWinTitle: "Windows",
    panelWinP1: "Installateurs produits en CI : MSI (entreprise / Intune) et EXE via Inno Setup.",
    panelWinFoot:
      "Après une release, les fichiers apparaissent comme Tonet-&lt;version&gt;-x64.msi et Tonet-Setup-&lt;version&gt;-x64.exe. La page « latest » liste toujours les binaires à jour.",
    winMsi: "Télécharger MSI (x64)",
    winExe: "Télécharger EXE (x64)",
    panelLinuxTitle: "Linux",
    panelLinuxP1: "Paquet .deb pour amd64 (lorsque le workflow le publie) ou compilation depuis les sources.",
    linuxDeb: "Ouvrir les releases (.deb)",
    linuxH4src: "Compiler depuis le dépôt",
    linuxH4user: "Installation utilisateur (bureau + binaire)",
    panelMacTitle: "macOS",
    panelMacP1:
      "Un bundle .app signé et notarisé est prévu. En attendant, compilez avec Rust stable et Xcode Command Line Tools.",
    footer1: "usetonet.com — projet",
    footer2: "Landing sur Cloudflare Workers · Moteur Tonet en développement actif",
    langSwitcher: "Langue du site",
  },
};

function setHtml(id: string, html: string): void {
  const el = document.getElementById(id);
  if (el) el.innerHTML = html;
}

function setText(id: string, text: string): void {
  const el = document.getElementById(id);
  if (el) el.textContent = text;
}

export function applyLandingLocale(lang: SiteLang): void {
  const L = landing[lang];
  document.documentElement.lang = lang;
  document.title = L.title;
  const meta = document.querySelector<HTMLMetaElement>('meta[name="description"]');
  if (meta) meta.content = L.metaDescription;

  const nav = document.getElementById("site-nav-links");
  if (nav) nav.setAttribute("aria-label", L.navAria);

  setText("nav-download", L.navDownload);
  setText("nav-features", L.navFeatures);
  setText("nav-docs", L.navDocs);
  setHtml("hero-title", L.heroTitle);
  setText("hero-lead", L.heroLead);
  setText("hero-download", L.heroDownload);
  setText("hero-docs", L.heroDocs);
  setText("features-title", L.featuresTitle);
  setText("features-lead", L.featuresLead);
  setText("card-purity-t", L.c1t);
  setText("card-purity-p", L.c1p);
  setText("card-parser-t", L.c2t);
  setText("card-parser-p", L.c2p);
  setText("card-updates-t", L.c3t);
  setText("card-updates-p", L.c3p);
  setText("card-opensource-t", L.c4t);
  setText("card-opensource-p", L.c4p);
  setText("download-title", L.downloadTitle);
  setText("download-lead", L.downloadLead);
  setText("panel-win-h3", L.panelWinTitle);
  setText("panel-win-p1", L.panelWinP1);
  setHtml("panel-win-foot", L.panelWinFoot);
  setText("win-msi", L.winMsi);
  setText("win-exe", L.winExe);
  setText("panel-linux-h3", L.panelLinuxTitle);
  setText("panel-linux-p1", L.panelLinuxP1);
  setText("linux-deb", L.linuxDeb);
  setText("linux-h4-src", L.linuxH4src);
  setText("linux-h4-user", L.linuxH4user);
  setText("panel-mac-h3", L.panelMacTitle);
  setText("panel-mac-p1", L.panelMacP1);
  setText("footer-line1-prefix", L.footer1);
  setText("footer-line2", L.footer2);
  const swLabel = document.getElementById("lang-switcher-label");
  if (swLabel) swLabel.textContent = L.langSwitcher;
}

interface DocsStrings {
  metaDescription: string;
  title: string;
  navHome: string;
  navDownload: string;
  h1: string;
  lead: string;
  installH: string;
  installP: string;
  updatesH: string;
  updatesP: string;
  updatesLi1: string;
  updatesLi2: string;
  updatesLi3: string;
  updatesP2: string;
  signH: string;
  signP: string;
  signFoot: string;
  debH: string;
  debP: string;
  appH: string;
  appP: string;
  cfH: string;
  cfP1: string;
  cfP2: string;
  footerBack: string;
}

const docs: Record<SiteLang, DocsStrings> = {
  en: {
    metaDescription: "Tonet documentation: install, updates, code signing, and packaging.",
    title: "Documentation — Tonet",
    navHome: "Home",
    navDownload: "Download",
    h1: "Documentation",
    lead: "Short guides for users and maintainers.",
    installH: "Installation",
    installP:
      "See the <a href=\"/#download\">downloads section</a>. Windows offers MSI and EXE; Linux supports .deb and a user install script; macOS is local builds until a signed bundle ships.",
    updatesH: "In-browser updates",
    updatesP:
      "Tonet checks the GitHub Releases API (it does not install binaries for you). By default it checks on startup. In <strong>Settings (⚙)</strong> you can choose:",
    updatesLi1: "<strong>On startup only</strong> — one check when the app opens.",
    updatesLi2: "<strong>Automatic (24 h)</strong> — repeats while Tonet is running.",
    updatesLi3: "<strong>Manual only</strong> — only with “Check now”.",
    updatesP2:
      "Settings are stored in the system config directory (for example <code>%APPDATA%\\tonet</code> on Windows).",
    signH: "Authenticode signing (Windows)",
    signP:
      "For SmartScreen trust, sign MSI and EXE with a code-signing certificate (SHA-256) on Windows using <code>signtool</code> from the Windows SDK.",
    signFoot:
      "In CI, store the PFX in an encrypted secret and use a self-hosted runner or an action that invokes <code>signtool</code>. Never commit certificates to the repository.",
    debH: "Debian package (.deb)",
    debP:
      "The script <code>packaging/debian/build-deb.sh</code> builds a <code>.deb</code> from <code>cargo build --release -p tonet</code>. Requires <code>dpkg-deb</code> (Linux).",
    appH: "AppImage (Linux)",
    appP:
      "Shipping a universal AppImage requires bundling GTK/GL dependencies — consider <a href=\"https://github.com/linuxdeploy/linuxdeploy\" target=\"_blank\" rel=\"noopener\">linuxdeploy</a> + the GTK plugin once the binary stabilizes. A dedicated workflow may land in a future release.",
    cfH: "Deploy this landing (Cloudflare Workers)",
    cfP1: "From the <code>web/landing</code> directory:",
    cfP2:
      "Set <code>CLOUDFLARE_API_TOKEN</code> and <code>CLOUDFLARE_ACCOUNT_ID</code> in GitHub Actions, or run <code>wrangler login</code> locally.",
    footerBack: "← Back to home",
  },
  es: {
    metaDescription:
      "Documentación de Tonet: instalación, actualizaciones, firma de código y empaquetado.",
    title: "Documentación — Tonet",
    navHome: "Inicio",
    navDownload: "Descargar",
    h1: "Documentación",
    lead: "Guías breves para usuarios y mantenedores del proyecto.",
    installH: "Instalación",
    installP:
      "Consulta la <a href=\"/#download\">sección de descargas</a>. Windows ofrece MSI y EXE; Linux admite <code>.deb</code> y script de usuario; macOS, compilación local hasta que haya bundle firmado.",
    updatesH: "Actualizaciones en el navegador",
    updatesP:
      "Tonet consulta la API de GitHub Releases (sin instalar binarios por ti). Por defecto comprueba al iniciar. En <strong>Ajustes (⚙)</strong> puedes elegir:",
    updatesLi1: "<strong>Solo al iniciar</strong> — una comprobación al abrir la app.",
    updatesLi2: "<strong>Automático (24 h)</strong> — repite mientras Tonet está abierto.",
    updatesLi3: "<strong>Solo manual</strong> — únicamente con «Comprobar ahora».",
    updatesP2:
      "Los ajustes se guardan en el directorio de configuración del sistema (por ejemplo <code>%APPDATA%\\tonet</code> en Windows).",
    signH: "Firma Authenticode (Windows)",
    signP:
      "Para que SmartScreen confíe en los instaladores, firma el MSI y el EXE con un certificado de publicador de código (SHA-256) en una máquina Windows con <code>signtool</code> del Windows SDK.",
    signFoot:
      "En CI, guarda el PFX en un secreto cifrado y usa un runner auto-hospedado o una acción que invoque <code>signtool</code>. No subas certificados al repositorio.",
    debH: "Paquete Debian (.deb)",
    debP:
      "El script <code>packaging/debian/build-deb.sh</code> genera un <code>.deb</code> desde <code>cargo build --release -p tonet</code>. Requiere <code>dpkg-deb</code> (Linux).",
    appH: "AppImage (Linux)",
    appP:
      "Para distribuir un AppImage universal necesitas empaquetar dependencias de GTK/GL — recomendamos evaluar <a href=\"https://github.com/linuxdeploy/linuxdeploy\" target=\"_blank\" rel=\"noopener\">linuxdeploy</a> + plugin GTK cuando el binario esté estabilizado. El repositorio puede añadir un workflow dedicado en una release futura.",
    cfH: "Desplegar esta landing (Cloudflare Workers)",
    cfP1: "En el directorio <code>web/landing</code>:",
    cfP2:
      "Configura <code>CLOUDFLARE_API_TOKEN</code> y <code>CLOUDFLARE_ACCOUNT_ID</code> en GitHub Actions o ejecuta <code>wrangler login</code> en local.",
    footerBack: "← Volver al inicio",
  },
  de: {
    metaDescription: "Tonet-Dokumentation: Installation, Updates, Codesignatur und Packaging.",
    title: "Dokumentation — Tonet",
    navHome: "Start",
    navDownload: "Download",
    h1: "Dokumentation",
    lead: "Kurze Anleitungen für Nutzer und Maintainer.",
    installH: "Installation",
    installP:
      "Siehe den <a href=\"/#download\">Download-Bereich</a>. Windows: MSI und EXE; Linux: <code>.deb</code> und Benutzer-Installationskript; macOS: lokaler Build, bis ein signiertes Bundle verfügbar ist.",
    updatesH: "Updates in der App",
    updatesP:
      "Tonet nutzt die GitHub-Releases-API (installiert keine Binärdateien für Sie). Standard: Prüfung beim Start. Unter <strong>Einstellungen (⚙)</strong> wählen Sie:",
    updatesLi1: "<strong>Nur beim Start</strong> — eine Prüfung beim Öffnen.",
    updatesLi2: "<strong>Automatisch (24 h)</strong> — wiederholt, solange Tonet läuft.",
    updatesLi3: "<strong>Nur manuell</strong> — nur mit „Jetzt prüfen“.",
    updatesP2:
      "Einstellungen liegen im System-Konfigurationsverzeichnis (z. B. <code>%APPDATA%\\tonet</code> unter Windows).",
    signH: "Authenticode-Signatur (Windows)",
    signP:
      "Für SmartScreen-Vertrauen MSI und EXE mit Codesignatur-Zertifikat (SHA-256) unter Windows mit <code>signtool</code> aus dem Windows SDK signieren.",
    signFoot:
      "In CI: PFX in verschlüsseltem Secret speichern und Self-Hosted-Runner oder Action mit <code>signtool</code> nutzen. Keine Zertifikate ins Repository legen.",
    debH: "Debian-Paket (.deb)",
    debP:
      "Das Skript <code>packaging/debian/build-deb.sh</code> erzeugt ein <code>.deb</code> aus <code>cargo build --release -p tonet</code>. Benötigt <code>dpkg-deb</code> (Linux).",
    appH: "AppImage (Linux)",
    appP:
      "Ein universelles AppImage erfordert GTK/GL-Abhängigkeiten — später z. B. <a href=\"https://github.com/linuxdeploy/linuxdeploy\" target=\"_blank\" rel=\"noopener\">linuxdeploy</a> + GTK-Plugin prüfen. Ein Workflow kann in einer späteren Release folgen.",
    cfH: "Diese Landing deployen (Cloudflare Workers)",
    cfP1: "Im Verzeichnis <code>web/landing</code>:",
    cfP2:
      "<code>CLOUDFLARE_API_TOKEN</code> und <code>CLOUDFLARE_ACCOUNT_ID</code> in GitHub Actions setzen oder lokal <code>wrangler login</code> ausführen.",
    footerBack: "← Zur Startseite",
  },
  fr: {
    metaDescription:
      "Documentation Tonet : installation, mises à jour, signature de code et packaging.",
    title: "Documentation — Tonet",
    navHome: "Accueil",
    navDownload: "Télécharger",
    h1: "Documentation",
    lead: "Guides courts pour les utilisateurs et les mainteneurs.",
    installH: "Installation",
    installP:
      "Voir la <a href=\"/#download\">section téléchargements</a>. Windows : MSI et EXE ; Linux : <code>.deb</code> et script utilisateur ; macOS : compilation locale jusqu’à un bundle signé.",
    updatesH: "Mises à jour dans le navigateur",
    updatesP:
      "Tonet interroge l’API GitHub Releases (sans installer les binaires à votre place). Par défaut : vérification au lancement. Dans <strong>Réglages (⚙)</strong> vous pouvez choisir :",
    updatesLi1: "<strong>Au lancement uniquement</strong> — une vérification à l’ouverture.",
    updatesLi2: "<strong>Automatique (24 h)</strong> — tant que Tonet est ouvert.",
    updatesLi3: "<strong>Manuel seulement</strong> — uniquement via « Vérifier maintenant ».",
    updatesP2:
      "Les réglages sont stockés dans le dossier de configuration système (par ex. <code>%APPDATA%\\tonet</code> sous Windows).",
    signH: "Signature Authenticode (Windows)",
    signP:
      "Pour la confiance SmartScreen, signez MSI et EXE avec un certificat de signature de code (SHA-256) sous Windows via <code>signtool</code> du Windows SDK.",
    signFoot:
      "En CI, placez le PFX dans un secret chiffré et utilisez un runner self-hosted ou une action qui appelle <code>signtool</code>. Ne commitez pas les certificats.",
    debH: "Paquet Debian (.deb)",
    debP:
      "Le script <code>packaging/debian/build-deb.sh</code> produit un <code>.deb</code> à partir de <code>cargo build --release -p tonet</code>. Nécessite <code>dpkg-deb</code> (Linux).",
    appH: "AppImage (Linux)",
    appP:
      "Un AppImage universel impose d’empaqueter GTK/GL — envisager <a href=\"https://github.com/linuxdeploy/linuxdeploy\" target=\"_blank\" rel=\"noopener\">linuxdeploy</a> + plugin GTK une fois le binaire stabilisé. Un workflow dédié pourra suivre.",
    cfH: "Déployer cette landing (Cloudflare Workers)",
    cfP1: "Dans le dossier <code>web/landing</code> :",
    cfP2:
      "Définir <code>CLOUDFLARE_API_TOKEN</code> et <code>CLOUDFLARE_ACCOUNT_ID</code> dans GitHub Actions, ou lancer <code>wrangler login</code> en local.",
    footerBack: "← Retour à l’accueil",
  },
};

export function applyDocsLocale(lang: SiteLang): void {
  const D = docs[lang];
  document.documentElement.lang = lang;
  document.title = D.title;
  const meta = document.querySelector<HTMLMetaElement>('meta[name="description"]');
  if (meta) meta.content = D.metaDescription;
  setText("docs-nav-home", D.navHome);
  setText("docs-nav-download", D.navDownload);
  setText("docs-h1", D.h1);
  setText("docs-lead", D.lead);
  setText("docs-install-h", D.installH);
  setHtml("docs-install-p", D.installP);
  setText("docs-updates-h", D.updatesH);
  setHtml("docs-updates-p", D.updatesP);
  setHtml("docs-updates-li1", D.updatesLi1);
  setHtml("docs-updates-li2", D.updatesLi2);
  setHtml("docs-updates-li3", D.updatesLi3);
  setHtml("docs-updates-p2", D.updatesP2);
  setText("docs-sign-h", D.signH);
  setHtml("docs-sign-p", D.signP);
  setHtml("docs-sign-foot", D.signFoot);
  setText("docs-deb-h", D.debH);
  setHtml("docs-deb-p", D.debP);
  setText("docs-app-h", D.appH);
  setHtml("docs-app-p", D.appP);
  setText("docs-cf-h", D.cfH);
  setHtml("docs-cf-p1", D.cfP1);
  setHtml("docs-cf-p2", D.cfP2);
  setText("docs-footer-back", D.footerBack);
}

export function wireCopyButtons(lang: SiteLang): void {
  const cu = copyUi[lang];
  document.querySelectorAll<HTMLButtonElement>(".copy-btn").forEach((btn) => {
    btn.textContent = cu.copy;
    btn.addEventListener("click", async () => {
      const id = btn.dataset.copy;
      if (!id) return;
      const el = document.getElementById(id);
      if (!el || !(el instanceof HTMLElement)) return;
      const text = el.innerText;
      try {
        await navigator.clipboard.writeText(text);
        const prev = btn.textContent;
        btn.textContent = cu.copied;
        setTimeout(() => {
          btn.textContent = prev;
        }, 1600);
      } catch {
        btn.textContent = cu.error;
        setTimeout(() => {
          btn.textContent = cu.copy;
        }, 1600);
      }
    });
  });
}

export function wireLanguageSwitcher(_resolvedLang: SiteLang): void {
  const sel = document.getElementById("site-lang") as HTMLSelectElement | null;
  if (!sel) return;
  sel.value = getStoredSiteLang() ?? "auto";
  sel.addEventListener("change", () => {
    const v = sel.value;
    if (v === "auto") {
      setStoredSiteLang("clear");
      location.reload();
      return;
    }
    if (v === "en" || v === "es" || v === "de" || v === "fr") {
      setStoredSiteLang(v);
      location.reload();
    }
  });
}
