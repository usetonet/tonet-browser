/**
 * Long-form page copy (roadmap, guide body, handbook, compare, docs sidebar).
 * Keeps site-i18n.ts focused on shared landing strings.
 */

import DOMPurify from "isomorphic-dompurify";
import type { SiteLang } from "../site-i18n";

function setText(id: string, text: string): void {
  const el = document.getElementById(id);
  if (el) el.textContent = text;
}

function setHtml(id: string, html: string): void {
  const el = document.getElementById(id);
  if (!el) return;
  el.innerHTML = DOMPurify.sanitize(html, {
    ALLOWED_TAGS: ["a", "strong", "code", "br", "em", "b", "i", "span"],
    ALLOWED_ATTR: ["href", "target", "rel", "class"],
  });
}

/* ——— Roadmap ——— */

const roadmap: Record<
  SiteLang,
  {
    title: string;
    lead: string;
    pNow: string;
    hShip: string;
    shipL1: string;
    shipL2: string;
    shipL3: string;
    pNext: string;
    hCompat: string;
    compL1: string;
    compL2: string;
    compL3: string;
    pLater: string;
    hDepth: string;
    depthL1: string;
    depthL2: string;
    depthL3: string;
    authH: string;
    authP: string;
    footerLink: string;
  }
> = {
  en: {
    title: "Roadmap",
    lead:
      "Phases mirror how engineering tracks work in public: quality gates in <code class=\"rounded bg-white/[0.06] px-1\">TONET_VISION.md</code>, CI on main, and release artifacts behind a manifest you control. Dates stay coarse — milestones ship when gates pass.",
    pNow: "Phase — Now",
    hShip: "Ship & operate",
    shipL1: "Stable installers (Windows MSI/Inno + Linux .deb) published via CI and manifest v2.",
    shipL2: "Servo integration smoke paths repeatable on Windows (LLVM/bindgen, ANGLE DLLs next to exe).",
    shipL3: "Landing and docs aligned with operator runbooks (this site).",
    pNext: "Phase — Next",
    hCompat: "Compatibility & shell",
    compL1: "Grow HTML/CSS coverage toward documented conformance milestones (engine checklist).",
    compL2: "Browser chrome: tabs, navigation, and settings parity with Tonet’s minimalist contract.",
    compL3: "Packaging parity (macOS signing track, optional AppImage evaluation).",
    pLater: "Phase — Later",
    hDepth: "Depth & scale",
    depthL1: "Performance budgets and measurable regressions in CI (beyond smoke).",
    depthL2: "Broader platform coverage where Servo and packaging support align.",
    depthL3: "Security assurance processes alongside feature growth.",
    authH: "Authoritative detail",
    authP:
      "Full milestones: <code class=\"rounded bg-white/[0.06] px-1\">TONET_VISION.md</code>. Integration discipline: <code class=\"rounded bg-white/[0.06] px-1\">docs/SERVO_INTEGRATION_CHECKLIST.md</code>.",
    footerLink: "Open full docs portal",
  },
  es: {
    title: "Hoja de ruta",
    lead:
      "Las fases reflejan el trabajo de ingeniería en abierto: barreras de calidad en <code class=\"rounded bg-white/[0.06] px-1\">TONET_VISION.md</code>, CI en main y artefactos de publicación detrás de un manifiesto bajo tu control. Sin fechas ficticias: cada hito llega al cumplir los gates.",
    pNow: "Fase — Ahora",
    hShip: "Publicar y operar",
    shipL1: "Instaladores estables (MSI/Inno en Windows + .deb en Linux) vía CI y manifiesto v2.",
    shipL2: "Rutas de humo de Servo repetibles en Windows (LLVM/bindgen, DLLs ANGLE junto al exe).",
    shipL3: "Landing y documentación alineadas con runbooks de operación (este sitio).",
    pNext: "Fase — Siguiente",
    hCompat: "Compatibilidad y shell",
    compL1: "Ampliar cobertura HTML/CSS hacia hitos de conformidad documentados (checklist del motor).",
    compL2: "Cromo del navegador: pestañas, navegación y ajustes coherentes con el contrato minimalista de Tonet.",
    compL3: "Paridad de empaquetado (firma en macOS, evaluación opcional de AppImage).",
    pLater: "Fase — Más adelante",
    hDepth: "Profundidad y escala",
    depthL1: "Presupuestos de rendimiento y regresiones medibles en CI (más allá del humo).",
    depthL2: "Más plataformas donde Servo y el empaquetado lo permitan.",
    depthL3: "Procesos de seguridad junto al crecimiento de funciones.",
    authH: "Detalle autoritativo",
    authP:
      "Hitos completos: <code class=\"rounded bg-white/[0.06] px-1\">TONET_VISION.md</code>. Disciplina de integración: <code class=\"rounded bg-white/[0.06] px-1\">docs/SERVO_INTEGRATION_CHECKLIST.md</code>.",
    footerLink: "Abrir portal de documentación",
  },
  de: {
    title: "Roadmap",
    lead:
      "Die Phasen spiegeln öffentliches Engineering ab: Qualitätsgates in <code class=\"rounded bg-white/[0.06] px-1\">TONET_VISION.md</code>, CI auf main und Release-Artefakte hinter einem Manifest unter Ihrer Kontrolle. Termine bleiben grob — Meilensteine liefern, wenn Gates grün sind.",
    pNow: "Phase — Jetzt",
    hShip: "Ausliefern & betreiben",
    shipL1: "Stabile Installer (Windows MSI/Inno + Linux .deb) via CI und Manifest v2.",
    shipL2: "Servo-Smoke-Pfade unter Windows wiederholbar (LLVM/bindgen, ANGLE-DLLs neben der exe).",
    shipL3: "Landing und Docs an Operator-Runbooks ausgerichtet (diese Site).",
    pNext: "Phase — Als Nächstes",
    hCompat: "Kompatibilität & Shell",
    compL1: "HTML/CSS-Abdeckung Richtung dokumentierte Konformitäts-Meilensteine (Engine-Checkliste).",
    compL2: "Browser-Chrome: Tabs, Navigation und Settings im minimalistischen Tonet-Vertrag.",
    compL3: "Packaging-Parität (macOS-Signatur, optionale AppImage-Evaluation).",
    pLater: "Phase — Später",
    hDepth: "Tiefe & Skalierung",
    depthL1: "Performance-Budgets und messbare Regressionen in CI (über Smoke hinaus).",
    depthL2: "Breitere Plattformen, wo Servo und Packaging mitziehen.",
    depthL3: "Security-Prozesse parallel zum Feature-Wachstum.",
    authH: "Maßgebliche Details",
    authP:
      "Volle Meilensteine: <code class=\"rounded bg-white/[0.06] px-1\">TONET_VISION.md</code>. Integrationsdisziplin: <code class=\"rounded bg-white/[0.06] px-1\">docs/SERVO_INTEGRATION_CHECKLIST.md</code>.",
    footerLink: "Zur vollen Dokumentation",
  },
  fr: {
    title: "Feuille de route",
    lead:
      "Les phases suivent l’ingénierie en public : garde-fous dans <code class=\"rounded bg-white/[0.06] px-1\">TONET_VISION.md</code>, CI sur main et artefacts de publication derrière un manifeste que vous maîtrisez. Pas de dates artificielles — les jalons arrivent quand les gates sont passés.",
    pNow: "Phase — Maintenant",
    hShip: "Publier et exploiter",
    shipL1: "Installateurs stables (MSI/Inno Windows + .deb Linux) via CI et manifeste v2.",
    shipL2: "Parcours de fumée Servo reproductibles sous Windows (LLVM/bindgen, DLL ANGLE à côté de l’exe).",
    shipL3: "Landing et documentation alignés sur les runbooks opérationnels (ce site).",
    pNext: "Phase — Ensuite",
    hCompat: "Compatibilité et shell",
    compL1: "Étendre la couverture HTML/CSS vers des jalons de conformité documentés (checklist moteur).",
    compL2: "Chrome navigateur : onglets, navigation et réglages cohérents avec le contrat minimaliste de Tonet.",
    compL3: "Parité d’empaquetage (signature macOS, évaluation AppImage optionnelle).",
    pLater: "Phase — Plus tard",
    hDepth: "Profondeur et échelle",
    depthL1: "Budgets de perf et régressions mesurables en CI (au-delà du smoke).",
    depthL2: "Plateformes plus larges lorsque Servo et l’empaquetage le permettent.",
    depthL3: "Processus de sécurité aux côtés des fonctionnalités.",
    authH: "Référence détaillée",
    authP:
      "Jalons complets : <code class=\"rounded bg-white/[0.06] px-1\">TONET_VISION.md</code>. Discipline d’intégration : <code class=\"rounded bg-white/[0.06] px-1\">docs/SERVO_INTEGRATION_CHECKLIST.md</code>.",
    footerLink: "Ouvrir la documentation complète",
  },
};

