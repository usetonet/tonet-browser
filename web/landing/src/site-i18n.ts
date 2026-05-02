/** Landing + docs: English-first HTML; runtime locale from `navigator.language` or user override. */

import DOMPurify from "isomorphic-dompurify";
import type { DetectedOS } from "./detect-os";

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

export function detectedOsLine(lang: SiteLang, os: DetectedOS): string {
  const m = {
    en: {
      windows: "Detected: Windows — web installer plus optional MSI/Inno links.",
      linux: "Detected: Linux — web installer, .deb, and build commands.",
      macos: "Detected: macOS — web installer and local build steps.",
      unknown: "Could not detect your OS — defaulting to Windows.",
    },
    es: {
      windows: "Detectado: Windows — instalador web y enlaces MSI/Inno opcionales.",
      linux: "Detectado: Linux — instalador web, .deb y comandos de compilación.",
      macos: "Detectado: macOS — instalador web y compilación local.",
      unknown: "No pudimos detectar el SO — mostrando Windows por defecto.",
    },
    de: {
      windows: "Erkannt: Windows — Web-Installer plus optionale MSI/Inno-Links.",
      linux: "Erkannt: Linux — Web-Installer, .deb und Build-Befehle.",
      macos: "Erkannt: macOS — Web-Installer und lokaler Build.",
      unknown: "Betriebssystem unbekannt — Standard: Windows.",
    },
    fr: {
      windows: "Détecté : Windows — installateur web et liens MSI/Inno optionnels.",
      linux: "Détecté : Linux — installateur web, .deb et compilation.",
      macos: "Détecté : macOS — installateur web et build local.",
      unknown: "Système inconnu — affichage Windows par défaut.",
    },
  }[lang];
  return m[os];
}

export function versionPillPrefix(lang: SiteLang): string {
  const map: Record<SiteLang, string> = {
    en: "Latest release:",
    es: "Última versión:",
    de: "Aktuelle Version:",
    fr: "Dernière version :",
  };
  return map[lang];
}

interface LandingStrings {
  metaDescription: string;
  title: string;
  /** Dedicated `/download.html` page */
  pageTitleDownload: string;
  metaDescriptionDownload: string;
  /** `/guide.html` — non-technical help */
  pageTitleGuide: string;
  metaDescriptionGuide: string;
  navAria: string;
  /** Navbar repo link fallback when stars unavailable or zero */
  navGithub: string;
  navDownload: string;
  navFeatures: string;
  navGuide: string;
  navHandbook: string;
  navDocs: string;
  /** Top-level + “More” dropdown (Brave-style) */
  navMore: string;
  navRoadmap: string;
  navCompare: string;
  /** Small caps label above links inside the dropdown */
  navDropdownExplore: string;
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
  winSetup: string;
  winMsi: string;
  winExe: string;
  panelLinuxTitle: string;
  panelLinuxP1: string;
  linuxSetup: string;
  linuxDeb: string;
  linuxH4src: string;
  linuxH4user: string;
  panelMacTitle: string;
  panelMacP1: string;
  macSetup: string;
  /** Brave-style footer */
  footerColExplore: string;
  footerColLegal: string;
  footerColProject: string;
  footerColContact: string;
  footerLinkFeatures: string;
  footerLinkDownload: string;
  footerLinkGuide: string;
  footerLinkDocs: string;
  footerLinkCompare: string;
  footerLinkRoadmap: string;
  footerLinkPrivacy: string;
  footerLinkTerms: string;
  footerLinkAbout: string;
  footerLinkGithub: string;
  footerContactHtml: string;
  footerCopyright: string;
  langSwitcher: string;
  /** Home-only sections (beyond hero + first feature grid) */
  homeBannerTitle: string;
  homeBannerLead: string;
  homeExploreTitle: string;
  homeExploreLead: string;
  diffSectionTitle: string;
  diffSectionLead: string;
  kpi1Label: string;
  kpi1Strong: string;
  kpi2Label: string;
  kpi2Strong: string;
  kpi3Label: string;
  kpi3Strong: string;
  /** Hero KPI card supporting line (Brave-style) */
  kpi1Hint: string;
  kpi2Hint: string;
  kpi3Hint: string;
  /** Caption under scroll-pinned browser showcase */
  homeScrollFoot: string;
  /** Final lockup between Tonet mark and Servo mark */
  scrollPoweredBy: string;
  homeScrollAria: string;
  homeScrollHint: string;
  c5t: string;
  c5p: string;
  c6t: string;
  c6p: string;
  c7t: string;
  c7p: string;
  c8t: string;
  c8p: string;
  d1t: string;
  d1p: string;
  d2t: string;
  d2p: string;
  d3t: string;
  d3p: string;
  /** Home CTA strip */
  homeGoToDownloads: string;
  homeLinkGuide: string;
  homeLinkDocs: string;
  homeLinkCompare: string;
  homeLinkRoadmap: string;
  langOptAuto: string;
  langOptEn: string;
  langOptEs: string;
  langOptDe: string;
  langOptFr: string;
  /** Download page extras */
  downloadSectionTitle: string;
  downloadSectionLead: string;
  downloadHeroCta: string;
  downloadModalTitle: string;
  downloadModalLead: string;
  downloadModalStep1: string;
  downloadModalStep2: string;
  downloadModalStep3: string;
  downloadModalRetryPrefix: string;
  downloadModalRetryLink: string;
  downloadModalHelp: string;
  modalCloseLabel: string;
  channelLabel: string;
  channelStable: string;
  channelDev: string;
  channelSpecific: string;
  versionLabel: string;
  versionStableSuffix: string;
  versionPreviewSuffix: string;
  channelHintStable: string;
  channelHintDev: string;
  channelHintDevNone: string;
  channelHintSpecific: string;
}

