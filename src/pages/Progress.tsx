import { useEffect } from "react";
import type { CSSProperties } from "react";
import { Link } from "react-router-dom";
import { useProgressStore } from "../store/progressStore";
import type { CefrLevel } from "../types/lesson";
import "./Progress.css";

const RING_COLOR: Record<CefrLevel, string> = {
  A1: "var(--teal)",
  A2: "var(--accent)",
  B1: "var(--indigo)",
};

export default function Progress() {
  const { levelProgress, loadLevelProgress } = useProgressStore();

  useEffect(() => {
    loadLevelProgress();
  }, [loadLevelProgress]);

  return (
    <div className="progress-page">
      <Link to="/" className="progress-page__back">
        ← Back
      </Link>
      <h1>Progress</h1>
      {levelProgress.length === 0 ? (
        <p>No attempts yet — go practice a lesson first.</p>
      ) : (
        <div className="progress-grid">
          {levelProgress.map((lp) => {
            const pct = Math.round(lp.averageAccuracy * 100);
            return (
              <div key={lp.level} className="level-card">
                <div
                  className="ring"
                  style={{ "--pct": pct, "--ring-color": RING_COLOR[lp.level] } as CSSProperties}
                >
                  <span>{pct}%</span>
                </div>
                <div>
                  <p className="level-card__label">{lp.level}</p>
                  <p className="level-card__count">
                    <strong>{lp.lessonsCompleted}</strong> lesson{lp.lessonsCompleted === 1 ? "" : "s"} completed
                  </p>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