export function applyRoadmapLocale(lang: SiteLang): void {
  const R = roadmap[lang];
  setText("roadmap-h1", R.title);
  setHtml("roadmap-lead", R.lead);
  setText("roadmap-pnow-badge", R.pNow);
  setText("roadmap-ship-h", R.hShip);
  setText("roadmap-ship-l1", R.shipL1);
  setText("roadmap-ship-l2", R.shipL2);
  setText("roadmap-ship-l3", R.shipL3);
  setText("roadmap-pnext-badge", R.pNext);
  setText("roadmap-compat-h", R.hCompat);
  setText("roadmap-compat-l1", R.compL1);
  setText("roadmap-compat-l2", R.compL2);
  setText("roadmap-compat-l3", R.compL3);
  setText("roadmap-plater-badge", R.pLater);
  setText("roadmap-depth-h", R.hDepth);
  setText("roadmap-depth-l1", R.depthL1);
  setText("roadmap-depth-l2", R.depthL2);
  setText("roadmap-depth-l3", R.depthL3);
  setText("roadmap-auth-h", R.authH);
  setHtml("roadmap-auth-p", R.authP);
  setText("roadmap-footer-link", R.footerLink);
}

/* ——— Guide (plain-language page) ——— */

const guidePage: Record<
  SiteLang,
  {
    h1: string;
    lead: string;
    installH: string;
    installP: string;
    updatesH: string;
    updatesP: string;
    depthH: string;
    li1: string;
    li2: string;
    li3: string;
    back: string;
  }
> = {
  en: {
    h1: "Using Tonet",
    lead:
      "For everyday users—no packaging jargon. Developers who need signing and manifests should open <a class=\"text-tonet-link hover:underline\" href=\"/docs.html\">Technical documentation</a>.",
    installH: "Install Tonet",
    installP:
      "Go to <a class=\"text-tonet-link hover:underline\" href=\"/download.html\">Downloads</a>. Your OS tab is highlighted: use the main download button to fetch the installer; it behaves like any normal file download. When it finishes, run it like any other desktop app.",
    updatesH: "Stay up to date",
    updatesP:
      "Tonet checks your official update manifest. Open <strong>Settings</strong> (gear icon) and choose how often to check—on launch, every day, or only when you tap “Check now”. Tonet tells you when a build is ready; you decide when to download it.",
    depthH: "Need more depth?",
    li1:
      "<a class=\"text-tonet-link hover:underline\" href=\"/handbook.html\">Handbook</a> — product framing and extended topics.",
    li2:
      "<a class=\"text-tonet-link hover:underline\" href=\"/docs.html\">Technical docs</a> — signing, Debian packages, deployment.",
    li3:
      "<a class=\"text-tonet-link hover:underline\" href=\"/compare.html\">Compare</a> — how Tonet differs from mainstream browsers.",
    back: "← Back to home",
  },
  es: {
    h1: "Uso de Tonet",
    lead:
      "Para el día a día, sin tecnicismos de empaquetado. Quien necesite firma y manifiestos puede ir a <a class=\"text-tonet-link hover:underline\" href=\"/docs.html\">Documentación técnica</a>.",
    installH: "Instalar Tonet",
    installP:
      "Entra en <a class=\"text-tonet-link hover:underline\" href=\"/download.html\">Descargas</a>. Tu pestaña de SO queda resaltada: usa el botón principal de descarga; se comporta como cualquier descarga de archivo. Cuando termine, ejecútalo como cualquier app.",
    updatesH: "Mantente al día",
    updatesP:
      "Tonet consulta el manifiesto oficial de actualizaciones. En <strong>Ajustes</strong> (engranaje) elige la frecuencia: al iniciar, cada día o solo con «Comprobar ahora». Tonet avisa cuando haya build; tú decides cuándo descargar.",
    depthH: "¿Más detalle?",
    li1:
      "<a class=\"text-tonet-link hover:underline\" href=\"/handbook.html\">Manual</a> — enfoque de producto y temas ampliados.",
    li2:
      "<a class=\"text-tonet-link hover:underline\" href=\"/docs.html\">Docs técnicas</a> — firma, paquetes Debian, despliegue.",
    li3:
      "<a class=\"text-tonet-link hover:underline\" href=\"/compare.html\">Comparar</a> — cómo se diferencia Tonet del resto.",
    back: "← Volver al inicio",
  },
  de: {
    h1: "Tonet nutzen",
    lead:
      "Für den Alltag ohne Packaging-Jargon. Wer Signatur und Manifeste braucht, öffnet die <a class=\"text-tonet-link hover:underline\" href=\"/docs.html\">technische Dokumentation</a>.",
    installH: "Tonet installieren",
    installP:
      "Zur <a class=\"text-tonet-link hover:underline\" href=\"/download.html\">Download-Seite</a>. Dein OS-Reiter ist hervorgehoben: Haupt-Download-Button nutzen — wie jeder normale Dateidownload. Danach wie gewohnt ausführen.",
    updatesH: "Aktuell bleiben",
    updatesP:
      "Tonet prüft dein offizielles Update-Manifest. Unter <strong>Einstellungen</strong> (Zahnrad) wählst du die Häufigkeit: beim Start, täglich oder nur mit „Jetzt prüfen“. Tonet meldet neue Builds; du entscheidest über den Download.",
    depthH: "Mehr Tiefe?",
    li1:
      "<a class=\"text-tonet-link hover:underline\" href=\"/handbook.html\">Handbuch</a> — Produktbild und vertiefende Themen.",
    li2:
      "<a class=\"text-tonet-link hover:underline\" href=\"/docs.html\">Technische Docs</a> — Signatur, Debian-Pakete, Deployment.",
    li3:
      "<a class=\"text-tonet-link hover:underline\" href=\"/compare.html\">Vergleich</a> — wie sich Tonet von Mainstream-Browsern unterscheidet.",
    back: "← Zur Startseite",
  },
  fr: {
    h1: "Utiliser Tonet",
    lead:
      "Pour un usage quotidien, sans jargon d’empaquetage. Pour la signature et les manifestes, ouvrez la <a class=\"text-tonet-link hover:underline\" href=\"/docs.html\">documentation technique</a>.",
    installH: "Installer Tonet",
    installP:
      "Allez sur la page <a class=\"text-tonet-link hover:underline\" href=\"/download.html\">Téléchargements</a>. L’onglet OS est mis en avant : utilisez le bouton principal — comme tout téléchargement de fichier. Puis lancez l’installeur comme d’habitude.",
    updatesH: "Restez à jour",
    updatesP:
      "Tonet interroge le manifeste officiel. Dans <strong>Réglages</strong> (roue dentée), choisissez la fréquence : au lancement, chaque jour ou seulement avec « Vérifier maintenant ». Tonet signale les builds ; vous choisissez quand les télécharger.",
    depthH: "Besoin de plus de détail ?",
    li1:
      "<a class=\"text-tonet-link hover:underline\" href=\"/handbook.html\">Manuel</a> — cadrage produit et sujets étendus.",
    li2:
      "<a class=\"text-tonet-link hover:underline\" href=\"/docs.html\">Documentation technique</a> — signature, paquets Debian, déploiement.",
    li3:
      "<a class=\"text-tonet-link hover:underline\" href=\"/compare.html\">Comparer</a> — différences avec les navigateurs grand public.",
    back: "← Retour à l’accueil",
  },
};

export function applyGuidePageLocale(lang: SiteLang): void {
  const G = guidePage[lang];
  setText("guide-h1", G.h1);
  setHtml("guide-lead", G.lead);
  setText("guide-install-h", G.installH);
  setHtml("guide-install-p", G.installP);
  setText("guide-updates-h", G.updatesH);
  setHtml("guide-updates-p", G.updatesP);
  setText("guide-depth-h", G.depthH);
  setHtml("guide-li1", G.li1);
  setHtml("guide-li2", G.li2);
  setHtml("guide-li3", G.li3);
  setHtml("guide-back", G.back);
}

/* ——— Handbook ——— */

