import { useCallback, useEffect, useRef, useState, ReactNode } from "react";
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
  const [isMounted, setIsMounted] = useState<boolean>(false);
  const [isClosing, setIsClosing] = useState<boolean>(false);
  const closeTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const clearCloseTimeout = useCallback(() => {
    if (closeTimeoutRef.current !== null) {
      clearTimeout(closeTimeoutRef.current);
      closeTimeoutRef.current = null;
    }
  }, []);

  useEffect(() => {
    if (isOpen && !isMounted) {
      clearCloseTimeout();
      setIsMounted(true);
      setIsClosing(false);
    } else if (!isOpen && isMounted && !isClosing) {
      setIsClosing(true);
      closeTimeoutRef.current = setTimeout(() => {
        setIsMounted(false);
        setIsClosing(false);
        closeTimeoutRef.current = null;
      }, 200);
    }
  }, [isOpen, isMounted, isClosing, clearCloseTimeout]);

  useEffect(() => {
    if (!isMounted) {
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
  }, [isMounted, onClose]);

  useEffect(() => {
    return () => {
      clearCloseTimeout();
    };
  }, [clearCloseTimeout]);

  if (!isMounted) {
    return null;
  }

  return (
    <>
      <div
        className={`dialog-overlay${isClosing ? " closing" : ""}`}
        onClick={onClose}
      />
      <div
        role="dialog"
        className={`dialog-content${isClosing ? " closing" : ""}${
          className ? " " + className : ""
        }`}
        onClick={(e) => e.stopPropagation()}
      >
        {children}
      </div>
    </>
  );
}
