export type DisplayInfo = {
  device_name: string;
  friendly_name: string;
  display_name: string;
  is_primary: boolean;
  position_x: number;
  position_y: number;
  current: Resolution;
  modes: Resolution[];
};

export type Resolution = {
  width: number;
  height: number;
  bits_per_pixel: number;
  refresh_hz: number;
};

export type PopularResolution = {
  key: string;
  width: number;
  height: number;
  text: string;
  stdLabel: string | null;
  area: number;
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
  if (w === 2560 && h === 1440) return "2K";
  if (w === 3840 && h === 2160) return "4K";
  return null;
}

export function getPopularResolutions(
  modes: Resolution[],
  current: Resolution | null
): PopularResolution[] {
  // dedupe by width x height
  const uniq = new Map<
    string,
    { key: string; width: number; height: number }
  >();
  for (const m of modes) {
    const key = `${m.width}x${m.height}`;
    if (!uniq.has(key))
      uniq.set(key, { key, width: m.width, height: m.height });
  }
  const items = Array.from(uniq.values());

  const currentAspect = current
    ? aspectKey(current.width, current.height)
    : null;

  const scored = items.map((it) => {
    const isAspectMatch = currentAspect
      ? aspectKey(it.width, it.height) === currentAspect
      : false;
    const stdLabel = labelForStandard(it.width, it.height);
    const isPopular = Boolean(isAspectMatch && stdLabel);
    return {
      ...it,
      isPopular,
      stdLabel,
      area: it.width * it.height,
    };
  });

  // Filter only popular resolutions and sort by area (desc)
  const popularResolutions = scored
    .filter((s) => s.isPopular)
    .sort((a, b) => b.area - a.area);

  return popularResolutions.map((s) => ({
    key: s.key,
    width: s.width,
    height: s.height,
    text: `${s.width} × ${s.height}${s.stdLabel ? ` (${s.stdLabel})` : ""}`,
    stdLabel: s.stdLabel,
    area: s.area,
  }));
}