const handbookPage: Record<
  SiteLang,
  {
    h1: string;
    lead: string;
    servoH: string;
    servoP1: string;
    servoP2: string;
    servoP3: string;
    useH: string;
    useL1: string;
    useL2: string;
    useL3: string;
    useL4: string;
    utilH: string;
    utilL1: string;
    utilL2: string;
    utilL3: string;
    utilL4: string;
    recentH: string;
    recentP: string;
    futureH: string;
    futL1: string;
    futL2: string;
    futL3: string;
    futL4: string;
    compareH: string;
    compareP: string;
    footer: string;
  }
> = {
  en: {
    h1: "Public handbook",
    lead:
      "Expanded project documentation: use cases, internal pages, product evolution, and implementation tracks.",
    servoH: "Servo and Tonet",
    servoP1:
      "<strong>Servo</strong> is a free and open-source <strong>web rendering engine</strong> written in <strong>Rust</strong>. It is designed for <strong>embedding</strong> (WebView-style APIs), <strong>parallel</strong> layout and paint, and <strong>memory safety</strong> by construction. The project explains its goals and downloads on the official site: <a class=\"text-tonet-link hover:underline\" href=\"https://servo.org/\" target=\"_blank\" rel=\"noopener noreferrer\">servo.org</a> — a lightweight, embeddable alternative for applications that need real web technology without shipping an entire Chromium-sized platform.",
    servoP2:
      "<strong>History (short):</strong> Servo began at <strong>Mozilla</strong> around 2012 as a research engine; the <strong>Rust</strong> language grew up alongside it. After Mozilla restructured R&amp;D, the community kept Servo alive; governance now sits under <strong>Linux Foundation Europe</strong>, with open rules and a public Technical Steering Committee. Servo ships as Rust crates (including on <strong>crates.io</strong>) and targets desktop, mobile, and embedded scenarios. For a neutral overview, see Wikipedia’s article <a class=\"text-tonet-link hover:underline\" href=\"https://en.wikipedia.org/wiki/Servo_(software)\" target=\"_blank\" rel=\"noopener noreferrer\">Servo (software)</a>.",
    servoP3:
      "<strong>Why Tonet uses Servo:</strong> Tonet is a <strong>minimal browser shell</strong> that embeds Servo to render <strong>real web pages in-process</strong>. We want an engine surface that teams can <strong>audit and reason about</strong> — without the full weight and opacity of a Chromium-class embedding stack, and with room for <strong>policy-first</strong> browsing and explicit resource limits. Servo’s direction — Rust, modularity, embedding — matches that product bet. Official Servo home: <a class=\"text-tonet-link hover:underline\" href=\"https://servo.org/\" target=\"_blank\" rel=\"noopener noreferrer\">servo.org</a>.",
    useH: "Use cases",
    useL1: "Low-overhead browsing where predictable behavior matters.",
    useL2: "Education and R&D where browser internals must stay transparent.",
    useL3: "Deployments that require controlled artifacts and update manifests.",
    useL4: "Security-conscious setups that prefer explicit policy-driven behavior.",
    utilH: "Utilities and internal pages",
    utilL1: "<code class=\"rounded bg-white/[0.06] px-1\">tonet://settings</code>: language, theme, update policy, system toggles.",
    utilL2: "<code class=\"rounded bg-white/[0.06] px-1\">tonet://downloads</code>: page fetch and local save activity.",
    utilL3: "<code class=\"rounded bg-white/[0.06] px-1\">tonet://history</code>: visit timeline and data management hooks.",
    utilL4: "<code class=\"rounded bg-white/[0.06] px-1\">tonet://new-tab</code>: shortcuts and search integration.",
    recentH: "Recent changes",
    recentP:
      "Recent work focuses on Servo integration, policy surfaces, update flow hardening, and CI/doc reproducibility. Version notes follow each release.",
    futureH: "Future implementation tracks",
    futL1: "Broader selector/layout support for compatibility.",
    futL2: "Expanded cross-platform packaging and installer parity.",
    futL3: "Richer test matrices for conformance and regressions.",
    futL4: "Incremental performance and memory budget enforcement.",
    compareH: "Comparison notes",
    compareP:
      "Tonet is narrower than mature browsers like Chrome or Brave: clarity and operational control first. See <a class=\"text-tonet-link hover:underline\" href=\"/compare.html\">Compare</a> for a concise summary.",
    footer: "Back to docs portal",
  },
  es: {
    h1: "Manual público",
    lead:
      "Documentación ampliada del proyecto: casos de uso, páginas internas, evolución del producto y líneas de implementación.",
    servoH: "Servo y Tonet",
    servoP1:
      "<strong>Servo</strong> es un <strong>motor de renderizado web</strong> libre y de código abierto escrito en <strong>Rust</strong>. Está pensado para <strong>embeberse</strong> (API estilo WebView), paralelizar diseño y pintado y ofrecer <strong>seguridad de memoria</strong>. La documentación y los objetivos del proyecto están en <a class=\"text-tonet-link hover:underline\" href=\"https://servo.org/\" target=\"_blank\" rel=\"noopener noreferrer\">servo.org</a>.",
    servoP2:
      "<strong>Historia (breve):</strong> Servo nació en <strong>Mozilla</strong> hacia 2012 como motor de investigación; <strong>Rust</strong> creció en paralelo. Tras la reorganización de Mozilla, la comunidad mantuvo Servo; la gobernanza pasó a <strong>Linux Foundation Europe</strong>, con reglas abiertas. Los crates se publican entre otros en <strong>crates.io</strong>. Visión general neutral: Wikipedia <a class=\"text-tonet-link hover:underline\" href=\"https://en.wikipedia.org/wiki/Servo_(software)\" target=\"_blank\" rel=\"noopener noreferrer\">Servo (software)</a> (artículo en inglés).",
    servoP3:
      "<strong>Por qué Tonet usa Servo:</strong> Tonet es un <strong>shell mínimo</strong> que integra Servo para renderizar <strong>páginas web reales en proceso</strong>. Buscamos una superficie de motor <strong>auditable</strong>, sin la opacidad de un stack tipo Chromium, con espacio para políticas claras y límites de recursos. Sitio oficial: <a class=\"text-tonet-link hover:underline\" href=\"https://servo.org/\" target=\"_blank\" rel=\"noopener noreferrer\">servo.org</a>.",
    useH: "Casos de uso",
    useL1: "Navegación ligera donde importa un comportamiento predecible.",
    useL2: "Educación e I+D donde los internals del navegador deben ser transparentes.",
    useL3: "Despliegues que requieren artefactos y manifiestos de actualización controlados.",
    useL4: "Entornos sensibles a la seguridad con políticas explícitas.",
    utilH: "Utilidades y páginas internas",
    utilL1: "<code class=\"rounded bg-white/[0.06] px-1\">tonet://settings</code>: idioma, tema, política de actualizaciones, interruptores del sistema.",
    utilL2: "<code class=\"rounded bg-white/[0.06] px-1\">tonet://downloads</code>: actividad de descarga y guardado local.",
    utilL3: "<code class=\"rounded bg-white/[0.06] px-1\">tonet://history</code>: historial de visitas y gestión de datos.",
    utilL4: "<code class=\"rounded bg-white/[0.06] px-1\">tonet://new-tab</code>: atajos e integración de búsqueda.",
    recentH: "Cambios recientes",
    recentP:
      "El trabajo reciente se centra en integración Servo, superficies de política, endurecimiento del flujo de actualizaciones y reproducibilidad en CI/documentación. Las notas de versión siguen a cada release.",
    futureH: "Líneas de implementación futuras",
    futL1: "Mayor soporte de selectores y maquetado para compatibilidad.",
    futL2: "Más paridad de empaquetado e instaladores multiplataforma.",
    futL3: "Matrices de pruebas más completas para conformidad y regresiones.",
    futL4: "Presupuestos de rendimiento y memoria de forma incremental.",
    compareH: "Notas de comparación",
    compareP:
      "Tonet es más acotado que navegadores maduros como Chrome o Brave: primero claridad y control operativo. Ver <a class=\"text-tonet-link hover:underline\" href=\"/compare.html\">Comparar</a> para un resumen breve.",
    footer: "Volver al portal de documentación",
  },
  de: {
    h1: "Öffentliches Handbuch",
    lead:
      "Erweiterte Projektdokumentation: Anwendungsfälle, interne Seiten, Produktentwicklung und Umsetzungspfade.",
    servoH: "Servo und Tonet",
    servoP1:
      "<strong>Servo</strong> ist eine freie, quelloffene <strong>Web-Rendering-Engine</strong> in <strong>Rust</strong>, ausgelegt auf <strong>Einbettung</strong> (WebView-APIs), Parallelität und Speichersicherheit. Ziele und Downloads: <a class=\"text-tonet-link hover:underline\" href=\"https://servo.org/\" target=\"_blank\" rel=\"noopener noreferrer\">servo.org</a>.",
    servoP2:
      "<strong>Geschichte (kurz):</strong> Servo startete bei <strong>Mozilla</strong> um 2012 als Forschungsmotor; <strong>Rust</strong> entwickelte sich parallel. Nach Umstrukturierungen führte die Community Servo weiter; die Governance liegt bei der <strong>Linux Foundation Europe</strong>. Crates u. a. auf <strong>crates.io</strong>. Neutraler Überblick: Wikipedia <a class=\"text-tonet-link hover:underline\" href=\"https://en.wikipedia.org/wiki/Servo_(software)\" target=\"_blank\" rel=\"noopener noreferrer\">Servo (software)</a> (englisch).",
    servoP3:
      "<strong>Warum Tonet Servo nutzt:</strong> Tonet ist eine <strong>minimale Shell</strong>, die Servo für echtes Web-Rendering <strong>im Prozess</strong> einbindet. Ziel ist eine nachvollziehbare Engine-Oberfläche ohne vollen Chromium-Stapel — mit Raum für klare Policies und Ressourcenlimits. Offizielle Seite: <a class=\"text-tonet-link hover:underline\" href=\"https://servo.org/\" target=\"_blank\" rel=\"noopener noreferrer\">servo.org</a>.",
    useH: "Anwendungsfälle",
    useL1: "Browsing mit geringem Overhead, wo deterministisches Verhalten zählt.",
    useL2: "Lehre und Forschung, wo Browser-Internals transparent bleiben müssen.",
    useL3: "Deployments mit kontrollierten Artefakten und Update-Manifesten.",
    useL4: "Sicherheitsbewusste Setups mit expliziter Policy.",
    utilH: "Utilities und interne Seiten",
    utilL1: "<code class=\"rounded bg-white/[0.06] px-1\">tonet://settings</code>: Sprache, Theme, Update-Richtlinie, System-Schalter.",
    utilL2: "<code class=\"rounded bg-white/[0.06] px-1\">tonet://downloads</code>: Fetch-Aktivität und lokales Speichern.",
    utilL3: "<code class=\"rounded bg-white/[0.06] px-1\">tonet://history</code>: Verlauf und Daten-Management.",
    utilL4: "<code class=\"rounded bg-white/[0.06] px-1\">tonet://new-tab</code>: Shortcuts und Suche.",
    recentH: "Aktuelle Änderungen",
    recentP:
      "Der Fokus liegt auf Servo-Integration, Policy-Flächen, Update-Härtung und reproduzierbarer CI/Dokumentation. Versionshinweise folgen den Releases.",
    futureH: "Künftige Umsetzungsschritte",
    futL1: "Breitere Selector-/Layout-Unterstützung für Kompatibilität.",
    futL2: "Mehr Packaging-Parität und Installer über Plattformen.",
    futL3: "Ausführlichere Testmatrizen für Konformität und Regressionen.",
    futL4: "Schrittweise verschärfte Performance- und Speicher-Budgets.",
    compareH: "Vergleichshinweise",
    compareP:
      "Tonet ist schmaler als ausgereifte Browser wie Chrome oder Brave: Klarheit und operative Kontrolle zuerst. Siehe <a class=\"text-tonet-link hover:underline\" href=\"/compare.html\">Vergleich</a>.",
    footer: "Zurück zum Docs-Portal",
  },
  fr: {
    h1: "Manuel public",
    lead:
      "Documentation élargie : cas d’usage, pages internes, évolution du produit et pistes d’implémentation.",
    servoH: "Servo et Tonet",
    servoP1:
      "<strong>Servo</strong> est un <strong>moteur de rendu web</strong> libre et open source, écrit en <strong>Rust</strong>, conçu pour l’<strong>embarquement</strong> (API de type WebView), le parallélisme et la sûreté mémoire. Présentation officielle : <a class=\"text-tonet-link hover:underline\" href=\"https://servo.org/\" target=\"_blank\" rel=\"noopener noreferrer\">servo.org</a>.",
    servoP2:
      "<strong>Histoire (bref) :</strong> Servo démarre chez <strong>Mozilla</strong> vers 2012 comme projet de moteur de rendu ; <strong>Rust</strong> évolue en parallèle. Après les restructurations, la communauté poursuit le projet ; gouvernance ouverte sous la <strong>Linux Foundation Europe</strong>. Crates notamment sur <strong>crates.io</strong>. Vue d’ensemble : Wikipédia <a class=\"text-tonet-link hover:underline\" href=\"https://en.wikipedia.org/wiki/Servo_(software)\" target=\"_blank\" rel=\"noopener noreferrer\">Servo (software)</a> (anglais).",
    servoP3:
      "<strong>Pourquoi Tonet utilise Servo :</strong> Tonet est une <strong>coque minimaliste</strong> qui embarque Servo pour afficher le <strong>web réel en processus</strong>. Nous visons une surface de moteur <strong>auditable</strong>, sans la pile opaque d’un empilement type Chromium, avec de la place pour des politiques explicites et des limites de ressources. Site officiel : <a class=\"text-tonet-link hover:underline\" href=\"https://servo.org/\" target=\"_blank\" rel=\"noopener noreferrer\">servo.org</a>.",
    useH: "Cas d’usage",
    useL1: "Navigation sobre lorsque le comportement prévisible compte.",
    useL2: "Enseignement et R&D où les internals du navigateur doivent rester transparents.",
    useL3: "Déploiements nécessitant des artefacts et manifestes maîtrisés.",
    useL4: "Environnements sensibles à la sécurité avec politiques explicites.",
    utilH: "Utilitaires et pages internes",
    utilL1: "<code class=\"rounded bg-white/[0.06] px-1\">tonet://settings</code> : langue, thème, politique de mise à jour, bascules système.",
    utilL2: "<code class=\"rounded bg-white/[0.06] px-1\">tonet://downloads</code> : activité de téléchargement et stockage local.",
    utilL3: "<code class=\"rounded bg-white/[0.06] px-1\">tonet://history</code> : historique et gestion des données.",
    utilL4: "<code class=\"rounded bg-white/[0.06] px-1\">tonet://new-tab</code> : raccourcis et recherche.",
    recentH: "Changements récents",
    recentP:
      "Les travaux récents portent sur l’intégration Servo, les surfaces de politique, le durcissement des mises à jour et la reproductibilité CI/docs. Les notes de version suivent chaque publication.",
    futureH: "Pistes d’implémentation futures",
    futL1: "Support sélecteur/mise en page élargi pour la compatibilité.",
    futL2: "Meilleure parité d’empaquetage et d’installateurs multiplateforme.",
    futL3: "Matrices de tests plus riches pour conformité et régressions.",
    futL4: "Renforcement progressif des budgets perf et mémoire.",
    compareH: "Notes de comparaison",
    compareP:
      "Tonet est plus ciblé que des navigateurs matures comme Chrome ou Brave : clarté et contrôle opérationnel d’abord. Voir <a class=\"text-tonet-link hover:underline\" href=\"/compare.html\">Comparer</a>.",
    footer: "Retour au portail documentation",
  },
};