const landing: Record<SiteLang, LandingStrings> = {
  en: {
    metaDescription:
      "Tonet — a lightweight browser built on the Servo engine. Fast, intentional, minimal. Downloads for Windows, Linux, and docs.",
    title: "Tonet — Browse light",
    pageTitleDownload: "Download Tonet — installers & formats",
    metaDescriptionDownload:
      "Download Tonet for Windows, Linux, or macOS. Official CDN builds: EXE, MSI, DEB, and release channels.",
    pageTitleGuide: "Using Tonet — quick start for everyone",
    metaDescriptionGuide:
      "Plain-language help: install Tonet, check for updates, and use settings—without developer jargon.",
    navAria: "Main",
    navGithub: "GitHub",
    navDownload: "Download",
    navFeatures: "Features",
    navGuide: "Using Tonet",
    navHandbook: "Handbook",
    navDocs: "Technical docs",
    navMore: "More",
    navRoadmap: "Roadmap",
    navCompare: "Compare",
    navDropdownExplore: "Explore",
    heroTitle: "Browse without the weight.<br />Push back on web bloat.",
    heroLead:
      "Tonet is built around the Servo rendering engine and a minimal shell—speed, clarity, and intentional limits. Not Blink, WebKit, or CEF — you control what comes in.",
    heroDownload: "Download Tonet",
    heroDocs: "Quick start",
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
      "We highlight the option that matches your system. You can always switch OS tabs manually.",
    panelWinTitle: "Windows",
    panelWinP1:
      "Primary download is the CI-built Inno Setup installer (EXE) for the version embedded on this site. MSI and other formats are linked below.",
    panelWinFoot:
      "On the CDN, stable builds use short names (<code>Tonet-Setup.exe</code>, <code>Tonet-x64.msi</code>) that always point to the latest stable release. Choose <strong>Specific version</strong> to download a particular build.",
    winSetup: "Download Tonet (Windows)",
    winMsi: "MSI (x64)",
    winExe: "Inno Setup EXE (x64)",
    panelLinuxTitle: "Linux",
    panelLinuxP1:
      "Primary download is the CI-built <code>.deb</code> for amd64 (Debian/Ubuntu) matching the embedded version. Other layouts and source builds are below.",
    linuxSetup: "Download Tonet (Linux)",
    linuxDeb: "Debian package (.deb)",
    linuxH4src: "Build from the repository",
    linuxH4user: "Per-user install (desktop + binary)",
    panelMacTitle: "macOS",
    panelMacP1:
      "macOS binaries (tarball or TonetSetup) appear on GitHub Releases when published for a tag. Until then, build from source below. A signed <code>.app</code> bundle is on the roadmap.",
    macSetup: "GitHub releases (macOS)",
    footerColExplore: "Explore",
    footerColLegal: "Legal",
    footerColProject: "Project",
    footerColContact: "Contact",
    footerLinkFeatures: "Features",
    footerLinkDownload: "Download",
    footerLinkGuide: "Using Tonet",
    footerLinkDocs: "Technical docs",
    footerLinkCompare: "Compare",
    footerLinkRoadmap: "Roadmap",
    footerLinkPrivacy: "Privacy Policy",
    footerLinkTerms: "Terms of Use",
    footerLinkAbout: "About",
    footerLinkGithub: "GitHub",
    footerContactHtml:
      'Questions about this site or Tonet? Use <a class="text-tonet-link" href="https://github.com/usetonet/tonet-browser/issues">GitHub Issues</a>.',
    footerCopyright:
      "© 2026 Tonet · usetonet.com · Open source · Servo-based browser under active development",
    homeBannerTitle: "Ready to install?",
    homeBannerLead:
      "Choose your OS and format on the download page—the installer downloads like any normal file.",
    homeExploreTitle: "Need more detail before installing?",
    homeExploreLead:
      "Plain-language help, technical references, positioning, and what ships next.",
    diffSectionTitle: "What makes Tonet different",
    diffSectionLead:
      "Built for teams that want explicit control over what runs in the browser, how updates arrive, and how much complexity enters the runtime.",
    kpi1Label: "Philosophy",
    kpi1Strong: "Servo + minimal shell",
    kpi2Label: "Primary channels",
    kpi2Strong: "Windows + Linux",
    kpi3Label: "Updates",
    kpi3Strong: "Manifest URL",
    kpi1Hint: "A thin shell on Servo — not another Blink bundle.",
    kpi2Hint: "Desktop builds you can mirror; channels you control.",
    kpi3Hint: "Your manifest URL on your CDN — no release-API roulette.",
    homeScrollFoot:
      "Tonet is designed for focus: a clear window onto the web, with infrastructure you can reason about — not an endless compatibility treadmill.",
    scrollPoweredBy: "powered by",
    homeScrollAria: "Scroll the page: layers combine into a Tonet and Servo lockup.",
    homeScrollHint: "Scroll to assemble",
    c5t: "Servo rendering",
    c5p:
      "Web content uses the Servo engine—not Chromium, WebKit, or CEF. Fewer opaque layers between your policy and the network.",
    c6t: "Strict resource bounds",
    c6p:
      "A 1 MB page ceiling keeps accidental megabyte payloads out of your session; failures are explicit instead of freezing the UI.",
    c7t: "Operational transparency",
    c7p:
      "You choose where installers and update manifests live. Update checks don’t need to call GitHub’s release API from the client.",
    c8t: "Privacy-minded defaults",
    c8p:
      "Minimal surface for trackers and third-party SDKs compared to browsers built for maximal compatibility.",
    d1t: "No black box engine",
    d1p:
      "The engine surface stays understandable and auditable, with the roadmap and quality gates published in the project docs.",
    d2t: "Predictable update flow",
    d2p:
      "Installers and manifests are served from the infrastructure you configure, so update checks aren’t tied to a vendor’s release API.",
    d3t: "Docs-first project",
    d3p:
      "Public documentation covers setup, architecture, use cases, comparisons, release notes, and implementation plans.",
    langSwitcher: "Site language",
    homeGoToDownloads: "Go to downloads",
    homeLinkGuide: "Using Tonet (plain language)",
    homeLinkDocs: "Technical documentation",
    homeLinkCompare: "Compare browsers",
    homeLinkRoadmap: "Roadmap",
    langOptAuto: "Auto (browser)",
    langOptEn: "English",
    langOptEs: "Español",
    langOptDe: "Deutsch",
    langOptFr: "Français",
    downloadSectionTitle: "All channels & formats",
    downloadSectionLead:
      "Advanced options: pick a release channel, switch OS tabs, or grab MSI / DEB / alternate builds.",
    downloadHeroCta: "Get Tonet",
    downloadModalTitle: "Almost there…",
    downloadModalLead: "You’re seconds away from running Tonet. Follow the steps while the installer downloads.",
    downloadModalStep1: "Download Tonet",
    downloadModalStep2: "Run the installer",
    downloadModalStep3: "Open Tonet from the shortcut",
    downloadModalRetryPrefix: "If the download didn’t start,",
    downloadModalRetryLink: "click here to try again",
    downloadModalHelp: "Need help getting started?",
    modalCloseLabel: "Close dialog",
    channelLabel: "Release channel",
    channelStable: "Latest stable (recommended)",
    channelDev: "Latest preview / development",
    channelSpecific: "Specific version…",
    versionLabel: "Version",
    versionStableSuffix: "stable",
    versionPreviewSuffix: "preview",
    channelHintStable:
      "Recommended production builds. Short filenames (Tonet-Setup.exe, tonet_amd64.deb, …) always point at the latest stable release.",
    channelHintDev:
      "Preview channel: may include unstable changes. Short filenames (e.g. Tonet-Setup-Preview.exe) track the latest preview.",
    channelHintDevNone: "No preview release is published on the CDN yet.",
    channelHintSpecific:
      "Versioned filenames on the CDN (e.g. Tonet-Setup-x.y.z-x64.exe). Pick the build you need.",
  },
  es: {
    metaDescription:
      "Tonet — navegador ligero basado en el motor Servo. Rápido, intencional y minimalista. Descargas para Windows, Linux y documentación.",
    title: "Tonet — Navega ligero",
    pageTitleDownload: "Descargar Tonet — instaladores y formatos",
    metaDescriptionDownload:
      "Descarga Tonet para Windows, Linux o macOS. Builds oficiales en el CDN: EXE, MSI, DEB y canales de release.",
    pageTitleGuide: "Uso de Tonet — guía sencilla",
    metaDescriptionGuide:
      "Ayuda en lenguaje claro: instalar Tonet, actualizar y usar ajustes, sin tecnicismos de desarrollo.",
    navAria: "Principal",
    navGithub: "GitHub",
    navDownload: "Descargar",
    navFeatures: "Características",
    navGuide: "Uso de Tonet",
    navHandbook: "Manual",
    navDocs: "Documentación técnica",
    navMore: "Más",
    navRoadmap: "Hoja de ruta",
    navCompare: "Comparar",
    navDropdownExplore: "Explorar",
    heroTitle: "Navega sin peso.<br />Rechaza la basura web.",
    heroLead:
      "Tonet está centrado en el motor de renderizado Servo y un shell mínimo: velocidad, claridad y límites deliberados. Sin Blink, WebKit ni CEF — tú controlas qué entra.",
    heroDownload: "Descargar Tonet",
    heroDocs: "Inicio rápido",
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
      "Detectamos tu sistema para resaltar la opción adecuada. Siempre puedes cambiar de pestaña de SO manualmente.",
    panelWinTitle: "Windows",
    panelWinP1:
      "La descarga principal es el instalador Inno Setup (EXE) generado en CI para la versión incrustada en esta web. El MSI y otros formatos están enlazados abajo.",
    panelWinFoot:
      "En el CDN, la rama estable usa nombres cortos (<code>Tonet-Setup.exe</code>, <code>Tonet-x64.msi</code>) que siempre apuntan a la última versión estable. Elige <strong>Versión concreta</strong> para bajar un build exacto.",
    winSetup: "Descargar Tonet (Windows)",
    winMsi: "MSI (x64)",
    winExe: "EXE Inno Setup (x64)",
    panelLinuxTitle: "Linux",
    panelLinuxP1:
      "La descarga principal es el <code>.deb</code> para amd64 (Debian/Ubuntu) generado en CI para la versión incrustada. Otros despliegues y compilar desde fuente están abajo.",
    linuxSetup: "Descargar Tonet (Linux)",
    linuxDeb: "Paquete Debian (.deb)",
    linuxH4src: "Compilar desde el repositorio",
    linuxH4user: "Instalación de usuario (desktop + binario)",
    panelMacTitle: "macOS",
    panelMacP1:
      "Los binarios de macOS (tarball o TonetSetup) aparecen en GitHub Releases cuando el tag los incluye. Hasta entonces, compila desde fuente abajo. Un <code>.app</code> firmado está en la hoja de ruta.",
    macSetup: "Releases en GitHub (macOS)",
    homeBannerTitle: "¿Listo para instalar?",
    homeBannerLead:
      "Elige tu sistema y formato en la página de descargas: la descarga se comporta como cualquier archivo.",
    homeExploreTitle: "¿Más detalle antes de instalar?",
    homeExploreLead:
      "Ayuda sencilla, referencias técnicas, comparativa y próximos pasos.",
    diffSectionTitle: "Qué distingue a Tonet",
    diffSectionLead:
      "Pensado para equipos que quieren control explícito sobre qué se ejecuta en el navegador, cómo llegan las actualizaciones y cuánta complejidad entra en tiempo de ejecución.",
    kpi1Label: "Filosofía",
    kpi1Strong: "Servo + shell mínima",
    kpi2Label: "Canales",
    kpi2Strong: "Windows + Linux",
    kpi3Label: "Actualizaciones",
    kpi3Strong: "URL del manifiesto",
    kpi1Hint: "Capa fina sobre Servo — no otro paquete tipo Blink.",
    kpi2Hint: "Builds de escritorio que puedes replicar; canales bajo tu control.",
    kpi3Hint: "Tu URL de manifiesto en tu CDN — sin ruleta de API de releases.",
    homeScrollFoot:
      "Tonet está pensado para el foco: una ventana clara a la web, con infraestructura que puedes entender — no una cinta interminable de compatibilidad.",
    scrollPoweredBy: "impulsado por",
    homeScrollAria: "Desplázate: las capas forman el bloque Tonet y Servo.",
    homeScrollHint: "*scrollea para continuar",
    c5t: "Renderizado Servo",
    c5p:
      "El contenido web usa el motor Servo: sin Chromium, WebKit ni CEF. Menos capas opacas entre tu política y la red.",
    c6t: "Límites de recursos",
    c6p:
      "Un techo de ~1 MB evita megabytes accidentales; los fallos son claros en lugar de congelar la interfaz.",
    c7t: "Transparencia operativa",
    c7p:
      "Tú decides dónde viven los instaladores y el manifiesto. Las comprobaciones de actualización no tienen que llamar a la API de releases de GitHub en el cliente.",
    c8t: "Privacidad por defecto",
    c8p:
      "Menos superficie para rastreadores y SDKs frente a navegadores centrados en compatibilidad máxima.",
    d1t: "Pila Servo auditable",
    d1p:
      "Servo y la hoja de ruta de Tonet están visibles en el repositorio, con barreras de calidad públicas.",
    d2t: "Flujo de actualización predecible",
    d2p:
      "Los instaladores y manifiestos se sirven desde la infraestructura que configures; las actualizaciones no quedan atadas a la API de un proveedor.",
    d3t: "Proyecto con documentación",
    d3p:
      "Documentación pública: instalación, arquitectura, casos de uso, comparativas y notas de versión.",
    footerColExplore: "Explorar",
    footerColLegal: "Legal",
    footerColProject: "Proyecto",
    footerColContact: "Contacto",
    footerLinkFeatures: "Funciones",
    footerLinkDownload: "Descargas",
    footerLinkGuide: "Uso de Tonet",
    footerLinkDocs: "Documentación técnica",
    footerLinkCompare: "Comparar",
    footerLinkRoadmap: "Hoja de ruta",
    footerLinkPrivacy: "Política de privacidad",
    footerLinkTerms: "Condiciones de uso",
    footerLinkAbout: "Acerca de",
    footerLinkGithub: "GitHub",
    footerContactHtml:
      '¿Dudas sobre esta web o Tonet? Usa <a class="text-tonet-link" href="https://github.com/usetonet/tonet-browser/issues">GitHub Issues</a>.',
    footerCopyright:
      "© 2026 Tonet · usetonet.com · Código abierto · Navegador basado en Servo en desarrollo activo",
    langSwitcher: "Idioma del sitio",
    homeGoToDownloads: "Ir a descargas",
    homeLinkGuide: "Uso de Tonet (lenguaje claro)",
    homeLinkDocs: "Documentación técnica",
    homeLinkCompare: "Comparar navegadores",
    homeLinkRoadmap: "Hoja de ruta",
    langOptAuto: "Automático (navegador)",
    langOptEn: "English",
    langOptEs: "Español",
    langOptDe: "Deutsch",
    langOptFr: "Français",
    downloadSectionTitle: "Todos los canales y formatos",
    downloadSectionLead:
      "Opciones avanzadas: canal de release, pestañas de SO o MSI / DEB / builds alternativos.",
    downloadHeroCta: "Obtener Tonet",
    downloadModalTitle: "Casi listo…",
    downloadModalLead: "Faltan segundos para usar Tonet. Sigue los pasos mientras se descarga el instalador.",
    downloadModalStep1: "Descargar Tonet",
    downloadModalStep2: "Ejecutar el instalador",
    downloadModalStep3: "Abrir Tonet desde el acceso directo",
    downloadModalRetryPrefix: "Si la descarga no empezó,",
    downloadModalRetryLink: "pulsa aquí para reintentar",
    downloadModalHelp: "¿Necesitas ayuda para empezar?",
    modalCloseLabel: "Cerrar",
    channelLabel: "Canal de release",
    channelStable: "Última estable (recomendado)",
    channelDev: "Última preview / desarrollo",
    channelSpecific: "Versión concreta…",
    versionLabel: "Versión",
    versionStableSuffix: "estable",
    versionPreviewSuffix: "preview",
    channelHintStable:
      "Builds de producción recomendados. Los nombres cortos siempre apuntan a la última versión estable.",
    channelHintDev:
      "Canal preview: puede incluir cambios inestables. Los nombres cortos siguen la última preview.",
    channelHintDevNone: "Aún no hay preview publicada en el CDN.",
    channelHintSpecific:
      "Nombres de archivo versionados en el CDN. Elige el build que necesites.",
  },
  de: {
    metaDescription:
      "Tonet — ein leichter Browser auf Basis der Servo-Engine. Schnell, bewusst, minimal. Downloads für Windows, Linux und Dokumentation.",
    title: "Tonet — Leicht surfen",
    pageTitleDownload: "Tonet herunterladen — Installer und Formate",
    metaDescriptionDownload:
      "Tonet für Windows, Linux oder macOS herunterladen. Offizielle CDN-Builds: EXE, MSI, DEB und Release-Kanäle.",
    pageTitleGuide: "Tonet nutzen — einfache Kurzanleitung",
    metaDescriptionGuide:
      "Klartext-Hilfe: Installation, Updates und Einstellungen — ohne Entwicklerjargon.",
    navAria: "Hauptnavigation",
    navGithub: "GitHub",
    navDownload: "Download",
    navFeatures: "Funktionen",
    navGuide: "Tonet nutzen",
    navHandbook: "Handbuch",
    navDocs: "Technische Docs",
    navMore: "Mehr",
    navRoadmap: "Roadmap",
    navCompare: "Vergleich",
    navDropdownExplore: "Entdecken",
    heroTitle: "Surfen ohne Ballast.<br />Web-Bloat zurückweisen.",
    heroLead:
      "Tonet nutzt die Servo-Rendering-Engine und eine schlanke Shell: Geschwindigkeit, Klarheit und bewusste Grenzen. Kein Blink, WebKit oder CEF — Sie entscheiden, was reinkommt.",
    heroDownload: "Tonet herunterladen",
    heroDocs: "Schnellstart",
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
    panelWinP1:
      "Haupt-Download: Inno-Setup-Installer (EXE) aus der CI-Pipeline für die auf dieser Seite eingebettete Version. MSI und weitere Formate sind unten verlinkt.",
    panelWinFoot:
      "Im CDN nutzen stabile Builds kurze Dateinamen (<code>Tonet-Setup.exe</code>, <code>Tonet-x64.msi</code>), die stets die aktuelle stabile Version referenzieren. Wähle <strong>Bestimmte Version</strong> für einen konkreten Build.",
    winSetup: "Tonet herunterladen (Windows)",
    winMsi: "MSI (x64)",
    winExe: "Inno-Setup-EXE (x64)",
    panelLinuxTitle: "Linux",
    panelLinuxP1:
      "Haupt-Download: <code>.deb</code> für amd64 (Debian/Ubuntu) aus der CI für die eingebettete Version. Weitere Varianten und Quellcode-Builds siehe unten.",
    linuxSetup: "Tonet herunterladen (Linux)",
    linuxDeb: "Debian-Paket (.deb)",
    linuxH4src: "Aus dem Repository bauen",
    linuxH4user: "Benutzerinstallation (Desktop + Binary)",
    panelMacTitle: "macOS",
    panelMacP1:
      "macOS-Binärdateien (Tarball oder TonetSetup) erscheinen auf GitHub Releases, wenn der Tag sie enthält. Bis dahin aus dem Quellcode bauen (unten). Ein signiertes <code>.app</code>-Bundle ist geplant.",
    macSetup: "GitHub-Releases (macOS)",
    homeBannerTitle: "Bereit zur Installation?",
    homeBannerLead:
      "OS und Format auf der Download-Seite wählen — der Download verhält sich wie jede normale Datei.",
    homeExploreTitle: "Mehr Details vor der Installation?",
    homeExploreLead:
      "Einfache Hilfe, technische Docs, Vergleich und Roadmap.",
    diffSectionTitle: "Was Tonet unterscheidet",
    diffSectionLead:
      "Für Teams, die steuern wollen, was im Browser läuft, wie Updates ankommen und wie viel Komplexität ins Laufzeitverhalten fließt.",
    kpi1Label: "Ansatz",
    kpi1Strong: "Servo + minimale Shell",
    kpi2Label: "Plattformen",
    kpi2Strong: "Windows + Linux",
    kpi3Label: "Updates",
    kpi3Strong: "Manifest-URL",
    kpi1Hint: "Dünne Shell auf Servo — kein weiteres Blink-Paket.",
    kpi2Hint: "Desktop-Builds zum Spiegeln; Kanäle unter deiner Kontrolle.",
    kpi3Hint: "Manifest-URL auf deinem CDN — ohne Release-API-Roulette.",
    homeScrollFoot:
      "Tonet ist auf Fokus ausgelegt: ein klares Fenster ins Web mit nachvollziehbarer Infrastruktur — kein endloser Kompatibilitätshamster.",
    scrollPoweredBy: "angetrieben von",
    homeScrollAria: "Scrollen: Ebenen fügen sich zu Tonet und Servo zusammen.",
    homeScrollHint: "Zum Zusammensetzen scrollen",
    c5t: "Servo-Rendering",
    c5p:
      "Webinhalte laufen über Servo — nicht Chromium, WebKit oder CEF. Weniger undurchsichtige Schichten zwischen Policy und Netz.",
    c6t: "Strenge Ressourcengrenzen",
    c6p:
      "1-MB-Seitenlimit hält große Lasten raus; Fehler sind sichtbar statt UI-Freeze.",
    c7t: "Operative Transparenz",
    c7p:
      "Sie entscheiden, wo Installer und Manifeste liegen. Update-Prüfungen müssen nicht die GitHub-Releases-API im Client aufrufen.",
    c8t: "Datenschutzbewusste Defaults",
    c8p:
      "Geringere Fläche für Tracker/Third-Party-SDKs als bei maximaler Web-Kompatibilität.",
    d1t: "Prüfbare Servo-Basis",
    d1p:
      "Servo und die Tonet-Roadmap sind im Repository nachvollziehbar, mit veröffentlichten Quality Gates.",
    d2t: "Vorhersehbarer Update-Flow",
    d2p:
      "Installer und Manifeste laufen über Infrastruktur, die Sie konfigurieren — nicht gebunden an eine Hersteller-Release-API.",
    d3t: "Dokumentationsfokus",
    d3p:
      "Öffentliche Docs: Setup, Architektur, Use Cases, Vergleich, Release Notes, Pläne.",
    footerColExplore: "Entdecken",
    footerColLegal: "Rechtliches",
    footerColProject: "Projekt",
    footerColContact: "Kontakt",
    footerLinkFeatures: "Funktionen",
    footerLinkDownload: "Download",
    footerLinkGuide: "Tonet nutzen",
    footerLinkDocs: "Technische Dokumentation",
    footerLinkCompare: "Vergleich",
    footerLinkRoadmap: "Roadmap",
    footerLinkPrivacy: "Datenschutzerklärung",
    footerLinkTerms: "Nutzungsbedingungen",
    footerLinkAbout: "Über Tonet",
    footerLinkGithub: "GitHub",
    footerContactHtml:
      'Fragen zu dieser Website oder Tonet? Nutzen Sie <a class="text-tonet-link" href="https://github.com/usetonet/tonet-browser/issues">GitHub Issues</a>.',
    footerCopyright:
      "© 2026 Tonet · usetonet.com · Open Source · Servo-basierter Browser in aktiver Entwicklung",
    langSwitcher: "Sprache der Website",
    homeGoToDownloads: "Zur Download-Seite",
    homeLinkGuide: "Tonet nutzen (einfach erklärt)",
    homeLinkDocs: "Technische Dokumentation",
    homeLinkCompare: "Browser vergleichen",
    homeLinkRoadmap: "Roadmap",
    langOptAuto: "Automatisch (Browser)",
    langOptEn: "English",
    langOptEs: "Español",
    langOptDe: "Deutsch",
    langOptFr: "Français",
    downloadSectionTitle: "Alle Kanäle & Formate",
    downloadSectionLead:
      "Erweitert: Release-Kanal, OS-Tabs oder MSI/DEB/Alternative Builds.",
    downloadHeroCta: "Tonet laden",
    downloadModalTitle: "Fast geschafft…",
    downloadModalLead: "Nur noch wenige Schritte. Folgen Sie der Anleitung, während der Installer lädt.",
    downloadModalStep1: "Tonet herunterladen",
    downloadModalStep2: "Installer ausführen",
    downloadModalStep3: "Tonet über die Verknüpfung starten",
    downloadModalRetryPrefix: "Wenn der Download nicht startet,",
    downloadModalRetryLink: "hier erneut versuchen",
    downloadModalHelp: "Hilfe beim Einstieg?",
    modalCloseLabel: "Schließen",
    channelLabel: "Release-Kanal",
    channelStable: "Letzte stabile Version (empfohlen)",
    channelDev: "Letzte Preview / Entwicklung",
    channelSpecific: "Bestimmte Version…",
    versionLabel: "Version",
    versionStableSuffix: "stabil",
    versionPreviewSuffix: "Preview",
    channelHintStable:
      "Empfohlene Produktions-Builds. Kurze Dateinamen zeigen stets die aktuelle stabile Version.",
    channelHintDev:
      "Preview-Kanal: kann instabile Änderungen enthalten. Kurze Namen folgen der aktuellen Preview.",
    channelHintDevNone: "Noch keine Preview auf dem CDN veröffentlicht.",
    channelHintSpecific:
      "Versionsbezogene Dateinamen auf dem CDN — wählen Sie den passenden Build.",
  },
  fr: {
    metaDescription:
      "Tonet — navigateur léger fondé sur le moteur Servo. Rapide, volontaire et minimal. Téléchargements Windows, Linux et documentation.",
    title: "Tonet — Naviguer léger",
    pageTitleDownload: "Télécharger Tonet — installateurs et formats",
    metaDescriptionDownload:
      "Téléchargez Tonet pour Windows, Linux ou macOS. Builds CDN officiels : EXE, MSI, DEB et canaux de publication.",
    pageTitleGuide: "Utiliser Tonet — guide simple",
    metaDescriptionGuide:
      "Aide en langage clair : installer Tonet, mises à jour et réglages, sans jargon technique.",
    navAria: "Principal",
    navGithub: "GitHub",
    navDownload: "Télécharger",
    navFeatures: "Fonctionnalités",
    navGuide: "Utiliser Tonet",
    navHandbook: "Manuel",
    navDocs: "Documentation technique",
    navMore: "Plus",
    navRoadmap: "Feuille de route",
    navCompare: "Comparer",
    navDropdownExplore: "Explorer",
    heroTitle: "Naviguez sans le poids.<br />Rejetez le superflu du web.",
    heroLead:
      "Tonet s’appuie sur le moteur de rendu Servo et une enveloppe minimale : vitesse, clarté et limites assumées. Pas de Blink, WebKit ni CEF — vous contrôlez ce qui entre.",
    heroDownload: "Télécharger Tonet",
    heroDocs: "Démarrage rapide",
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
    panelWinP1:
      "Téléchargement principal : installateur Inno Setup (EXE) produit en CI pour la version intégrée sur ce site. Le MSI et d’autres formats sont liés ci-dessous.",
    panelWinFoot:
      "Sur le CDN, les builds stables utilisent des noms courts (<code>Tonet-Setup.exe</code>, <code>Tonet-x64.msi</code>) vers la dernière version stable. Choisissez <strong>Version spécifique</strong> pour un build précis.",
    winSetup: "Télécharger Tonet (Windows)",
    winMsi: "MSI (x64)",
    winExe: "EXE Inno Setup (x64)",
    panelLinuxTitle: "Linux",
    panelLinuxP1:
      "Téléchargement principal : paquet <code>.deb</code> amd64 (Debian/Ubuntu) produit en CI pour la version intégrée. Autres méthodes et compilation ci-dessous.",
    linuxSetup: "Télécharger Tonet (Linux)",
    linuxDeb: "Paquet Debian (.deb)",
    linuxH4src: "Compiler depuis le dépôt",
    linuxH4user: "Installation utilisateur (bureau + binaire)",
    panelMacTitle: "macOS",
    panelMacP1:
      "Les binaires macOS (archive ou TonetSetup) figurent sur GitHub Releases lorsque le tag les publie. Sinon, compiler depuis les sources ci-dessous. Un <code>.app</code> signé est prévu.",
    macSetup: "Releases GitHub (macOS)",
    homeBannerTitle: "Prêt à installer ?",
    homeBannerLead:
      "Choisissez l’OS et le format sur la page Téléchargements — le téléchargement se comporte comme n’importe quel fichier.",
    homeExploreTitle: "Plus de détails avant d’installer ?",
    homeExploreLead:
      "Aide claire, docs techniques, comparaison et feuille de route.",
    diffSectionTitle: "Ce qui distingue Tonet",
    diffSectionLead:
      "Pour les équipes qui veulent maîtriser ce qui s’exécute dans le navigateur, comment arrivent les mises à jour et la complexité du runtime.",
    kpi1Label: "Approche",
    kpi1Strong: "Servo + enveloppe minimale",
    kpi2Label: "Canaux",
    kpi2Strong: "Windows + Linux",
    kpi3Label: "Mises à jour",
    kpi3Strong: "URL du manifeste",
    kpi1Hint: "Coque fine sur Servo — pas un autre bundle Blink.",
    kpi2Hint: "Installateurs desktop que vous pouvez miroiter ; canaux maîtrisés.",
    kpi3Hint: "Votre URL de manifeste sur votre CDN — sans roulette API releases.",
    homeScrollFoot:
      "Tonet est pensé pour la concentration : une fenêtre claire sur le web, avec une infrastructure que vous comprenez — pas un tapis roulant infini de compatibilité.",
    scrollPoweredBy: "propulsé par",
    homeScrollAria: "Faites défiler : les couches forment le bloc Tonet et Servo.",
    homeScrollHint: "Défilez pour assembler",
    c5t: "Rendu Servo",
    c5p:
      "Le contenu web passe par Servo — pas Chromium, WebKit ni CEF. Moins de couches opaques entre la politique et le réseau.",
    c6t: "Limites de ressources",
    c6p:
      "Plafond ~1 Mo pour éviter les charges énormes ; les échecs sont explicites.",
    c7t: "Transparence opérationnelle",
    c7p:
      "Vous choisissez où vivent les installateurs et le manifeste. Les vérifications de mise à jour n’ont pas besoin d’appeler l’API Releases GitHub côté client.",
    c8t: "Vie privée par défaut",
    c8p:
      "Surface réduite pour traqueurs et SDKs par rapport au navigateur « tout compatible ».",
    d1t: "Base Servo auditable",
    d1p:
      "Servo et la feuille de route Tonet restent visibles dans le dépôt, avec critères de qualité publiés.",
    d2t: "Flux de mise à jour maîtrisé",
    d2p:
      "Les installateurs et manifestes transitent par l’infrastructure que vous configurez — sans être liés à l’API d’un éditeur.",
    d3t: "Projet orienté documentation",
    d3p:
      "Docs publiques : installation, architecture, cas d’usage, comparaisons, notes de version, plans.",
    footerColExplore: "Explorer",
    footerColLegal: "Mentions légales",
    footerColProject: "Projet",
    footerColContact: "Contact",
    footerLinkFeatures: "Fonctionnalités",
    footerLinkDownload: "Téléchargements",
    footerLinkGuide: "Utiliser Tonet",
    footerLinkDocs: "Documentation technique",
    footerLinkCompare: "Comparer",
    footerLinkRoadmap: "Feuille de route",
    footerLinkPrivacy: "Politique de confidentialité",
    footerLinkTerms: "Conditions d’utilisation",
    footerLinkAbout: "À propos",
    footerLinkGithub: "GitHub",
    footerContactHtml:
      'Questions sur ce site ou Tonet ? Utilisez <a class="text-tonet-link" href="https://github.com/usetonet/tonet-browser/issues">GitHub Issues</a>.',
    footerCopyright:
      "© 2026 Tonet · usetonet.com · Open source · Navigateur fondé sur Servo en développement actif",
    langSwitcher: "Langue du site",
    homeGoToDownloads: "Aller aux téléchargements",
    homeLinkGuide: "Utiliser Tonet (langage simple)",
    homeLinkDocs: "Documentation technique",
    homeLinkCompare: "Comparer les navigateurs",
    homeLinkRoadmap: "Feuille de route",
    langOptAuto: "Auto (navigateur)",
    langOptEn: "English",
    langOptEs: "Español",
    langOptDe: "Deutsch",
    langOptFr: "Français",
    downloadSectionTitle: "Tous les canaux et formats",
    downloadSectionLead:
      "Options avancées : canal de publication, onglets OS ou MSI / DEB / builds alternatifs.",
    downloadHeroCta: "Obtenir Tonet",
    downloadModalTitle: "Presque terminé…",
    downloadModalLead:
      "Encore quelques secondes pour lancer Tonet. Suivez les étapes pendant le téléchargement de l’installeur.",
    downloadModalStep1: "Télécharger Tonet",
    downloadModalStep2: "Lancer l’installeur",
    downloadModalStep3: "Ouvrir Tonet depuis le raccourci",
    downloadModalRetryPrefix: "Si le téléchargement n’a pas démarré,",
    downloadModalRetryLink: "cliquez ici pour réessayer",
    downloadModalHelp: "Besoin d’aide pour commencer ?",
    modalCloseLabel: "Fermer",
    channelLabel: "Canal de publication",
    channelStable: "Dernière stable (recommandé)",
    channelDev: "Dernière preview / développement",
    channelSpecific: "Version spécifique…",
    versionLabel: "Version",
    versionStableSuffix: "stable",
    versionPreviewSuffix: "preview",
    channelHintStable:
      "Builds de production recommandés. Les noms courts pointent toujours vers la dernière version stable.",
    channelHintDev:
      "Canal preview : peut contenir des changements instables. Les noms courts suivent la dernière preview.",
    channelHintDevNone: "Aucune preview publiée sur le CDN pour l’instant.",
    channelHintSpecific:
      "Noms de fichiers versionnés sur le CDN. Choisissez le build dont vous avez besoin.",
  },
};

