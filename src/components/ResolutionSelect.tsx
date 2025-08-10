import React, { useMemo } from "react";

export type Resolution = {
  width: number;
  height: number;
  bits_per_pixel: number;
  refresh_hz: number;
};

type Props = {
  modes: Resolution[];
  current: Resolution | null;
  value: string;
  disabled?: boolean;
  onChange: (value: string) => void;
};

function gcd(a: number, b: number): number {
  while (b !== 0) {
    const t = b;
    b = a % b;
    a = t;
  }
  return Math.abs(a);
}

function aspectKey(w: number, h: number): string {
  const g = gcd(w, h) || 1;
  return `${Math.floor(w / g)}:${Math.floor(h / g)}`;
}

function labelForStandard(w: number, h: number): string | null {
  if (w === 1280 && h === 720) return "720p";
  if (w === 1920 && h === 1080) return "1080p";
  if (w === 2560 && h === 1440) return "2k";
  if (w === 3840 && h === 2160) return "4k";
  return null;
}

export default function ResolutionSelect(props: Props) {
  const { modes, current, value, disabled, onChange } = props;

  const options = useMemo(() => {
    // dedupe by width x height
    const uniq = new Map<string, { key: string; width: number; height: number }>();
    for (const m of modes) {
      const key = `${m.width}x${m.height}`;
      if (!uniq.has(key)) uniq.set(key, { key, width: m.width, height: m.height });
    }
    const items = Array.from(uniq.values());

    const currentAspect = current ? aspectKey(current.width, current.height) : null;

    const scored = items.map((it) => {
      const isAspectMatch = currentAspect ? aspectKey(it.width, it.height) === currentAspect : false;
      const stdLabel = labelForStandard(it.width, it.height);
      const isPopular = Boolean(isAspectMatch && stdLabel);
      return {
        ...it,
        isPopular,
        stdLabel,
        area: it.width * it.height,
      };
    });

    // Popular first (desc area), then others (desc area)
    scored.sort((a, b) => {
      if (a.isPopular !== b.isPopular) return a.isPopular ? -1 : 1;
      return b.area - a.area;
    });

    return scored.map((s) => ({
      key: s.key,
      text: `${s.width} × ${s.height}${s.stdLabel ? ` (${s.stdLabel})` : ""}`,
    }));
  }, [modes, current]);

  return (
    <select
      id="resolution-select"
      className="select"
      disabled={disabled}
      value={value}
      onChange={(e) => onChange(e.target.value)}
    >
      {options.map((opt) => (
        <option value={opt.key} key={opt.key}>
          {opt.text}
        </option>
      ))}
    </select>
  );
}


