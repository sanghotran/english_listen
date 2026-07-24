import type { CefrLevel } from "../../types/lesson";
import "./ProgressTracker.css";

interface ProgressTrackerProps {
  level: CefrLevel;
  attemptCount: number;
  bestAccuracy: number | null;
}

const LEVEL_CLASS: Record<CefrLevel, string> = {
  A1: "level-pill--a1",
  A2: "level-pill--a2",
  B1: "level-pill--b1",
};

export default function ProgressTracker({ level, attemptCount, bestAccuracy }: ProgressTrackerProps) {
  return (
    <div className="progress-tracker">
      <span className={`level-pill ${LEVEL_CLASS[level]}`}>{level}</span>
      <span className="progress-tracker__sep">·</span>
      <span>
        <strong>{attemptCount}</strong> attempt{attemptCount === 1 ? "" : "s"}
      </span>
      {bestAccuracy !== null && (
        <>
          <span className="progress-tracker__sep">·</span>
          <span>
            best <strong>{Math.round(bestAccuracy * 100)}%</strong>
          </span>
        </>
      )}
    </div>
  );
}
