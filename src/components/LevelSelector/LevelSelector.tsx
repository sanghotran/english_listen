import type { CefrLevel } from "../../types/lesson";
import "./LevelSelector.css";

export type LevelFilter = CefrLevel | "ALL";

const LEVELS: LevelFilter[] = ["ALL", "A1", "A2", "B1", "B2"];

const LEVEL_MODIFIER: Record<LevelFilter, string> = {
  ALL: "level-selector__btn--all",
  A1: "level-selector__btn--a1",
  A2: "level-selector__btn--a2",
  B1: "level-selector__btn--b1",
  B2: "level-selector__btn--b2",
};

interface LevelSelectorProps {
  value: LevelFilter;
  onChange: (level: LevelFilter) => void;
}

export default function LevelSelector({ value, onChange }: LevelSelectorProps) {
  return (
    <div className="level-selector" role="group" aria-label="Filter by level">
      {LEVELS.map((level) => (
        <button
          key={level}
          type="button"
          className={`level-selector__btn ${LEVEL_MODIFIER[level]}${level === value ? " is-active" : ""}`}
          onClick={() => onChange(level)}
        >
          {level === "ALL" ? "All" : level}
        </button>
      ))}
    </div>
  );
}
