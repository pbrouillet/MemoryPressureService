import type { ReactNode, MouseEventHandler } from "react";
import "./Button.css";

interface ButtonProps {
  children: ReactNode;
  primary?: boolean;
  onClick?: MouseEventHandler<HTMLButtonElement>;
}

export function Button({ children, primary, onClick }: ButtonProps) {
  return (
    <button
      className={primary ? "btn btn-primary" : "btn"}
      onClick={onClick}
    >
      {children}
    </button>
  );
}
