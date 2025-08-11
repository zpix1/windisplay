import "./TextToggle.css";

type Props = {
  text: string;
  icon: React.ReactNode;
  toggled: boolean;
  disabled?: boolean;
  loading?: boolean;
  onClick: () => void;
};

export function TextToggle({
  text,
  icon,
  toggled,
  disabled,
  loading = false,
  onClick,
}: Props) {
  return (
    <div
      className={`text-toggle ${toggled ? "toggled" : ""} ${
        disabled ? "disabled" : ""
      } ${loading ? "loading" : ""}`}
      onClick={onClick}
      aria-busy={loading}
    >
      <span className="icon">{icon}</span>
      <span>{text}</span>
    </div>
  );
}