export function getLandingStrings(lang: SiteLang): LandingStrings {
  return landing[lang];
}

export function getNavLabels(lang: SiteLang): {
  ariaMain: string;
  github: string;
  download: string;
  roadmap: string;
  more: string;
  dropdownExplore: string;
  features: string;
  guide: string;
  handbook: string;
  technicalDocs: string;
  compare: string;
} {
  const L = landing[lang];
  return {
    ariaMain: L.navAria,
    github: L.navGithub,
    download: L.navDownload,
    roadmap: L.navRoadmap,
    more: L.navMore,
    dropdownExplore: L.navDropdownExplore,
    features: L.navFeatures,
    guide: L.navGuide,
    handbook: L.navHandbook,
    technicalDocs: L.navDocs,
    compare: L.navCompare,
  };
}

function setHtml(id: string, html: string): void {
  const el = document.getElementById(id);
  if (!el) return;
  el.innerHTML = DOMPurify.sanitize(html, {
    ALLOWED_TAGS: ["a", "strong", "code", "br", "em", "b", "i", "span"],
    ALLOWED_ATTR: ["href", "target", "rel", "class"],
  });
}

function setText(id: string, text: string): void {
  const el = document.getElementById(id);
  if (el) el.textContent = text;
}

/** Footer column links + copyright + language label (safe on any page). */
export function applyFooterLocale(lang: SiteLang): void {
  const L = landing[lang];
  setText("footer-col-explore", L.footerColExplore);
  setText("footer-col-legal", L.footerColLegal);
  setText("footer-col-project", L.footerColProject);
  setText("footer-col-contact", L.footerColContact);
  setText("footer-link-features", L.footerLinkFeatures);
  setText("footer-link-download", L.footerLinkDownload);
  setText("footer-link-guide", L.footerLinkGuide);
  setText("footer-link-docs", L.footerLinkDocs);
  setText("footer-link-compare", L.footerLinkCompare);
  setText("footer-link-roadmap", L.footerLinkRoadmap);
  setText("footer-link-privacy", L.footerLinkPrivacy);
  setText("footer-link-terms", L.footerLinkTerms);
  setText("footer-link-about", L.footerLinkAbout);
  setText("footer-link-github", L.footerLinkGithub);
  setText("footer-strip-terms", L.footerLinkTerms);
  setText("footer-strip-privacy", L.footerLinkPrivacy);
  setHtml("footer-contact-html", L.footerContactHtml);
  setText("footer-copyright", L.footerCopyright);
  const swLabel = document.getElementById("lang-switcher-label");
  if (swLabel) swLabel.textContent = L.langSwitcher;
  applyLanguageSelectLabels(lang);
}

