import { useCallback, useState } from "react";
import { ChevronLeftIcon, ChevronRightIcon } from "../icons/ArrowIcons";
import { Dialog } from "../Dialog/Dialog";
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

  const closeMenu = useCallback(() => {
    setIsOpen(false);
  }, []);

  const openMenu = useCallback(() => {
    setIsOpen(true);
    setPage(1);
  }, []);

  return (
    <>
      <div className={`select-input ${isOpen ? "open" : ""}`}>
        <button
          type="button"
          className="button select-trigger"
          disabled={disabled}
          onClick={() => {
            if (isOpen) {
              closeMenu();
            } else {
              openMenu();
            }
          }}
          aria-haspopup="menu"
          aria-expanded={isOpen}
        >
          <span style={{ display: "flex", alignItems: "center", gap: 8 }}>
            {triggerIcon}
            <span className="slider-label">{triggerLabel}</span>
          </span>
          <span className="caret">▾</span>
        </button>
      </div>
      <Dialog isOpen={isOpen} onClose={closeMenu} className="select-menu">
        <div className="select-menu-content">
          {pageCount > 1 && (
            <>
              <div className="pager">
                <button
                  type="button"
                  className="button pager-btn"
                  onClick={() => setPage((p) => (p > 1 ? p - 1 : 1))}
                  disabled={page === 1}
                  aria-label="Previous page"
                >
                  <ChevronLeftIcon size={16} />
                </button>
                <button
                  type="button"
                  className="button pager-btn"
                  onClick={() =>
                    setPage((p) => (p < pageCount ? p + 1 : pageCount))
                  }
                  disabled={page === pageCount}
                  aria-label="Next page"
                >
                  <ChevronRightIcon size={16} />
                </button>
              </div>
              <div className="menu-separator" />
            </>
          )}
          <div className="select-items-container">
            {getItemsForPage(page).map((it) => (
              <button
                title={`CLI Key: ${it.key}`}
                key={it.key}
                type="button"
                className={`select-item ${
                  selectedLabel &&
                  selectedLabel.toLowerCase() === it.label.toLowerCase()
                    ? "active"
                    : ""
                }`}
                onClick={() => {
                  onSelect(it.key, it.label);
                  closeMenu();
                }}
                role="menuitem"
              >
                {it.icon}
                <span>{it.label}</span>
              </button>
            ))}
          </div>
        </div>
      </Dialog>
    </>
  );
}
