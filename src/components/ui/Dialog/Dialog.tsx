import { useEffect, ReactNode } from "react";
import "./Dialog.css";

type DialogProps = {
  isOpen: boolean;
  onClose: () => void;
  children: ReactNode;
  className?: string;
};

export function Dialog({
  isOpen,
  onClose,
  children,
  className = "",
}: DialogProps) {
  useEffect(() => {
    if (!isOpen) {
      return;
    }
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("keydown", handleEscape);
    document.body.style.overflow = "hidden";
    return () => {
      document.removeEventListener("keydown", handleEscape);
      document.body.style.overflow = "";
    };
  }, [isOpen, onClose]);

  if (!isOpen) {
    return null;
  }

  return (
    <>
      <div
        className="dialog-overlay"
        onClick={onClose}
      />
      <div
        role="dialog"
        className={`dialog-content${className ? " " + className : ""}`}
        onClick={(e) => e.stopPropagation()}
      >
        {children}
      </div>
    </>
  );
}
