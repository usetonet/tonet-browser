import type { IconType } from "react-icons";
import { FaWindows } from "react-icons/fa6";
import { SiAndroid, SiApple, SiIos, SiLinux } from "react-icons/si";

const BRANDS: { Icon: IconType; title: string; className: string }[] = [
  { Icon: FaWindows, title: "Windows", className: "text-[#00A4EF]" },
  { Icon: SiLinux, title: "Linux", className: "text-[#FCC624]" },
  { Icon: SiApple, title: "macOS", className: "text-white" },
  { Icon: SiAndroid, title: "Android", className: "text-[#3DDC84]" },
  { Icon: SiIos, title: "iPhone (iOS)", className: "text-[#a8b0c4]" },
];

function BrandIcon({ Icon, title, className }: { Icon: IconType; title: string; className: string }) {
  return (
    <div className={`home-kpi-os-ico shrink-0 ${className}`} title={title} aria-hidden="true">
      <Icon className="size-8" />
    </div>
  );
}

/** Infinite L→R marquee of platform marks (react-icons). Pauses when reduced motion is preferred. */
export function HomePlatformIcons() {
  return (
    <div className="home-kpi-marquee w-full min-w-0" aria-hidden="true">
      <div className="home-kpi-marquee-track">
        {[0, 1].map((cycle) => (
          <div key={cycle} className="home-kpi-marquee-set flex shrink-0 items-center gap-4 pr-4">
            {BRANDS.map((b, i) => (
              <BrandIcon key={`${cycle}-${i}`} Icon={b.Icon} title={b.title} className={b.className} />
            ))}
          </div>
        ))}
      </div>
    </div>
  );
}
