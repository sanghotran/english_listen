import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import LevelSelector from "../components/LevelSelector/LevelSelector";
import { MascotBunny } from "../components/Mascot/Mascot";
import { useLessonStore } from "../store/lessonStore";
import { useProgressStore } from "../store/progressStore";
import type { CefrLevel } from "../types/lesson";
import "./Home.css";

const LEVEL_CLASS: Record<CefrLevel, string> = {
  A1: "level-pill--a1",
  A2: "level-pill--a2",
  B1: "level-pill--b1",
  B2: "level-pill--b2",
};

const LEVEL_CARD_CLASS: Record<CefrLevel, string> = {
  A1: "lesson-card--a1",
  A2: "lesson-card--a2",
  B1: "lesson-card--b1",
  B2: "lesson-card--b2",
};

function HeadphoneIcon() {
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2.4} strokeLinecap="round">
      <path d="M4 15v-3a8 8 0 0116 0v3" />
      <path d="M4 15a2 2 0 002 2h1v-6H6a2 2 0 00-2 2z" />
      <path d="M20 15a2 2 0 01-2 2h-1v-6h1a2 2 0 012 2z" />
    </svg>
  );
}

export default function Home() {
  const {
    lessons,
    isLoading,
    error,
    levelFilter,
    loadLessons,
    setLevelFilter,
    getLessonsByLevel,
    refreshLessons,
    refreshProgress,
  } = useLessonStore();
  const { lessonCompletionById, loadLessonProgress } = useProgressStore();
  const [isRefreshing, setIsRefreshing] = useState(false);

  useEffect(() => {
    loadLessons();
    loadLessonProgress();
  }, [loadLessons, loadLessonProgress]);

  const handleRefresh = async () => {
    setIsRefreshing(true);
    try {
      await refreshLessons();
    } catch {
      // error is already surfaced via the store's `error` field
    } finally {
      setIsRefreshing(false);
    }
  };

  if (isLoading && lessons.length === 0) return <p>Loading lessons...</p>;
  if (error) return <p role="alert">Error: {error}</p>;

  // Stable sort: keeps the store's order within each group, just pushes finished
  // lessons (100% of segments attempted) after unfinished/not-yet-started ones.
  const visibleLessons = [...getLessonsByLevel(levelFilter)].sort((a, b) => {
    const aDone = (lessonCompletionById[a.id] ?? 0) >= 1 ? 1 : 0;
    const bDone = (lessonCompletionById[b.id] ?? 0) >= 1 ? 1 : 0;
    return aDone - bDone;
  });

  return (
    <div className="home-page">
      <div className="home-page__head">
        <div className="home-page__title">
          <MascotBunny size={44} />
          <h1>English Listen</h1>
        </div>
        <button type="button" className="btn" onClick={handleRefresh} disabled={isRefreshing}>
          {isRefreshing && <span className="spinner" aria-hidden="true" />}
          {isRefreshing ? "Refreshing…" : "Refresh lessons"}
        </button>
      </div>
      {isRefreshing && refreshProgress && (
        <div className="refresh-progress">
          <div className="refresh-progress__bar">
            <div
              className="refresh-progress__fill"
              style={{ width: `${refreshProgress.total > 0 ? (refreshProgress.processed / refreshProgress.total) * 100 : 0}%` }}
            />
          </div>
          <p className="refresh-progress__label">
            {refreshProgress.category}: {refreshProgress.processed}/{refreshProgress.total} lessons checked ·{" "}
            {refreshProgress.newCount} new
          </p>
        </div>
      )}
      <div className="home-page__tabs-row">
        <LevelSelector value={levelFilter} onChange={setLevelFilter} />
        <Link to="/progress" className="progress-link">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2.6} strokeLinecap="round" strokeLinejoin="round">
            <path d="M4 20V14M12 20V8M20 20V4" />
          </svg>
          Progress
        </Link>
      </div>
      <div className="lesson-grid">
        {visibleLessons.map((lesson) => {
          const pct = Math.round((lessonCompletionById[lesson.id] ?? 0) * 100);
          const isDone = pct >= 100;
          return (
            <Link
              key={lesson.id}
              to={`/practice/${lesson.id}`}
              className={`lesson-card ${LEVEL_CARD_CLASS[lesson.level]}${isDone ? " lesson-card--done" : ""}`}
            >
              <div className="lesson-card__head">
                <span className="lesson-icon">
                  <HeadphoneIcon />
                </span>
                <span className={`progress-badge${isDone ? " progress-badge--done" : ""}`}>
                  {isDone ? "✓ 100%" : `${pct}%`}
                </span>
              </div>
              <h3>{lesson.title}</h3>
              <p className="lesson-card__meta">{lesson.category}</p>
              <span className={`level-pill ${LEVEL_CLASS[lesson.level]}`}>{lesson.level}</span>
            </Link>
          );
        })}
      </div>
    </div>
  );
}
