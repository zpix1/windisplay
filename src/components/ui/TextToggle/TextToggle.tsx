import "./TextToggle.css";

type Props = {
  text: string;
  icon: React.ReactNode;
  toggled: boolean;
  disabled?: boolean;
  onClick: () => void;
};

export function TextToggle({ text, icon, toggled, disabled, onClick }: Props) {
  return (
    <div
      className={`text-toggle ${toggled ? "toggled" : ""} ${
        disabled ? "disabled" : ""
      }`}
      onClick={onClick}
    >
      <span className="icon">{icon}</span>
      <span>{text}</span>
    </div>
  );
}
