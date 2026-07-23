import { useAudioPlayer } from "../../hooks/useAudioPlayer";
import "./AudioPlayer.css";

interface AudioPlayerProps {
  src: string;
}

const RATES = [0.75, 1, 1.25];

function formatTime(seconds: number): string {
  if (!Number.isFinite(seconds)) return "0:00";
  const m = Math.floor(seconds / 60);
  const s = Math.floor(seconds % 60);
  return `${m}:${s.toString().padStart(2, "0")}`;
}

export default function AudioPlayer({ src }: AudioPlayerProps) {
  const { audioRef, isPlaying, currentTime, duration, playbackRate, toggle, replayFromStart, seekTo, setPlaybackRate } =
    useAudioPlayer(src);

  return (
    <div className="audio-player">
      <audio ref={audioRef} src={src} preload="metadata" />
      <div className="audio-player__controls">
        <button type="button" onClick={toggle} aria-label={isPlaying ? "Pause" : "Play"}>
          {isPlaying ? "⏸" : "▶"}
        </button>
        <button type="button" onClick={replayFromStart} aria-label="Replay from start">
          ⏮ Replay
        </button>
        <select
          value={playbackRate}
          onChange={(e) => setPlaybackRate(Number(e.target.value))}
          aria-label="Playback speed"
        >
          {RATES.map((rate) => (
            <option key={rate} value={rate}>
              {rate}x
            </option>
          ))}
        </select>
      </div>
      <div className="audio-player__timeline">
        <span>{formatTime(currentTime)}</span>
        <input
          type="range"
          min={0}
          max={duration || 0}
          step={0.1}
          value={currentTime}
          onChange={(e) => seekTo(Number(e.target.value))}
        />
        <span>{formatTime(duration)}</span>
      </div>
    </div>
  );
}
