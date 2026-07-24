import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import LevelSelector from "../components/LevelSelector/LevelSelector";
import { useLessonStore } from "../store/lessonStore";
import type { CefrLevel } from "../types/lesson";
import "./Home.css";

const LEVEL_CLASS: Record<CefrLevel, string> = {
  A1: "level-pill--a1",
  A2: "level-pill--a2",
  B1: "level-pill--b1",
};

export default function Home() {
  const { lessons, isLoading, error, levelFilter, loadLessons, setLevelFilter, getLessonsByLevel, refreshFromVoa } =
    useLessonStore();
  const [isRefreshing, setIsRefreshing] = useState(false);

  useEffect(() => {
    loadLessons();
  }, [loadLessons]);

  const handleRefresh = async () => {
    setIsRefreshing(true);
    try {
      await refreshFromVoa();
    } catch {
      // error is already surfaced via the store's `error` field
    } finally {
      setIsRefreshing(false);
    }
  };

  if (isLoading && lessons.length === 0) return <p>Loading lessons...</p>;
  if (error) return <p role="alert">Error: {error}</p>;

  const visibleLessons = getLessonsByLevel(levelFilter);

  return (
    <div className="home-page">
      <div className="home-page__head">
        <h1>English Listen</h1>
        <button type="button" className="btn" onClick={handleRefresh} disabled={isRefreshing}>
          {isRefreshing && <span className="spinner" aria-hidden="true" />}
          {isRefreshing ? "Refreshing…" : "Refresh from VOA"}
        </button>
      </div>
      <LevelSelector value={levelFilter} onChange={setLevelFilter} />
      <div className="lesson-grid">
        {visibleLessons.map((lesson) => (
          <Link key={lesson.id} to={`/practice/${lesson.id}`} className="lesson-card">
            <span className={`level-pill ${LEVEL_CLASS[lesson.level]}`}>{lesson.level}</span>
            <h3>{lesson.title}</h3>
            <p className="lesson-card__meta">{lesson.sourceShow}</p>
          </Link>
        ))}
      </div>
      <Link to="/progress" className="home-page__progress-link">
        View progress
      </Link>
    </div>
  );
}