export function applyHandbookLocale(lang: SiteLang): void {
  const H = handbookPage[lang];
  setText("handbook-h1", H.h1);
  setText("handbook-lead", H.lead);
  setText("handbook-servo-h", H.servoH);
  setHtml("handbook-servo-p1", H.servoP1);
  setHtml("handbook-servo-p2", H.servoP2);
  setHtml("handbook-servo-p3", H.servoP3);
  setText("handbook-use-h", H.useH);
  setText("handbook-use-l1", H.useL1);
  setText("handbook-use-l2", H.useL2);
  setText("handbook-use-l3", H.useL3);
  setText("handbook-use-l4", H.useL4);
  setText("handbook-util-h", H.utilH);
  setHtml("handbook-util-l1", H.utilL1);
  setHtml("handbook-util-l2", H.utilL2);
  setHtml("handbook-util-l3", H.utilL3);
  setHtml("handbook-util-l4", H.utilL4);
  setText("handbook-recent-h", H.recentH);
  setText("handbook-recent-p", H.recentP);
  setText("handbook-future-h", H.futureH);
  setText("handbook-fut-l1", H.futL1);
  setText("handbook-fut-l2", H.futL2);
  setText("handbook-fut-l3", H.futL3);
  setText("handbook-fut-l4", H.futL4);
  setText("handbook-compare-h", H.compareH);
  setHtml("handbook-compare-p", H.compareP);
  setText("handbook-footer", H.footer);
}

