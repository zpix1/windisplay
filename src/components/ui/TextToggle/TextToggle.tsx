import { SectionIcon } from "../icons/SectionIcon";
import "./TextToggle.css";

type Props = {
  text: string;
  icon: React.ReactNode;
  toggled: boolean;
  isSection?: boolean;
  disabled?: boolean;
  loading?: boolean;
  /**
   * When true, removes the rounded background behind the icon.
   */
  hideIconBackground?: boolean;
  onClick: () => void;
};

export function TextToggle({
  text,
  icon,
  toggled,
  disabled,
  isSection,
  loading,
  hideIconBackground,
  onClick,
}: Props) {
  return (
    <div
      className={`text-toggle ${toggled ? "toggled" : ""} ${
        disabled ? "disabled" : ""
      } ${loading ? "loading" : ""} ${hideIconBackground ? "no-icon-bg" : ""}`}
      onClick={onClick}
      aria-busy={loading}
    >
      <span className="icon">{icon}</span>
      <span className="text">{text}</span>
      {isSection && <SectionIcon />}
    </div>
  );
}
