import type { CefrLevel } from "../../types/lesson";

interface ProgressTrackerProps {
  level: CefrLevel;
  attemptCount: number;
  bestAccuracy: number | null;
}

export default function ProgressTracker({ level, attemptCount, bestAccuracy }: ProgressTrackerProps) {
  return (
    <div className="progress-tracker">
      <span className="progress-tracker__badge">{level}</span>
      <span>{attemptCount} attempt(s)</span>
      {bestAccuracy !== null && <span>Best: {Math.round(bestAccuracy * 100)}%</span>}
    </div>
  );
}