/* ——— Compare (tables filled by id) ——— */

type CmpMatrix = { dimensions: string[][]; features: string[] };

const compareData: Record<SiteLang, CmpMatrix> = {
  en: {
    dimensions: [
      [
        "Rendering engine (core)",
        "Servo (in-process for pages); custom shell — not Blink/WebKit/CEF.",
        "Blink (Chromium).",
        "Blink (Chromium).",
        "Gecko + SpiderMonkey.",
      ],
      [
        "Primary distribution model",
        "Update manifest you configure (<code class=\"rounded bg-white/[0.06] px-1\">version.json</code>) + GitHub release artifacts.",
        "Google update infra + OEM bundles.",
        "Brave update channels.",
        "Mozilla + distro packages.",
      ],
      [
        "Open-source license (browser)",
        "GPL-3.0-or-later (repository).",
        "Chromium BSD-style + proprietary Chrome additions.",
        "MPL/Chromium mix; Brave-owned services.",
        "MPL 2.0.",
      ],
      [
        "Default web compatibility target",
        "MVP reading/search stack; strict page-size policy (e.g. 1 MB guard).",
        "Full modern web.",
        "Full modern web + shields.",
        "Full modern web.",
      ],
      [
        "Extension ecosystem",
        "Not a goal for MVP (no Web Store parity).",
        "Chrome Web Store.",
        "Chrome-compatible extensions.",
        "Firefox AMO + MV3 transition.",
      ],
      [
        "Update check mechanism (Tonet app)",
        "Configurable manifest URL; no GitHub Releases API required at runtime.",
        "Google updater.",
        "Brave updater.",
        "Mozilla updater.",
      ],
      [
        "Typical install size (order of magnitude)",
        "Single-digit ×10 MB binary + platform deps (e.g. Windows ANGLE DLLs beside exe).",
        "Hundreds of MB installed.",
        "Similar to Chromium-class.",
        "Hundreds of MB installed.",
      ],
    ],
    features: [
      "Broad web compatibility (full “Chrome-tier” surface)",
      "Explicit lightweight / purity budgets",
      "Built-in aggressive ad / tracker blocking (default-on)",
      "Fully open-source engine stack (auditable end-to-end)",
      "Operator-controlled release artifacts & update manifest",
      "Public roadmap & engineering transparency",
    ],
  },
  es: {
    dimensions: [
      [
        "Motor de renderizado (núcleo)",
        "Servo (en proceso para páginas); shell propio — no Blink/WebKit/CEF.",
        "Blink (Chromium).",
        "Blink (Chromium).",
        "Gecko + SpiderMonkey.",
      ],
      [
        "Modelo de distribución principal",
        "Manifiesto que configuras (<code class=\"rounded bg-white/[0.06] px-1\">version.json</code>) + artefactos en GitHub Releases.",
        "Infra de actualizaciones de Google + bundles OEM.",
        "Canales de actualización de Brave.",
        "Mozilla + paquetes de distro.",
      ],
      [
        "Licencia open source (navegador)",
        "GPL-3.0-or-later (repositorio).",
        "Chromium estilo BSD + piezas propietarias de Chrome.",
        "Mix MPL/Chromium; servicios de Brave.",
        "MPL 2.0.",
      ],
      [
        "Objetivo de compatibilidad web por defecto",
        "MVP de lectura/búsqueda; política estricta de tamaño (p. ej. límite 1 MB).",
        "Web moderna completa.",
        "Web moderna completa + escudos.",
        "Web moderna completa.",
      ],
      [
        "Ecosistema de extensiones",
        "No es objetivo del MVP (sin paridad con Web Store).",
        "Chrome Web Store.",
        "Extensiones compatibles con Chrome.",
        "Firefox AMO + transición MV3.",
      ],
      [
        "Mecanismo de comprobación de actualizaciones (app Tonet)",
        "URL de manifiesto configurable; sin API de GitHub Releases en tiempo de ejecución.",
        "Actualizador de Google.",
        "Actualizador de Brave.",
        "Actualizador de Mozilla.",
      ],
      [
        "Tamaño típico de instalación (orden de magnitud)",
        "Binario de decenas de MB + dependencias de plataforma (p. ej. DLL ANGLE en Windows).",
        "Cientos de MB instalados.",
        "Similar a Chromium.",
        "Cientos de MB instalados.",
      ],
    ],
    features: [
      "Compatibilidad web amplia (superficie estilo «Chrome»)",
      "Presupuestos explícitos de ligereza / pureza",
      "Bloqueo agresivo de anuncios/rastreadores integrado (activo por defecto)",
      "Pila de motor totalmente open source (auditable de punta a punta)",
      "Artefactos de publicación y manifiesto bajo control del operador",
      "Hoja de ruta pública y transparencia de ingeniería",
    ],
  },
  de: {
    dimensions: [
      [
        "Rendering-Engine (Kern)",
        "Servo (In-Process für Seiten); eigene Shell — kein Blink/WebKit/CEF.",
        "Blink (Chromium).",
        "Blink (Chromium).",
        "Gecko + SpiderMonkey.",
      ],
      [
        "Primäres Verteilungsmodell",
        "Konfigurierbares Manifest (<code class=\"rounded bg-white/[0.06] px-1\">version.json</code>) + GitHub-Release-Artefakte.",
        "Google-Update-Infrastruktur + OEM-Bundles.",
        "Brave-Update-Kanäle.",
        "Mozilla + Distributionspakete.",
      ],
      [
        "Open-Source-Lizenz (Browser)",
        "GPL-3.0-or-later (Repository).",
        "Chromium BSD-ähnlich + proprietäre Chrome-Teile.",
        "MPL/Chromium-Mix; Brave-Dienste.",
        "MPL 2.0.",
      ],
      [
        "Standard-Webkompatibilitätsziel",
        "MVP Lese-/Such-Stack; strikte Seitengröße (z. B. 1-MB-Limit).",
        "Volle moderne Webplattform.",
        "Volle moderne Webplattform + Shields.",
        "Volle moderne Webplattform.",
      ],
      [
        "Erweiterungsökosystem",
        "Kein MVP-Ziel (keine Web-Store-Parität).",
        "Chrome Web Store.",
        "Chrome-kompatible Erweiterungen.",
        "Firefox AMO + MV3-Übergang.",
      ],
      [
        "Update-Prüfmechanismus (Tonet-App)",
        "Konfigurierbare Manifest-URL; keine GitHub-Releases-API zur Laufzeit nötig.",
        "Google-Updater.",
        "Brave-Updater.",
        "Mozilla-Updater.",
      ],
      [
        "Typische Installationsgröße (Größenordnung)",
        "Zweistellige MB-Binary + Plattformabhängigkeiten (z. B. ANGLE-DLLs unter Windows).",
        "Hunderte MB installiert.",
        "Ähnlich Chromium-Klasse.",
        "Hunderte MB installiert.",
      ],
    ],
    features: [
      "Breite Webkompatibilität („Chrome-tier“ Oberfläche)",
      "Explizite Leichtgewicht-/Reinheits-Budgets",
      "Eingebautes aggressives Ad-/Tracker-Blocking (standardmäßig an)",
      "Vollständig open-source Engine-Stack (Ende-zu-Ende auditierbar)",
      "Vom Betreiber gesteuerte Artefakte & Update-Manifest",
      "Öffentliche Roadmap & Engineering-Transparenz",
    ],
  },
  fr: {
    dimensions: [
      [
        "Moteur de rendu (noyau)",
        "Servo (in-process pour les pages) ; enveloppe dédiée — pas Blink/WebKit/CEF.",
        "Blink (Chromium).",
        "Blink (Chromium).",
        "Gecko + SpiderMonkey.",
      ],
      [
        "Modèle de distribution principal",
        "Manifeste configurable (<code class=\"rounded bg-white/[0.06] px-1\">version.json</code>) + artefacts GitHub Releases.",
        "Infra de mise à jour Google + bundles OEM.",
        "Canaux de mise à jour Brave.",
        "Mozilla + paquets distrib.",
      ],
      [
        "Licence open source (navigateur)",
        "GPL-3.0-or-later (dépôt).",
        "Chromium BSD + ajouts propriétaires Chrome.",
        "Mix MPL/Chromium ; services Brave.",
        "MPL 2.0.",
      ],
      [
        "Cible de compatibilité web par défaut",
        "MVP lecture/recherche ; politique stricte de taille (ex. garde 1 Mo).",
        "Web moderne complet.",
        "Web moderne complet + shields.",
        "Web moderne complet.",
      ],
      [
        "Écosystème d’extensions",
        "Pas un objectif MVP (pas de parité Web Store).",
        "Chrome Web Store.",
        "Extensions compatibles Chrome.",
        "Firefox AMO + transition MV3.",
      ],
      [
        "Mécanisme de mise à jour (app Tonet)",
        "URL de manifeste configurable ; pas d’API GitHub Releases imposée au runtime.",
        "Updater Google.",
        "Updater Brave.",
        "Updater Mozilla.",
      ],
      [
        "Taille d’installation typique (ordre de grandeur)",
        "Binaire ~ dizaines de Mo + deps plateforme (ex. DLL ANGLE sous Windows).",
        "Centaines de Mo installés.",
        "Proche de la classe Chromium.",
        "Centaines de Mo installés.",
      ],
    ],
    features: [
      "Large compatibilité web (surface « Chrome-tier »)",
      "Budgets explicites de légèreté / pureté",
      "Blocage pub/trackers agressif intégré (activé par défaut)",
      "Pile moteur entièrement open source (auditable de bout en bout)",
      "Artefacts de publication et manifeste pilotés par l’opérateur",
      "Feuille de route publique et transparence d’ingénierie",
    ],
  },
};

