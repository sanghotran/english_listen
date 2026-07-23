import type { WordDiffToken } from "../../utils/diff";
import "./DiffViewer.css";

interface DiffViewerProps {
  tokens: WordDiffToken[];
  accuracy: number;
}

export default function DiffViewer({ tokens, accuracy }: DiffViewerProps) {
  return (
    <div className="diff-viewer">
      <p className="diff-viewer__score">Accuracy: {Math.round(accuracy * 100)}%</p>
      <p className="diff-viewer__tokens">
        {tokens.map((token, i) => (
          <span key={i} className={`diff-token diff-token--${token.status}`}>
            {token.text}{" "}
          </span>
        ))}
      </p>
      <ul className="diff-viewer__legend">
        <li className="diff-token--correct">correct</li>
        <li className="diff-token--missing">missing (in transcript, not typed)</li>
        <li className="diff-token--extra">extra (typed, not in transcript)</li>
      </ul>
    </div>
  );
}