export function applyLandingLocale(lang: SiteLang, opts?: { page?: "home" | "download" | "guide" }): void {
  const L = landing[lang];
  const page = opts?.page ?? "home";
  document.documentElement.lang = lang;
  document.title =
    page === "download" ? L.pageTitleDownload : page === "guide" ? L.pageTitleGuide : L.title;
  const meta = document.querySelector<HTMLMetaElement>('meta[name="description"]');
  if (meta) {
    meta.content =
      page === "download"
        ? L.metaDescriptionDownload
        : page === "guide"
          ? L.metaDescriptionGuide
          : L.metaDescription;
  }

  const nav = document.getElementById("site-nav-links");
  if (nav) nav.setAttribute("aria-label", L.navAria);

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
  setText("card-engine-t", L.c5t);
  setText("card-engine-p", L.c5p);
  setText("card-strict-t", L.c6t);
  setText("card-strict-p", L.c6p);
  setText("card-ops-t", L.c7t);
  setText("card-ops-p", L.c7p);
  setText("card-privacy-t", L.c8t);
  setText("card-privacy-p", L.c8p);
  setText("home-banner-title", L.homeBannerTitle);
  setText("home-banner-lead", L.homeBannerLead);
  setText("home-explore-title", L.homeExploreTitle);
  setText("home-explore-lead", L.homeExploreLead);
  setText("diff-section-title", L.diffSectionTitle);
  setText("diff-section-lead", L.diffSectionLead);
  setText("kpi-1-label", L.kpi1Label);
  setText("kpi-1-strong", L.kpi1Strong);
  setText("kpi-2-label", L.kpi2Label);
  setText("kpi-2-strong", L.kpi2Strong);
  setText("kpi-3-label", L.kpi3Label);
  setText("kpi-3-strong", L.kpi3Strong);
  setText("kpi-1-hint", L.kpi1Hint);
  setText("kpi-2-hint", L.kpi2Hint);
  setText("kpi-3-hint", L.kpi3Hint);
  setText("home-scroll-foot", L.homeScrollFoot);
  setText("diff-1-t", L.d1t);
  setText("diff-1-p", L.d1p);
  setText("diff-2-t", L.d2t);
  setText("diff-2-p", L.d2p);
  setText("diff-3-t", L.d3t);
  setText("diff-3-p", L.d3p);
  setText("download-title", L.downloadTitle);
  setText("download-lead", L.downloadLead);
  setText("panel-win-h3", L.panelWinTitle);
  setText("panel-win-p1", L.panelWinP1);
  setHtml("panel-win-foot", L.panelWinFoot);
  setText("win-setup", L.winSetup);
  setText("win-msi", L.winMsi);
  setText("win-exe", L.winExe);
  setText("panel-linux-h3", L.panelLinuxTitle);
  setHtml("panel-linux-p1", L.panelLinuxP1);
  setText("linux-setup", L.linuxSetup);
  setText("linux-deb", L.linuxDeb);
  setText("linux-h4-src", L.linuxH4src);
  setText("linux-h4-user", L.linuxH4user);
  setText("panel-mac-h3", L.panelMacTitle);
  setHtml("panel-mac-p1", L.panelMacP1);
  setText("mac-setup", L.macSetup);

  const goDl = document.getElementById("home-go-download");
  if (goDl) goDl.textContent = L.homeGoToDownloads;
  const lkGuide = document.getElementById("home-link-guide");
  if (lkGuide) lkGuide.textContent = L.homeLinkGuide;
  const lkDocs = document.getElementById("home-link-docs");
  if (lkDocs) lkDocs.textContent = L.homeLinkDocs;
  const lkCmp = document.getElementById("home-link-compare");
  if (lkCmp) lkCmp.textContent = L.homeLinkCompare;
  const lkRm = document.getElementById("home-link-roadmap");
  if (lkRm) lkRm.textContent = L.homeLinkRoadmap;

  const dlSecT = document.getElementById("download-section-title");
  if (dlSecT) dlSecT.textContent = L.downloadSectionTitle;
  const dlSecL = document.getElementById("download-section-lead");
  if (dlSecL) dlSecL.textContent = L.downloadSectionLead;
  const dlHero = document.getElementById("download-primary");
  if (dlHero) dlHero.textContent = L.downloadHeroCta;

  const chLab = document.querySelector<HTMLLabelElement>('label[for="channel-select"]');
  if (chLab) chLab.textContent = L.channelLabel;
  const verLab = document.querySelector<HTMLLabelElement>('label[for="version-select"]');
  if (verLab) verLab.textContent = L.versionLabel;

  const chSel = document.getElementById("channel-select") as HTMLSelectElement | null;
  if (chSel) {
    const o0 = chSel.querySelector('option[value="stable"]');
    const o1 = chSel.querySelector('option[value="development"]');
    const o2 = chSel.querySelector('option[value="specific"]');
    if (o0) o0.textContent = L.channelStable;
    if (o1) o1.textContent = L.channelDev;
    if (o2) o2.textContent = L.channelSpecific;
  }

  setText("modal-title", L.downloadModalTitle);
  const modalLead = document.getElementById("modal-lead");
  if (modalLead) modalLead.textContent = L.downloadModalLead;
  setText("modal-step-1", L.downloadModalStep1);
  setText("modal-step-2", L.downloadModalStep2);
  setText("modal-step-3", L.downloadModalStep3);
  const retryPre = document.getElementById("modal-retry-prefix");
  if (retryPre) retryPre.textContent = L.downloadModalRetryPrefix;
  const retryL = document.getElementById("modal-retry-link");
  if (retryL) retryL.textContent = L.downloadModalRetryLink;
  const modalHelp = document.getElementById("modal-help-link");
  if (modalHelp) modalHelp.textContent = L.downloadModalHelp;
  const closeBtn = document.getElementById("modal-close-btn");
  if (closeBtn) closeBtn.setAttribute("aria-label", L.modalCloseLabel);

  applyFooterLocale(lang);
}