const compareChrome: Record<SiteLang, { h1: string; lead: string; legendH: string; legendP: string; posH: string; posP: string; dimH: string; featH: string; note: string; fitH: string; fitL1: string; fitL2: string; fitL3: string; footDocs: string; footGuide: string }> = {
  en: {
    h1: "Tonet vs mainstream browsers",
    lead:
      "Tonet is intentionally narrow in scope: a transparent, policy-driven surface built on the <strong>Servo</strong> engine with a custom shell. This page summarizes <strong>documented product facts</strong> today — not benchmark winners — so teams can decide fit quickly.",
    legendH: "Legend",
    legendP:
      "<span class=\"ck ck-yes\" aria-label=\"yes\">✓</span> strong / typical, <span class=\"ck ck-part\" aria-label=\"partial\">◐</span> partial or roadmap, <span class=\"ck ck-no\" aria-label=\"no\">○</span> not a focus (not necessarily “bad”).",
    posH: "Positioning",
    posP:
      "Chrome and Brave optimize for broad compatibility and ecosystem depth. Tonet optimizes for teams and advanced users who need a controlled browser surface with clear internals and deterministic packaging.",
    dimH: "Product dimensions",
    featH: "Feature matrix",
    note:
      "Rows reflect architecture and delivery choices described in this repository at the current milestone — not marketing claims or independent benchmarks. Refresh when major milestones ship (see <a class=\"text-tonet-link hover:underline\" href=\"/roadmap.html\">Roadmap</a>).",
    fitH: "When Tonet is a strong fit",
    fitL1: "You need audit-friendly internals and deterministic packaging over maximal site compatibility.",
    fitL2: "You prefer release artifacts and manifests you govern over opaque auto-update channels.",
    fitL3: "You accept MVP limits today in exchange for an explicit roadmap (<code class=\"rounded bg-white/[0.06] px-1\">TONET_VISION.md</code>).",
    footDocs: "Explore full documentation",
    footGuide: "Using Tonet",
  },
  es: {
    h1: "Tonet frente a navegadores generalistas",
    lead:
      "Tonet tiene un alcance deliberadamente acotado: una superficie transparente y orientada a políticas sobre el motor <strong>Servo</strong> y un shell propio. Esta página resume <strong>hechos de producto documentados</strong>, no ganadores de benchmarks, para decidir encaje con rapidez.",
    legendH: "Leyenda",
    legendP:
      "<span class=\"ck ck-yes\" aria-label=\"sí\">✓</span> sólido / típico, <span class=\"ck ck-part\" aria-label=\"parcial\">◐</span> parcial o hoja de ruta, <span class=\"ck ck-no\" aria-label=\"no\">○</span> no es el foco (no implica «malo»).",
    posH: "Posicionamiento",
    posP:
      "Chrome y Brave priorizan compatibilidad amplia y ecosistema. Tonet prioriza equipos y usuarios avanzados que necesitan una superficie controlada, internals claros y empaquetado determinista.",
    dimH: "Dimensiones de producto",
    featH: "Matriz de capacidades",
    note:
      "Las filas reflejan decisiones de arquitectura y entrega descritas en el repositorio en este hito — no claims de marketing ni benchmarks independientes. Actualiza cuando lleguen hitos mayores (véase <a class=\"text-tonet-link hover:underline\" href=\"/roadmap.html\">Hoja de ruta</a>).",
    fitH: "Cuándo Tonet encaja bien",
    fitL1: "Necesitas internals auditables y empaquetado determinista frente a compatibilidad máxima con cualquier sitio.",
    fitL2: "Prefieres artefactos y manifiestos bajo tu control frente a canales opacos.",
    fitL3: "Aceptas límites de MVP a cambio de una hoja de ruta explícita (<code class=\"rounded bg-white/[0.06] px-1\">TONET_VISION.md</code>).",
    footDocs: "Explorar documentación completa",
    footGuide: "Uso de Tonet",
  },
  de: {
    h1: "Tonet vs. Mainstream-Browser",
    lead:
      "Tonet ist bewusst schmal: eine transparente, policy-getriebene Oberfläche auf <strong>Servo</strong> mit eigener Shell. Diese Seite fasst <strong>dokumentierte Produktfakten</strong> zusammen — keine Benchmark-Sieger — damit Teams schnell passend entscheiden können.",
    legendH: "Legende",
    legendP:
      "<span class=\"ck ck-yes\" aria-label=\"ja\">✓</span> stark / typisch, <span class=\"ck ck-part\" aria-label=\"teilweise\">◐</span> teilweise oder Roadmap, <span class=\"ck ck-no\" aria-label=\"nein\">○</span> kein Fokus (nicht automatisch „schlecht“).",
    posH: "Positionierung",
    posP:
      "Chrome und Brave optimieren für breite Kompatibilität und Ökosystemtiefe. Tonet optimiert für Teams und fortgeschrittene Nutzer, die eine kontrollierte Oberfläche, klare Internals und deterministisches Packaging brauchen.",
    dimH: "Produktdimensionen",
    featH: "Feature-Matrix",
    note:
      "Zeilen spiegeln Architektur- und Auslieferungsentscheidungen dieses Repos im aktuellen Meilenstein wider — keine Marketing-Aussagen oder unabhängige Benchmarks. Aktualisieren bei großen Meilensteinen (siehe <a class=\"text-tonet-link hover:underline\" href=\"/roadmap.html\">Roadmap</a>).",
    fitH: "Wann Tonet gut passt",
    fitL1: "Audit-freundliche Internals und deterministisches Packaging sind wichtiger als maximale Site-Kompatibilität.",
    fitL2: "Sie Release-Artefakte und Manifeste selbst steuern möchten statt undurchsichtiger Auto-Update-Kanäle.",
    fitL3: "Sie MVP-Grenzen gegen eine explizite Roadmap (<code class=\"rounded bg-white/[0.06] px-1\">TONET_VISION.md</code>) eintauschen.",
    footDocs: "Zur vollen Dokumentation",
    footGuide: "Tonet nutzen",
  },
  fr: {
    h1: "Tonet face aux navigateurs grand public",
    lead:
      "Tonet est volontairement étroit : une surface transparente pilotée par les politiques, fondée sur <strong>Servo</strong> avec une enveloppe dédiée. Cette page résume des <strong>faits produit documentés</strong>, pas des gagnants de benchmarks, pour décider vite de la pertinence.",
    legendH: "Légende",
    legendP:
      "<span class=\"ck ck-yes\" aria-label=\"oui\">✓</span> fort / typique, <span class=\"ck ck-part\" aria-label=\"partiel\">◐</span> partiel ou feuille de route, <span class=\"ck ck-no\" aria-label=\"non\">○</span> hors focus (pas forcément « mauvais »).",
    posH: "Positionnement",
    posP:
      "Chrome et Brave visent compatibilité large et écosystème. Tonet vise les équipes et utilisateurs avancés qui ont besoin d’une surface contrôlée, d’internals clairs et d’un packaging déterministe.",
    dimH: "Dimensions produit",
    featH: "Matrice de fonctionnalités",
    note:
      "Les lignes reflètent les choix d’architecture et de livraison décrits dans ce dépôt à ce jalon — pas du marketing ni des benchmarks tiers. Rafraîchir lors des jalons majeurs (voir <a class=\"text-tonet-link hover:underline\" href=\"/roadmap.html\">Feuille de route</a>).",
    fitH: "Quand Tonet convient",
    fitL1: "Vous privilégiez des internals auditables et un packaging déterministe plutôt qu’une compatibilité maximale avec tous les sites.",
    fitL2: "Vous préférez maîtriser artefacts et manifestes plutôt que des canaux opaques.",
    fitL3: "Vous acceptez les limites du MVP contre une feuille de route explicite (<code class=\"rounded bg-white/[0.06] px-1\">TONET_VISION.md</code>).",
    footDocs: "Explorer la documentation complète",
    footGuide: "Utiliser Tonet",
  },
};

