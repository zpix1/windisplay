import React, { useCallback, useMemo } from "react";
import "./Selector.css";

export type SelectorProps<T> = {
  items: T[];
  selectedItem: T | null;
  onChange: (item: T) => void;
  getKey: (item: T) => string;
  getLabel: (item: T) => React.ReactNode;
  ariaLabel?: string;
  disabled?: boolean;
  getItemDisabled?: (item: T) => boolean;
};

export function Selector<T>(props: SelectorProps<T>) {
  const {
    items,
    selectedItem,
    onChange,
    getKey,
    getLabel,
    ariaLabel,
    disabled,
    getItemDisabled,
  } = props;

  const selectedIndex = useMemo(() => {
    if (!selectedItem) return -1;
    const selectedKey = getKey(selectedItem);
    return items.findIndex((it) => getKey(it) === selectedKey);
  }, [items, selectedItem, getKey]);

  const changeBy = useCallback(
    (delta: number) => {
      if (disabled || items.length === 0) return;
      let nextIndex = selectedIndex + delta;
      if (nextIndex < 0) nextIndex = 0;
      if (nextIndex >= items.length) nextIndex = items.length - 1;

      const trySelect = (start: number, step: number) => {
        let idx = start;
        while (idx >= 0 && idx < items.length) {
          const candidate = items[idx];
          const isItemDisabled = getItemDisabled?.(candidate) ?? false;
          if (!isItemDisabled) {
            onChange(candidate);
            break;
          }
          idx += step;
        }
      };
      const step = delta > 0 ? 1 : -1;
      trySelect(nextIndex, step);
    },
    [disabled, items, selectedIndex, getItemDisabled, onChange]
  );

  const onKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLDivElement>) => {
      if (disabled) return;
      switch (e.key) {
        case "ArrowLeft":
        case "ArrowUp":
          e.preventDefault();
          changeBy(-1);
          break;
        case "ArrowRight":
        case "ArrowDown":
          e.preventDefault();
          changeBy(1);
          break;
        case "Home":
          e.preventDefault();
          if (items.length > 0) onChange(items[0]);
          break;
        case "End":
          e.preventDefault();
          if (items.length > 0) onChange(items[items.length - 1]);
          break;
        default:
          break;
      }
    },
    [changeBy, disabled, items, onChange]
  );

  const indicatorStyle = useMemo(() => {
    const count = Math.max(1, items.length);
    const widthPct = 100 / count;
    const index = selectedIndex >= 0 ? selectedIndex : 0;
    return {
      width: `calc(${widthPct}%)`,
      transform: `translateX(${index * 100}%)`,
    } as React.CSSProperties;
  }, [items.length, selectedIndex]);

  return (
    <div
      className={`selector ${disabled ? "selector-disabled" : ""}`}
      role="radiogroup"
      aria-label={ariaLabel}
      onKeyDown={onKeyDown}
      tabIndex={0}
    >
      {items.length > 0 && selectedIndex >= 0 && (
        <div className="selector-indicator" style={indicatorStyle} />
      )}
      {items.map((item, index) => {
        const key = getKey(item);
        const label = getLabel(item);
        const isSelected = index === selectedIndex;
        const isItemDisabled = disabled || (getItemDisabled?.(item) ?? false);
        return (
          <button
            key={key}
            type="button"
            className={`selector-segment ${isSelected ? "selected" : ""}`}
            aria-checked={isSelected}
            role="radio"
            onClick={() => !isItemDisabled && onChange(item)}
            disabled={isItemDisabled}
            title={typeof label === "string" ? label : undefined}
          >
            <span className="segment-label">{label}</span>
          </button>
        );
      })}
    </div>
  );
}
