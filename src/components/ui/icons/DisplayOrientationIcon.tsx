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
  const contentSize = viewBoxSize - padding * 2; // 80

  const match = /^(\d+)\s*:\s*(\d+)$/.exec(aspectRatioKey.trim());
  const aw = match ? parseInt(match[1], 10) : 16;
  const ah = match ? parseInt(match[2], 10) : 9;
  const safeAspectW = Math.max(1, Math.abs(aw));
  const safeAspectH = Math.max(1, Math.abs(ah));
  const aspect = safeAspectW / safeAspectH;

  // Fit rectangle inside content box while preserving aspect
  const rectWidth = aspect >= 1 ? contentSize : contentSize * aspect;
  const rectHeight = aspect >= 1 ? contentSize / aspect : contentSize;
  const cx = viewBoxSize / 2;
  const cy = viewBoxSize / 2;
  const rectX = cx - rectWidth / 2;
  const rectY = cy - rectHeight / 2;

  const stroke = "currentColor";
  const strokeWidth = 2;
  const rectFill = "transparent";
  const barFill = "currentColor";

  // External bar indicating monitor base (full width/height, outside the rectangle)
  const normalizedOrientation = ((orientation % 360) + 360) % 360;
  const gap = 0; // space between monitor outline and base line
  const barThickness = Math.max(4, Math.min(rectWidth, rectHeight) * 0.12);

  // Account for the rectangle stroke so the bar matches the OUTER bounds
  const outerRectX = rectX - strokeWidth / 2;
  const outerRectY = rectY - strokeWidth / 2;
  const outerRectWidth = rectWidth + strokeWidth;
  const outerRectHeight = rectHeight + strokeWidth;

  let barX = outerRectX;
  let barY = outerRectY + outerRectHeight + gap; // default: bottom, flush with outer edge
  let barW = outerRectWidth;
  let barH = barThickness;
  if (normalizedOrientation === 90) {
    // left
    barX = outerRectX - gap - barThickness;
    barY = outerRectY;
    barW = barThickness;
    barH = outerRectHeight;
  } else if (normalizedOrientation === 180) {
    // top
    barX = outerRectX;
    barY = outerRectY - gap - barThickness;
    barW = outerRectWidth;
    barH = barThickness;
  } else if (normalizedOrientation === 270) {
    // right
    barX = outerRectX + outerRectWidth + gap;
    barY = outerRectY;
    barW = barThickness;
    barH = outerRectHeight;
  }

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

  const roundTL = normalizedOrientation === 0 || normalizedOrientation === 270; // no bar on top/left sides touching TL
  const roundTR = normalizedOrientation === 0 || normalizedOrientation === 90; // no bar on top/right sides touching TR
  const roundBR = normalizedOrientation === 180 || normalizedOrientation === 90; // no bar on bottom/right sides touching BR
  const roundBL =
    normalizedOrientation === 180 || normalizedOrientation === 270; // no bar on bottom/left sides touching BL

  const rTL = roundTL ? r : 0;
  const rTR = roundTR ? r : 0;
  const rBR = roundBR ? r : 0;
  const rBL = roundBL ? r : 0;

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
  let barRTL = 0;
  let barRTR = 0;
  let barRBR = 0;
  let barRBL = 0;
  if (normalizedOrientation === 0) {
    // bar at bottom → round bottom-left and bottom-right
    barRBL = barCornerRadius;
    barRBR = barCornerRadius;
  } else if (normalizedOrientation === 180) {
    // bar at top → round top-left and top-right
    barRTL = barCornerRadius;
    barRTR = barCornerRadius;
  } else if (normalizedOrientation === 90) {
    // bar at left → round top-left and bottom-left
    barRTL = barCornerRadius;
    barRBL = barCornerRadius;
  } else if (normalizedOrientation === 270) {
    // bar at right → round top-right and bottom-right
    barRTR = barCornerRadius;
    barRBR = barCornerRadius;
  }

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

  return (
    <svg
      width={24}
      height={24}
      viewBox={`0 0 ${viewBoxSize} ${viewBoxSize}`}
      xmlns="http://www.w3.org/2000/svg"
      aria-hidden
      focusable="false"
    >
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

      {/* External base line (full width/height) placed beside the monitor) with selective rounding on far side */}
      <path d={barPathD} fill={barFill} stroke="none" />
    </svg>
  );
}
