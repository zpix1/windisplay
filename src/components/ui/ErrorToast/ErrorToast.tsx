import { useEffect, useState } from "react";
import "./ErrorToast.css";

interface ErrorToastProps {
  message: string;
  onClose: () => void;
}

export function ErrorToast({ message, onClose }: ErrorToastProps) {
  const [countdown, setCountdown] = useState(5);

  useEffect(() => {
    if (countdown <= 0) {
      onClose();
      return;
    }

    const timer = setTimeout(() => {
      setCountdown((prev) => prev - 1);
    }, 1000);

    return () => clearTimeout(timer);
  }, [countdown, onClose]);

  return (
    <div className="error-toast">
      <span>{message}</span>
      <button className="countdown-btn" onClick={onClose} aria-label="Close">
        <svg className="countdown-ring" viewBox="0 0 24 24">
          <circle className="countdown-ring-bg" cx="12" cy="12" r="10" />
          <circle
            className="countdown-ring-progress"
            cx="12"
            cy="12"
            r="10"
            style={{
              strokeDashoffset: 62.83 * (1 - countdown / 5),
            }}
          />
        </svg>
        <span className="countdown-number">{countdown}</span>
      </button>
    </div>
  );
}
