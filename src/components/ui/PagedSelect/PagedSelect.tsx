import { useEffect, useRef, useState } from "react";
import "./PagedSelect.css";

export type PagedSelectItem = {
  key: string;
  label: string;
  icon?: React.ReactNode;
};

type PagedSelectProps = {
  disabled?: boolean;
  triggerLabel: string;
  triggerIcon?: React.ReactNode;
  pageCount: number;
  getItemsForPage: (page: number) => ReadonlyArray<PagedSelectItem>;
  selectedLabel?: string;
  onSelect: (itemKey: string, itemLabel: string) => void;
};

export function PagedSelect({
  disabled = false,
  triggerLabel,
  triggerIcon,
  pageCount,
  getItemsForPage,
  selectedLabel,
  onSelect,
}: PagedSelectProps) {
  const [isOpen, setIsOpen] = useState<boolean>(false);
  const [page, setPage] = useState<number>(1);
  const ref = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const onDocClick = (e: MouseEvent) => {
      if (!ref.current) return;
      if (!ref.current.contains(e.target as Node)) setIsOpen(false);
    };
    if (isOpen) document.addEventListener("mousedown", onDocClick);
    return () => document.removeEventListener("mousedown", onDocClick);
  }, [isOpen]);

  return (
    <div className={`select-input ${isOpen ? "open" : ""}`} ref={ref}>
      <button
        type="button"
        className="button select-trigger"
        disabled={disabled}
        onClick={() => {
          setIsOpen((v) => !v);
          setPage(1);
        }}
        aria-haspopup="menu"
        aria-expanded={isOpen}
      >
        <span style={{ display: "flex", alignItems: "center", gap: 8 }}>
          {triggerIcon}
          <span>{triggerLabel}</span>
        </span>
        <span className="caret">▾</span>
      </button>
      {isOpen && (
        <div role="menu" className="select-menu">
          {getItemsForPage(page).map((it) => (
            <button
              key={it.key}
              type="button"
              className={`select-item ${
                selectedLabel && selectedLabel.toLowerCase() === it.label.toLowerCase()
                  ? "active"
                  : ""
              }`}
              onClick={() => {
                onSelect(it.key, it.label);
                setIsOpen(false);
              }}
              role="menuitem"
            >
              {it.icon}
              <span>{it.label}</span>
            </button>
          ))}
          <div className="menu-separator" />
          <div className="pager">
            <button
              type="button"
              className="button pager-btn"
              onClick={() => setPage((p) => (p > 1 ? p - 1 : 1))}
              disabled={page === 1}
              aria-label="Previous page"
            >
              ‹
            </button>
            <button
              type="button"
              className="button pager-btn"
              onClick={() => setPage((p) => (p < pageCount ? p + 1 : pageCount))}
              disabled={page === pageCount}
              aria-label="Next page"
            >
              ›
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
