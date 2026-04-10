export type DetectedOS = "windows" | "linux" | "macos" | "unknown";

export function detectOS(): DetectedOS {
  const ua = navigator.userAgent.toLowerCase();
  const platform =
    (navigator as Navigator & { userAgentData?: { platform?: string } }).userAgentData?.platform?.toLowerCase() ?? "";

  if (/win/.test(ua) || platform.includes("win")) return "windows";
  if (/mac|iphone|ipad|ipod/.test(ua) || platform.includes("mac")) return "macos";
  if (/linux|android/.test(ua) || platform.includes("linux")) return "linux";
  return "unknown";
}
