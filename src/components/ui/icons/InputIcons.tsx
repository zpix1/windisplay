type IconProps = { size?: number };

export function DisplayPortIcon({ size = 18 }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" aria-hidden>
      <rect x="3" y="6" width="18" height="12" rx="2" fill="currentColor" opacity="0.12" />
      <path d="M7 9h5a3 3 0 1 1 0 6H7V9zm2 2v2h3a1 1 0 0 0 0-2H9z" fill="currentColor" />
    </svg>
  );
}

export function HdmiIcon({ size = 18 }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" aria-hidden>
      <path d="M4 10h16v4a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2v-4z" fill="currentColor" opacity="0.12" />
      <rect x="5" y="7" width="14" height="3" rx="1" fill="currentColor" />
    </svg>
  );
}

export function DviIcon({ size = 18 }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" aria-hidden>
      <rect x="3" y="8" width="18" height="8" rx="2" fill="currentColor" opacity="0.12" />
      <circle cx="7" cy="12" r="1" fill="currentColor" />
      <circle cx="10" cy="12" r="1" fill="currentColor" />
      <circle cx="13" cy="12" r="1" fill="currentColor" />
      <circle cx="16" cy="12" r="1" fill="currentColor" />
    </svg>
  );
}

export function VgaIcon({ size = 18 }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" aria-hidden>
      <rect x="4" y="9" width="16" height="6" rx="3" fill="currentColor" opacity="0.12" />
      <rect x="6" y="10.5" width="12" height="3" rx="1.5" fill="currentColor" />
    </svg>
  );
}

export function UsbCIcon({ size = 18 }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" aria-hidden>
      <rect x="5" y="10" width="14" height="4" rx="2" fill="currentColor" opacity="0.12" />
      <rect x="8" y="11" width="8" height="2" rx="1" fill="currentColor" />
    </svg>
  );
}

export function CompositeIcon({ size = 18 }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" aria-hidden>
      <circle cx="12" cy="12" r="5" fill="currentColor" opacity="0.12" />
      <circle cx="12" cy="12" r="2" fill="currentColor" />
    </svg>
  );
}

export function SVideoIcon({ size = 18 }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" aria-hidden>
      <circle cx="12" cy="12" r="6" fill="currentColor" opacity="0.12" />
      <circle cx="10" cy="12" r="1" fill="currentColor" />
      <circle cx="14" cy="12" r="1" fill="currentColor" />
    </svg>
  );
}

export function TunerIcon({ size = 18 }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" aria-hidden>
      <rect x="6" y="7" width="12" height="10" rx="2" fill="currentColor" opacity="0.12" />
      <rect x="8" y="9" width="8" height="6" rx="1" fill="currentColor" />
    </svg>
  );
}

export function ComponentIcon({ size = 18 }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" aria-hidden>
      <circle cx="8" cy="12" r="2" fill="currentColor" />
      <circle cx="12" cy="12" r="2" fill="currentColor" />
      <circle cx="16" cy="12" r="2" fill="currentColor" />
    </svg>
  );
}