export function applyComparePageLocale(lang: SiteLang): void {
  const C = compareChrome[lang];
  const M = compareData[lang];
  setText("compare-h1", C.h1);
  setHtml("compare-lead", C.lead);
  setText("compare-legend-h", C.legendH);
  setHtml("compare-legend-p", C.legendP);
  setText("compare-pos-h", C.posH);
  setText("compare-pos-p", C.posP);
  setText("compare-dim-h", C.dimH);
  setText("compare-feat-h", C.featH);
  setHtml("compare-note", C.note);
  setText("compare-fit-h", C.fitH);
  setHtml("compare-fit-l1", C.fitL1);
  setHtml("compare-fit-l2", C.fitL2);
  setHtml("compare-fit-l3", C.fitL3);
  setText("compare-foot-docs", C.footDocs);
  setText("compare-foot-guide", C.footGuide);

  for (let r = 0; r < M.dimensions.length; r++) {
    for (let c = 0; c < M.dimensions[r].length; c++) {
      setHtml(`cmp-dim-${r}-${c}`, M.dimensions[r][c]);
    }
  }
  for (let r = 0; r < M.features.length; r++) {
    setHtml(`cmp-feat-${r}`, M.features[r]);
  }
}

/* ——— Docs: sidebar + extra sections (titles only where applyDocsLocale doesn’t cover) ——— */

const docsExtra: Record<
  SiteLang,
  {
    sidebarTitle: string;
    navOverview: string;
    navServo: string;
    navMap: string;
    navInstall: string;
    navUpdates: string;
    navSign: string;
    navDeb: string;
    navApp: string;
    navCf: string;
    navCdn: string;
    navProduct: string;
    navEng: string;
    mapH: string;
    mapP: string;
    cardProdH: string;
    cardProdP: string;
    cardOpsH: string;
    cardOpsP: string;
    cardEngH: string;
    cardEngP: string;
    overviewH: string;
    overviewP: string;
    cdnH: string;
    cdnP: string;
    productH: string;
    productL1: string;
    productL2: string;
    productL3: string;
    productL4: string;
    engH: string;
    engL1: string;
    engL2: string;
    engL3: string;
    engP: string;
    engP2: string;
    moreH: string;
    moreP: string;
  }
