import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import LevelSelector, { type LevelFilter } from "../components/LevelSelector/LevelSelector";
import { useLessonStore } from "../store/lessonStore";

export default function Home() {
  const { lessons, isLoading, error, loadLessons, getLessonsByLevel } = useLessonStore();
  const [levelFilter, setLevelFilter] = useState<LevelFilter>("ALL");

  useEffect(() => {
    loadLessons();
  }, [loadLessons]);

  if (isLoading && lessons.length === 0) return <p>Loading lessons...</p>;
  if (error) return <p role="alert">Error: {error}</p>;

  const visibleLessons = getLessonsByLevel(levelFilter);

  return (
    <div className="home-page">
      <h1>English Listen</h1>
      <LevelSelector value={levelFilter} onChange={setLevelFilter} />
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
