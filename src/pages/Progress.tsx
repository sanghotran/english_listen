import { useEffect } from "react";
import type { CSSProperties } from "react";
import { Link } from "react-router-dom";
import { MascotBear, MascotBunny } from "../components/Mascot/Mascot";
import { useProgressStore } from "../store/progressStore";
import type { CefrLevel } from "../types/lesson";
import "./Progress.css";

const RING_COLOR: Record<CefrLevel, string> = {
  A1: "var(--matcha)",
  A2: "var(--yolk)",
  B1: "var(--taro)",
  B2: "var(--cocoa)",
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
      <div className="progress-page__title">
        <MascotBunny size={40} />
        <h1>Progress</h1>
      </div>
      {levelProgress.length === 0 ? (
        <div className="progress-empty">
          <MascotBear size={56} />
          <p>No attempts yet — go practice a lesson first.</p>
        </div>
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