> = {
  en: {
    sidebarTitle: "On this page",
    navOverview: "Overview",
    navServo: "Servo engine",
    navMap: "Documentation map",
    navInstall: "Installation",
    navUpdates: "In-browser updates",
    navSign: "Authenticode (Windows)",
    navDeb: "Debian (.deb)",
    navApp: "AppImage",
    navCf: "Deploy landing",
    navCdn: "CDN variables",
    navProduct: "Use cases",
    navEng: "Engineering",
    mapH: "Documentation map",
    mapP: "Deep guides live beside the repo; this page is the entry hub with anchors into each topic.",
    cardProdH: "Product and use cases",
    cardProdP: "Target users, deployment scenarios, and comparison framing.",
    cardOpsH: "Operations and delivery",
    cardOpsP: "Release assets, update manifest, and signing flow.",
    cardEngH: "Engineering",
    cardEngP: "Architecture, implementation phases, and future work tracking.",
    overviewH: "How this portal is organized",
    overviewP:
      "Use the left outline on desktop or jump links on mobile. Sections follow install → updates → packaging → deploy → audience → engineering — similar to common docs sites, without a second client-side framework.",
    cdnH: "CDN configuration (required variables)",
    cdnP:
      "Configure base URLs for builds and for Tonet’s in-app update checks—same pattern many browsers use with their own infrastructure.",
    productH: "Use cases and audience",
    productL1: "Minimal browsing environments where deterministic behavior matters.",
    productL2: "Teams that need transparent browser internals and release control.",
    productL3: "Operators who govern artifacts and update manifests end-to-end.",
    productL4: "Power users who want a focused browser without broad platform noise.",
    engH: "Implementation and future work",
    engL1: "<strong>Current:</strong> Servo integration foundation, policy modules, and CI coverage.",
    engL2: "<strong>Next:</strong> broader web compatibility layers and stronger packaging parity.",
    engL3: "<strong>Planned:</strong> deeper conformance targets, performance budgets, and expanded platform support.",
    engP:
      "For detailed milestones, consult <code class=\"rounded bg-white/[0.06] px-1\">TONET_VISION.md</code> and <code class=\"rounded bg-white/[0.06] px-1\">docs/SERVO_INTEGRATION_CHECKLIST.md</code>.",
    engP2: "Structured docs source (MkDocs-style): <code class=\"rounded bg-white/[0.06] px-1\">web/landing/docs-site</code>.",
    moreH: "Additional pages",
    moreP:
      "Explore: <a class=\"text-tonet-link hover:underline\" href=\"/guide.html\">Using Tonet</a>, <a class=\"text-tonet-link hover:underline\" href=\"/handbook.html\">Handbook</a>, <a class=\"text-tonet-link hover:underline\" href=\"/compare.html\">Compare</a>, and <a class=\"text-tonet-link hover:underline\" href=\"/roadmap.html\">Roadmap</a>.",
  },
  es: {
    sidebarTitle: "En esta página",
    navOverview: "Resumen",
    navServo: "Motor Servo",
    navMap: "Mapa de documentación",
    navInstall: "Instalación",
    navUpdates: "Actualizaciones en la app",
    navSign: "Authenticode (Windows)",
    navDeb: "Debian (.deb)",
    navApp: "AppImage",
    navCf: "Desplegar landing",
    navCdn: "Variables CDN",
    navProduct: "Casos de uso",
    navEng: "Ingeniería",
    mapH: "Mapa de documentación",
    mapP: "Las guías profundas viven junto al repositorio; esta página es el hub con anclas por tema.",
    cardProdH: "Producto y casos de uso",
    cardProdP: "Usuarios objetivo, despliegues y encuadre comparativo.",
    cardOpsH: "Operaciones y entrega",
    cardOpsP: "Artefactos, manifiesto de actualizaciones y flujo de firma.",
    cardEngH: "Ingeniería",
    cardEngP: "Arquitectura, fases de implementación y seguimiento.",
    overviewH: "Cómo está organizado este portal",
    overviewP:
      "Usa el índice a la izquierda en escritorio o los enlaces en móvil. El orden sigue instalación → actualizaciones → empaquetado → despliegue → audiencia → ingeniería.",
    cdnH: "Configuración CDN (variables necesarias)",
    cdnP:
      "Define URLs base para los builds y para las comprobaciones de actualización en la app—el mismo patrón que muchos navegadores con su infraestructura.",
    productH: "Casos de uso y audiencia",
    productL1: "Entornos donde importa un comportamiento determinista.",
    productL2: "Equipos que necesitan internals transparentes y control de releases.",
    productL3: "Operadores que gobiernan artefactos y manifiestos de punta a punta.",
    productL4: "Usuarios avanzados que buscan un navegador enfocado.",
    engH: "Implementación y trabajo futuro",
    engL1: "<strong>Actual:</strong> base Servo, módulos de política y cobertura CI.",
    engL2: "<strong>Siguiente:</strong> más compatibilidad web y paridad de empaquetado.",
    engL3: "<strong>Planeado:</strong> conformidad, presupuestos de rendimiento y más plataformas.",
    engP:
      "Para hitos detallados: <code class=\"rounded bg-white/[0.06] px-1\">TONET_VISION.md</code> y <code class=\"rounded bg-white/[0.06] px-1\">docs/SERVO_INTEGRATION_CHECKLIST.md</code>.",
    engP2: "Fuente estructurada (estilo MkDocs): <code class=\"rounded bg-white/[0.06] px-1\">web/landing/docs-site</code>.",
    moreH: "Páginas adicionales",
    moreP:
      "Explora: <a class=\"text-tonet-link hover:underline\" href=\"/guide.html\">Uso de Tonet</a>, <a class=\"text-tonet-link hover:underline\" href=\"/handbook.html\">Manual</a>, <a class=\"text-tonet-link hover:underline\" href=\"/compare.html\">Comparar</a> y <a class=\"text-tonet-link hover:underline\" href=\"/roadmap.html\">Hoja de ruta</a>.",
  },
  de: {
    sidebarTitle: "Auf dieser Seite",
    navOverview: "Überblick",
    navServo: "Servo-Engine",
    navMap: "Dokumentationskarte",
    navInstall: "Installation",
    navUpdates: "Updates in der App",
    navSign: "Authenticode (Windows)",
    navDeb: "Debian (.deb)",
    navApp: "AppImage",
    navCf: "Landing deployen",
    navCdn: "CDN-Variablen",
    navProduct: "Anwendungsfälle",
    navEng: "Engineering",
    mapH: "Dokumentationskarte",
    mapP: "Ausführliche Guides liegen am Repo; diese Seite ist der Einstieg mit Ankern.",
    cardProdH: "Produkt und Anwendungsfälle",
    cardProdP: "Zielgruppen, Szenarien und Vergleichsrahmen.",
    cardOpsH: "Betrieb und Auslieferung",
    cardOpsP: "Artefakte, Update-Manifest und Signaturablauf.",
    cardEngH: "Engineering",
    cardEngP: "Architektur, Phasen und Tracking.",
    overviewH: "Wie dieses Portal aufgebaut ist",
    overviewP:
      "Linke Navigation auf dem Desktop oder Sprunglinks mobil. Reihenfolge: Installation → Updates → Packaging → Deploy → Publikum → Engineering.",
    cdnH: "CDN-Konfiguration (erforderliche Variablen)",
    cdnP:
      "Basis-URLs für Builds und In-App-Update-Prüfungen — dasselbe Muster wie bei vielen Browsern mit eigener Infrastruktur.",
    productH: "Anwendungsfälle und Zielgruppe",
    productL1: "Umgebungen mit Bedarf an deterministischem Verhalten.",
    productL2: "Teams mit Bedarf an transparenten Internals und Release-Kontrolle.",
    productL3: "Betreiber, die Artefakte und Manifeste End-to-End steuern.",
    productL4: "Power-User mit Fokus statt Plattform-Rauschen.",
    engH: "Umsetzung und nächste Schritte",
    engL1: "<strong>Aktuell:</strong> Servo-Basis, Policy-Module, CI-Abdeckung.",
    engL2: "<strong>Als Nächstes:</strong> breitere Webkompatibilität und Packaging-Parität.",
    engL3: "<strong>Geplant:</strong> Konformität, Performance-Budgets, mehr Plattformen.",
    engP:
      "Details in <code class=\"rounded bg-white/[0.06] px-1\">TONET_VISION.md</code> und <code class=\"rounded bg-white/[0.06] px-1\">docs/SERVO_INTEGRATION_CHECKLIST.md</code>.",
    engP2: "Strukturierte Quelle (MkDocs-Stil): <code class=\"rounded bg-white/[0.06] px-1\">web/landing/docs-site</code>.",
    moreH: "Weitere Seiten",
    moreP:
      "Mehr: <a class=\"text-tonet-link hover:underline\" href=\"/guide.html\">Tonet nutzen</a>, <a class=\"text-tonet-link hover:underline\" href=\"/handbook.html\">Handbuch</a>, <a class=\"text-tonet-link hover:underline\" href=\"/compare.html\">Vergleich</a>, <a class=\"text-tonet-link hover:underline\" href=\"/roadmap.html\">Roadmap</a>.",
  },
  fr: {
    sidebarTitle: "Sur cette page",
    navOverview: "Vue d’ensemble",
    navServo: "Moteur Servo",
    navMap: "Carte de la documentation",
    navInstall: "Installation",
    navUpdates: "Mises à jour dans l’app",
    navSign: "Authenticode (Windows)",
    navDeb: "Debian (.deb)",
    navApp: "AppImage",
    navCf: "Déployer la landing",
    navCdn: "Variables CDN",
    navProduct: "Cas d’usage",
    navEng: "Ingénierie",
    mapH: "Carte de la documentation",
    mapP: "Les guides détaillés sont dans le dépôt ; cette page est le hub avec ancres.",
    cardProdH: "Produit et cas d’usage",
    cardProdP: "Cibles, déploiements et cadrage comparatif.",
    cardOpsH: "Opérations et livraison",
    cardOpsP: "Artefacts, manifeste de mise à jour et flux de signature.",
    cardEngH: "Ingénierie",
    cardEngP: "Architecture, phases et suivi.",
    overviewH: "Organisation de ce portail",
    overviewP:
      "Sommaire à gauche sur bureau ou liens sur mobile. Ordre : installation → mises à jour → packaging → déploiement → public → ingénierie.",
    cdnH: "Configuration CDN (variables requises)",
    cdnP:
      "Définissez les URL de base pour les builds et les vérifications de mise à jour — comme beaucoup de navigateurs avec leur infrastructure.",
    productH: "Cas d’usage et public",
    productL1: "Environnements où le comportement déterministe compte.",
    productL2: "Équipes qui exigent des internals transparents et un contrôle des releases.",
    productL3: "Opérateurs qui maîtrisent artefacts et manifestes.",
    productL4: "Utilisateurs avancés cherchant un navigateur ciblé.",
    engH: "Implémentation et suite",
    engL1: "<strong>Actuel :</strong> socle Servo, modules de politique, couverture CI.",
    engL2: "<strong>Ensuite :</strong> compatibilité web élargie et parité d’empaquetage.",
    engL3: "<strong>Prévu :</strong> conformité, budgets perf, plateformes élargies.",
    engP:
      "Jalons : <code class=\"rounded bg-white/[0.06] px-1\">TONET_VISION.md</code> et <code class=\"rounded bg-white/[0.06] px-1\">docs/SERVO_INTEGRATION_CHECKLIST.md</code>.",
    engP2: "Source structurée (style MkDocs) : <code class=\"rounded bg-white/[0.06] px-1\">web/landing/docs-site</code>.",
    moreH: "Pages supplémentaires",
    moreP:
      "Voir : <a class=\"text-tonet-link hover:underline\" href=\"/guide.html\">Utiliser Tonet</a>, <a class=\"text-tonet-link hover:underline\" href=\"/handbook.html\">Manuel</a>, <a class=\"text-tonet-link hover:underline\" href=\"/compare.html\">Comparer</a>, <a class=\"text-tonet-link hover:underline\" href=\"/roadmap.html\">Feuille de route</a>.",
  },
};

export function applyDocsExtraLocale(lang: SiteLang): void {
  const D = docsExtra[lang];
  setText("docs-sidebar-title", D.sidebarTitle);
  setText("docs-nav-overview", D.navOverview);
  setText("docs-nav-servo", D.navServo);
  setText("docs-nav-map", D.navMap);
  setText("docs-nav-install", D.navInstall);
  setText("docs-nav-updates", D.navUpdates);
  setText("docs-nav-sign", D.navSign);
  setText("docs-nav-deb", D.navDeb);
  setText("docs-nav-app", D.navApp);
  setText("docs-nav-cf", D.navCf);
  setText("docs-nav-cdn", D.navCdn);
  setText("docs-nav-product", D.navProduct);
  setText("docs-nav-eng", D.navEng);
  setText("docs-map-h", D.mapH);
  setText("docs-map-p", D.mapP);
  setText("docs-card-prod-h", D.cardProdH);
  setText("docs-card-prod-p", D.cardProdP);
  setText("docs-card-ops-h", D.cardOpsH);
  setText("docs-card-ops-p", D.cardOpsP);
  setText("docs-card-eng-h", D.cardEngH);
  setText("docs-card-eng-p", D.cardEngP);
  setText("docs-overview-h", D.overviewH);
  setText("docs-overview-p", D.overviewP);
  setText("docs-cdn-h", D.cdnH);
  setText("docs-cdn-p", D.cdnP);
  setText("docs-product-h", D.productH);
  setText("docs-product-l1", D.productL1);
  setText("docs-product-l2", D.productL2);
  setText("docs-product-l3", D.productL3);
  setText("docs-product-l4", D.productL4);
  setText("docs-eng-h", D.engH);
  setHtml("docs-eng-l1", D.engL1);
  setHtml("docs-eng-l2", D.engL2);
  setHtml("docs-eng-l3", D.engL3);
  setHtml("docs-eng-p", D.engP);
  setHtml("docs-eng-p2", D.engP2);
  setText("docs-more-h", D.moreH);
  setHtml("docs-more-p", D.moreP);
}
