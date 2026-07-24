import type { CefrLevel } from "../../types/lesson";
import "./LevelSelector.css";

export type LevelFilter = CefrLevel | "ALL";

const LEVELS: LevelFilter[] = ["ALL", "A1", "A2", "B1"];

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
          className={level === value ? "level-selector__btn is-active" : "level-selector__btn"}
          onClick={() => onChange(level)}
        >
          {level}
        </button>
      ))}
    </div>
  );
}
