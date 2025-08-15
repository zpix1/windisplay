type MonitorIconProps = {
  type: "laptop" | "external";
  manufacturer: string; // 3 letters string, will be uppercased and trimmed to 3
};

export function MonitorIcon({ type, manufacturer }: MonitorIconProps) {
  const viewBoxSize = 100;
  const padding = 10;
  const contentSize = viewBoxSize - padding * 2;

  const stroke = "currentColor";
  const strokeWidth = 2;
  const fill = "transparent";

  const cx = viewBoxSize / 2;
  const offsetY = 4;

  // Sanitize manufacturer text
  const label = (manufacturer || "").toUpperCase().slice(0, 3);

  // Common screen sizing (leave room below for base/stand)
  const screenWidth = contentSize;
  const screenHeight = Math.round(contentSize * 0.58);
  const screenX = padding;
  const screenY = padding + 2;
  const screenRx = 6;

  // External monitor stand
  const standWidth = 6;
  const standHeight = 8;
  const standX = cx - standWidth / 2;
  const standY = screenY + screenHeight + 4;

  const baseWidth = 34;
  const baseHeight = 6;
  const baseX = cx - baseWidth / 2;
  const baseY = standY + standHeight;

  // Laptop base (keyboard deck)
  const deckWidth = contentSize + 6;
  const deckHeight = 10;
  const deckX = cx - deckWidth / 2;
  const deckY = screenY + screenHeight + 6;

  // Trackpad (small detail)
  const padWidth = 12;
  const padHeight = 4;
  const padX = cx - padWidth / 2;
  const padY = deckY + (deckHeight - padHeight) / 2;

  // Text sizing inside screen
  const textFontSize = Math.max(8, Math.min(screenWidth, screenHeight) * 0.6);

  return (
    <svg
      width={30}
      height={30}
      viewBox={`0 0 ${viewBoxSize} ${viewBoxSize}`}
      xmlns="http://www.w3.org/2000/svg"
      aria-hidden
      focusable="false"
    >
      <g transform={`translate(0, ${offsetY})`}>
        {/* Screen */}
        <rect
          x={screenX}
          y={screenY}
          width={screenWidth}
          height={screenHeight}
          rx={screenRx}
          fill={fill}
          stroke={stroke}
          strokeWidth={strokeWidth}
        />

        {/* Manufacturer label */}
        {label && (
          <text
            x={screenX + screenWidth / 2}
            y={screenY + screenHeight / 2}
            textAnchor="middle"
            dominantBaseline="central"
            fontSize={textFontSize}
            fontWeight={600}
            fill={stroke}
            fontFamily="system-ui, -apple-system, Segoe UI, Roboto, Ubuntu, Cantarell, Noto Sans, Arial, sans-serif"
          >
            {label}
          </text>
        )}

        {type === "external" ? (
          <>
            {/* Stand */}
            <rect
              x={standX}
              y={standY}
              width={standWidth}
              height={standHeight}
              fill={stroke}
            />
            {/* Base plate */}
            <rect
              x={baseX}
              y={baseY}
              width={baseWidth}
              height={baseHeight}
              rx={baseHeight / 2}
              fill={stroke}
            />
          </>
        ) : (
          <>
            {/* Laptop hinge line */}
            <line
              x1={screenX + 8}
              y1={screenY + screenHeight + 2}
              x2={screenX + screenWidth - 8}
              y2={screenY + screenHeight + 2}
              stroke={stroke}
              strokeWidth={strokeWidth}
              strokeLinecap="round"
            />
            {/* Keyboard deck */}
            <rect
              x={deckX}
              y={deckY}
              width={deckWidth}
              height={deckHeight}
              rx={3}
              fill={stroke}
            />
            {/* Trackpad */}
            <rect
              x={padX}
              y={padY}
              width={padWidth}
              height={padHeight}
              rx={2}
              fill="white"
              opacity={0.3}
            />
          </>
        )}
      </g>
    </svg>
  );
}
