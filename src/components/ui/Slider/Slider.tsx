import { useEffect, useRef, ReactNode, useState } from "react";
import "./Slider.css";

type Props = {
  id?: string;
  min?: number;
  max?: number;
  step?: number;
  value: number;
  disabled?: boolean;
  onChange?: (value: number) => void;
  onValueSubmit?: (value: number) => void;
  className?: string;
  icon?: ReactNode;
  label?: string;
  stickyPoints?: number[];
};

export function Slider({
  id,
  min = 0,
  max = 100,
  step = 1,
  value,
  disabled = false,
  onChange,
  onValueSubmit,
  className = "",
  icon,
  label,
  stickyPoints,
}: Props) {
  const sliderRef = useRef<HTMLInputElement>(null);
  const [isMouseDown, setIsMouseDown] = useState(false);
  const [pendingValue, setPendingValue] = useState<number | null>(null);
  const [displayValue, setDisplayValue] = useState(value);

  const clamp = (v: number) => {
    if (Number.isNaN(v)) return min;
    return Math.min(max, Math.max(min, v));
  };

  // Helper function to find the nearest sticky point
  const findNearestStickyPoint = (inputValue: number): number => {
    if (!stickyPoints || stickyPoints.length === 0) {
      return inputValue;
    }

    if (max === min) {
      return min;
    }

    // Convert input value to 0-100 scale
    const normalizedInput = ((inputValue - min) / (max - min)) * 100;

    // Find the closest sticky point
    let nearestPoint = stickyPoints[0];
    let minDistance = Math.abs(normalizedInput - nearestPoint);

    for (const point of stickyPoints) {
      const distance = Math.abs(normalizedInput - point);
      if (distance < minDistance) {
        minDistance = distance;
        nearestPoint = point;
      }
    }

    // Convert back to original scale
    return min + (nearestPoint / 100) * (max - min);
  };

  // Apply external value changes immediately. These changes commonly happen while
  // switching monitors, where animation makes the UI feel like it is still loading.
  useEffect(() => {
    if (isMouseDown) {
      return;
    }
    setDisplayValue(clamp(value));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [value, min, max, isMouseDown]);

  // Keep displayValue within bounds when min/max change.
  useEffect(() => {
    setDisplayValue((v) => clamp(v));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [min, max]);

  // Update CSS custom property for slider background gradient (fill to thumb center).
  useEffect(() => {
    if (!sliderRef.current) return;
    const slider = sliderRef.current;
    const thumbSize = 25; // thumb width in pixels
    const sliderWidth = slider.offsetWidth;
    if (sliderWidth <= 0) return;

    // Calculate the effective track width (excluding thumb padding)
    const trackWidth = sliderWidth - thumbSize;
    const range = max - min;
    const normalizedValue = range === 0 ? 0 : (displayValue - min) / range;

    // Calculate the thumb position as a percentage of the track width
    const thumbPosition = normalizedValue * trackWidth + thumbSize / 2;

    // Convert to percentage of total slider width
    const progress = (thumbPosition / sliderWidth) * 100;

    slider.style.setProperty("--slider-progress", `${progress}%`);
  }, [displayValue, min, max]);

  // Handle value submission when mouse is released
  useEffect(() => {
    if (!isMouseDown && pendingValue !== null && onValueSubmit) {
      onValueSubmit(pendingValue);
      setPendingValue(null);
    }
  }, [isMouseDown, pendingValue, onValueSubmit]);

  return (
    <div className="slider-container">
      <input
        ref={sliderRef}
        id={id}
        className={`slider ${className}`}
        type="range"
        min={min}
        max={max}
        step={step}
        disabled={disabled}
        value={displayValue}
        onChange={(e) => {
          if (onChange) {
            const rawValue = Number(e.target.value);
            const finalValue = findNearestStickyPoint(rawValue);
            setDisplayValue(finalValue);
            onChange(finalValue);

            // Store pending value if mouse is down
            if (isMouseDown) {
              setPendingValue(finalValue);
            }
          }
        }}
        onMouseDown={() => {
          setIsMouseDown(true);
        }}
        onMouseUp={() => {
          setIsMouseDown(false);
        }}
        onTouchStart={() => {
          setIsMouseDown(true);
        }}
        onTouchEnd={() => {
          setIsMouseDown(false);
        }}
      />
      {(icon || label) && (
        <div className="slider-overlay">
          {icon}
          {label && (
            <label className="slider-label" htmlFor={id}>
              {label}
            </label>
          )}
        </div>
      )}
    </div>
  );
}
