import type { ReactNode } from "react";
import "./Card.css";

interface CardProps {
  title: string;
  children: ReactNode;
}

export function Card({ title, children }: CardProps) {
  return (
    <div className="card">
      <div className="card-title">{title}</div>
      {children}
    </div>
  );
}
