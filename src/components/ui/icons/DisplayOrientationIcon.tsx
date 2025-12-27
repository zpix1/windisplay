type DisplayOrientationIconProps = {
  aspectRatioKey: string; // e.g. "16:9", "4:3"
  orientation: number; // degrees (e.g. 0, 90, 180, 270)
};

/**
 * Renders a monitor rectangle with the given aspect ratio and an arrow
 * indicating the "up" direction based on orientation.
 *
 * The rectangle is scaled to fit within the icon while preserving aspect ratio.
 */
export function DisplayOrientationIcon({
  aspectRatioKey,
  orientation,
}: DisplayOrientationIconProps) {
  const viewBoxSize = 100;
  const padding = 10; // outer padding from edges (viewbox units)
  const baseContentSize = viewBoxSize - padding * 2; // 80

  const match = /^(\d+)\s*:\s*(\d+)$/.exec(aspectRatioKey.trim());
  const aw = match ? parseInt(match[1], 10) : 16;
  const ah = match ? parseInt(match[2], 10) : 9;
  const safeAspectW = Math.max(1, Math.abs(aw));
  const safeAspectH = Math.max(1, Math.abs(ah));
  const aspect = safeAspectW / safeAspectH;

  // Draw everything in a "base" orientation (bar at bottom), then rotate as needed.
  // To avoid clipping after rotation, reserve space for the base bar inside the viewBox.
  const normalizedOrientation = ((orientation % 360) + 360) % 360;
  const gap = 0; // space between monitor outline and base bar

  // Because we rotate the whole icon, 90/270 swaps the displayed width/height.
  // Invert the layout aspect in those cases so the final on-screen rectangle matches aspectRatioKey.
  const layoutAspect =
    normalizedOrientation === 90 || normalizedOrientation === 270
      ? 1 / aspect
      : aspect;

  // First pass: approximate rect size within base content, to estimate bar thickness.
  const approxRectWidth =
    layoutAspect >= 1 ? baseContentSize : baseContentSize * layoutAspect;
  const approxRectHeight =
    layoutAspect >= 1 ? baseContentSize / layoutAspect : baseContentSize;
  const approxBarThickness = Math.max(
    4,
    Math.min(approxRectWidth, approxRectHeight) * 0.12
  );

  // Second pass: fit the monitor rectangle within the remaining space above the bar.
  const contentSize = Math.max(1, baseContentSize - approxBarThickness - gap);
  const rectWidth = layoutAspect >= 1 ? contentSize : contentSize * layoutAspect;
  const rectHeight = layoutAspect >= 1 ? contentSize / layoutAspect : contentSize;
  const cx = viewBoxSize / 2;
  const cy = viewBoxSize / 2;
  const rectX = cx - rectWidth / 2;
  const rectY = cy - rectHeight / 2;

  const stroke = "currentColor";
  const strokeWidth = 2;
  const rectFill = "transparent";
  const barFill = "currentColor";

  const barThickness = Math.max(4, Math.min(rectWidth, rectHeight) * 0.12);

  // Account for the rectangle stroke so the bar matches the OUTER bounds
  const outerRectX = rectX - strokeWidth / 2;
  const outerRectY = rectY - strokeWidth / 2;
  const outerRectWidth = rectWidth + strokeWidth;
  const outerRectHeight = rectHeight + strokeWidth;

  const barX = outerRectX;
  const barY = outerRectY + outerRectHeight + gap; // bottom, flush with outer edge
  const barW = outerRectWidth;
  const barH = barThickness;

  // Helper to build a path for a rounded rectangle where corners can be toggled independently
  const buildRoundedRectPath = (
    x: number,
    y: number,
    width: number,
    height: number,
    rTL: number,
    rTR: number,
    rBR: number,
    rBL: number
  ): string => {
    const x0 = x;
    const y0 = y;
    const x1 = x + width;
    const y1 = y + height;

    return [
      `M ${x0 + rTL},${y0}`,
      `H ${x1 - rTR}`,
      rTR > 0 ? `A ${rTR},${rTR} 0 0 1 ${x1},${y0 + rTR}` : `L ${x1},${y0}`,
      `V ${y1 - rBR}`,
      rBR > 0 ? `A ${rBR},${rBR} 0 0 1 ${x1 - rBR},${y1}` : `L ${x1},${y1}`,
      `H ${x0 + rBL}`,
      rBL > 0 ? `A ${rBL},${rBL} 0 0 1 ${x0},${y1 - rBL}` : `L ${x0},${y1}`,
      `V ${y0 + rTL}`,
      rTL > 0 ? `A ${rTL},${rTL} 0 0 1 ${x0 + rTL},${y0}` : `L ${x0},${y0}`,
      "Z",
    ].join(" ");
  };

  // Build a path for the monitor with selective rounded corners
  const cornerRadius = 4;
  const maxRadius = Math.min(rectWidth, rectHeight) / 2;
  const r = Math.min(cornerRadius, maxRadius);

  // Base orientation: bar touches the bottom edge → top corners rounded, bottom corners square.
  const rTL = r;
  const rTR = r;
  const rBR = 0;
  const rBL = 0;

  const monitorPathD = buildRoundedRectPath(
    rectX,
    rectY,
    rectWidth,
    rectHeight,
    rTL,
    rTR,
    rBR,
    rBL
  );

  // Build a path for the bar with rounding only on the far side (away from the monitor)
  const barCornerRadius = Math.min(r, Math.min(barW, barH) / 2);
  // Base orientation: bar at bottom → round bottom-left and bottom-right
  const barRTL = 0;
  const barRTR = 0;
  const barRBR = barCornerRadius;
  const barRBL = barCornerRadius;

  const barPathD = buildRoundedRectPath(
    barX,
    barY,
    barW,
    barH,
    barRTL,
    barRTR,
    barRBR,
    barRBL
  );

  const prefersReducedMotion = () => {
    if (typeof window === "undefined" || !window.matchMedia) return false;
    return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  };

  return (
    <svg
      width={24}
      height={24}
      viewBox={`0 0 ${viewBoxSize} ${viewBoxSize}`}
      xmlns="http://www.w3.org/2000/svg"
      aria-hidden
      focusable="false"
    >
      <g
        style={{
          transform: `rotate(${normalizedOrientation}deg)`,
          // Use a stable bounding box for transform calculations so rotation never "jumps" in size/position.
          transformOrigin: "center",
          transformBox: "fill-box",
          transition: prefersReducedMotion()
            ? "none"
            : "transform 180ms cubic-bezier(0.2, 0, 0, 1)",
        }}
      >
        {/* Invisible rect ensures the group's bbox is always the full viewBox (prevents transform-box weirdness). */}
        <rect
          x={0}
          y={0}
          width={viewBoxSize}
          height={viewBoxSize}
          fill="transparent"
        />

        {/* Outer container for rectangle with selective rounded corners */}
        <path
          d={monitorPathD}
          fill={rectFill}
          stroke={stroke}
          strokeWidth={strokeWidth}
        />

        {/* Centered letter */}
        <text
          x={cx}
          y={cy}
          textAnchor="middle"
          dominantBaseline="central"
          fontSize={Math.max(8, Math.min(rectWidth, rectHeight) * 0.7)}
          fontWeight={500}
          fill={stroke}
          fontFamily="system-ui, -apple-system, Segoe UI, Roboto, Ubuntu, Cantarell, Noto Sans, Arial, sans-serif"
        >
          A
        </text>

        {/* External base bar with rounding only on the far side */}
        <path d={barPathD} fill={barFill} stroke="none" />
      </g>
    </svg>
  );
}
