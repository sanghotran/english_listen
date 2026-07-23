import { useEffect } from "react";
import { Link } from "react-router-dom";
import { useLessonStore } from "../store/lessonStore";
import { useProgressStore } from "../store/progressStore";

export default function Progress() {
  const { lessons, loadLessons } = useLessonStore();
  const { getLevelProgress } = useProgressStore();

  useEffect(() => {
    loadLessons();
  }, [loadLessons]);

  const levelProgress = getLevelProgress(lessons);

  return (
    <div className="progress-page">
      <Link to="/">← Back</Link>
      <h1>Progress</h1>
      {levelProgress.length === 0 ? (
        <p>No attempts yet — go practice a lesson first.</p>
      ) : (
        <ul>
          {levelProgress.map((lp) => (
            <li key={lp.level}>
              {lp.level}: {lp.lessonsCompleted} lesson(s), average accuracy{" "}
              {Math.round(lp.averageAccuracy * 100)}%
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
