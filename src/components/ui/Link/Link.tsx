import { AnchorHTMLAttributes, MouseEvent } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";

type LinkProps = Omit<AnchorHTMLAttributes<HTMLAnchorElement>, "onClick"> & {
  href: string;
  onClick?: (event: MouseEvent<HTMLAnchorElement>) => void | Promise<void>;
};

export function Link({ href, onClick, ...rest }: LinkProps) {
  return (
    <a
      {...rest}
      href={href}
      onClick={async (event) => {
        event.preventDefault();
        if (onClick) {
          await onClick(event);
        }
        await openUrl(href);
      }}
    />
  );
}
