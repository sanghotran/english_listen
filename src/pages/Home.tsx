import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import LevelSelector from "../components/LevelSelector/LevelSelector";
import { useLessonStore } from "../store/lessonStore";

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
      <h1>English Listen</h1>
      <LevelSelector value={levelFilter} onChange={setLevelFilter} />
      <button type="button" onClick={handleRefresh} disabled={isRefreshing}>
        {isRefreshing ? "Refreshing…" : "Refresh from VOA"}
      </button>
      <ul className="lesson-list">
        {visibleLessons.map((lesson) => (
          <li key={lesson.id}>
            <Link to={`/practice/${lesson.id}`}>
              <strong>{lesson.title}</strong> — {lesson.level} · {lesson.sourceShow}
            </Link>
          </li>
        ))}
      </ul>
      <Link to="/progress">View progress</Link>
    </div>
  );
}
