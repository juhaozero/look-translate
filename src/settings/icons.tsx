import type { ReactNode } from "react";
import appIcon from "../assets/app-icon.svg";

type IconProps = { className?: string };

function LineIcon({ className, children }: IconProps & { children: ReactNode }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.65" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      {children}
    </svg>
  );
}

export function IconApp({ className }: IconProps) {
  return <img className={className} src={appIcon} alt="" aria-hidden="true" draggable={false} />;
}

export function IconGeneral(props: IconProps) {
  return <LineIcon {...props}><path d="M4 7h5m5 0h6M4 17h10m5 0h1" /><circle cx="11.5" cy="7" r="2.5" /><circle cx="16.5" cy="17" r="2.5" /></LineIcon>;
}

export function IconTranslate(props: IconProps) {
  return <LineIcon {...props}><path d="M4 7h14m-4-4 4 4-4 4M20 17H6m4-4-4 4 4 4" /></LineIcon>;
}

export function IconHotkey(props: IconProps) {
  return <LineIcon {...props}><rect x="3" y="5.5" width="18" height="13" rx="3" /><path d="M7 9.5h.5m4.25 0h.5m4.25 0h.5M7 14.5h10" /></LineIcon>;
}

export function IconService(props: IconProps) {
  return <LineIcon {...props}><rect x="4" y="3.5" width="16" height="7" rx="2.5" /><rect x="4" y="13.5" width="16" height="7" rx="2.5" /><path d="M8 7h.5M12 7h4M8 17h.5M12 17h4" /></LineIcon>;
}

export function IconDict(props: IconProps) {
  return <LineIcon {...props}><path d="M12 6c-2.5-2-5.5-2.5-9-2v15c3.5-.5 6.5 0 9 2 2.5-2 5.5-2.5 9-2V4c-3.5-.5-6.5 0-9 2Zm0 0v15M6.5 8.5l2 .5M6.5 12.5l2 .5m7-4 2-.5m-2 4.5 2-.5" /></LineIcon>;
}

export function IconAbout(props: IconProps) {
  return <LineIcon {...props}><rect x="4" y="3" width="16" height="18" rx="4" /><path d="M12 11v5m-1-5h1m-2 5h4" /><circle cx="12" cy="7.5" r=".75" fill="currentColor" stroke="none" /></LineIcon>;
}