export function applyLanguageSelectLabels(lang: SiteLang): void {
  const L = landing[lang];
  const sel = document.getElementById("site-lang") as HTMLSelectElement | null;
  if (!sel) return;
  for (const opt of sel.options) {
    if (opt.value === "auto") opt.textContent = L.langOptAuto;
    else if (opt.value === "en") opt.textContent = L.langOptEn;
    else if (opt.value === "es") opt.textContent = L.langOptEs;
    else if (opt.value === "de") opt.textContent = L.langOptDe;
    else if (opt.value === "fr") opt.textContent = L.langOptFr;
  }
}

export function getDownloadChannelHints(lang: SiteLang): {
  stable: string;
  development: string;
  developmentNone: string;
  specific: string;
  versionStable: string;
  versionPreview: string;
} {
  const L = landing[lang];
  return {
    stable: L.channelHintStable,
    development: L.channelHintDev,
    developmentNone: L.channelHintDevNone,
    specific: L.channelHintSpecific,
    versionStable: L.versionStableSuffix,
    versionPreview: L.versionPreviewSuffix,
  };
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
    title: "Technical documentation — Tonet",
    navHome: "Home",
    navDownload: "Download",
    h1: "Technical documentation",
    lead: "For packaging, signing, and maintainers. Everyday (non-technical) help: <a href=\"/guide.html\">Using Tonet</a>.",
    installH: "Installation",
    installP:
      "See the <a href=\"/download.html\">download page</a>. The site reads <code>version.json</code> (built from <code>crates/tonet</code>) and points download buttons to your configured CDN base URL. MSI/EXE/DEB/tarball links are generated from that path.",
    updatesH: "In-browser updates",
    updatesP:
      "Tonet checks your update manifest URL (it does not install binaries for you). By default it checks on startup. In <strong>Settings (⚙)</strong> you can choose:",
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
    title: "Documentación técnica — Tonet",
    navHome: "Inicio",
    navDownload: "Descargar",
    h1: "Documentación técnica",
    lead: "Para empaquetado, firma y mantenedores. Ayuda cotidiana (sencilla): <a href=\"/guide.html\">Uso de Tonet</a>.",
    installH: "Instalación",
    installP:
      "Consulta la <a href=\"/download.html\">página de descargas</a>. El sitio lee <code>version.json</code> (generado desde <code>crates/tonet</code>) y apunta los botones a la URL base de tu CDN. Los enlaces MSI/EXE/DEB/tarball se generan desde esa base.",
    updatesH: "Actualizaciones en el navegador",
    updatesP:
      "Tonet consulta tu URL de manifiesto de actualizaciones (sin instalar binarios por ti). Por defecto comprueba al iniciar. En <strong>Ajustes (⚙)</strong> puedes elegir:",
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
    title: "Technische Dokumentation — Tonet",
    navHome: "Start",
    navDownload: "Download",
    h1: "Technische Dokumentation",
    lead: "Für Packaging, Signatur und Betrieb. Einfache Hilfe für den Alltag: <a href=\"/guide.html\">Tonet nutzen</a>.",
    installH: "Installation",
    installP:
      "Siehe die <a href=\"/download.html\">Download-Seite</a>. Die Site liest <code>version.json</code> (aus <code>crates/tonet</code>) und erzeugt Download-Links über die konfigurierte CDN-Basis-URL.",
    updatesH: "Updates in der App",
    updatesP:
      "Tonet nutzt Ihre Update-Manifest-URL (installiert keine Binärdateien für Sie). Standard: Prüfung beim Start. Unter <strong>Einstellungen (⚙)</strong> wählen Sie:",
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
    title: "Documentation technique — Tonet",
    navHome: "Accueil",
    navDownload: "Télécharger",
    h1: "Documentation technique",
    lead: "Pour le packaging, la signature et l’exploitation. Aide du quotidien (simple) : <a href=\"/guide.html\">Utiliser Tonet</a>.",
    installH: "Installation",
    installP:
      "Voir la <a href=\"/download.html\">page Téléchargements</a>. Le site lit <code>version.json</code> (généré depuis <code>crates/tonet</code>) et génère les liens MSI/EXE/DEB/tarball depuis votre base CDN.",
    updatesH: "Mises à jour dans le navigateur",
    updatesP:
      "Tonet interroge votre URL de manifeste de mises à jour (sans installer les binaires à votre place). Par défaut : vérification au lancement. Dans <strong>Réglages (⚙)</strong> vous pouvez choisir :",
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
  setText("docs-h1", D.h1);
  setHtml("docs-lead", D.lead);
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

export function wireLanguageSwitcher(resolvedLang: SiteLang): void {
  applyLanguageSelectLabels(resolvedLang);
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
