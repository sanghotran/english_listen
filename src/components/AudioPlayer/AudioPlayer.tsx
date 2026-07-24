import { type AudioSegmentBounds, useAudioPlayer } from "../../hooks/useAudioPlayer";
import "./AudioPlayer.css";

interface AudioPlayerProps {
  src: string;
  /** Clamps playback to this [startTime, endTime] window in seconds (see useAudioPlayer). Omit
   * to play the whole track. */
  bounds?: AudioSegmentBounds;
}

const RATES = [0.75, 1, 1.25];

function formatTime(seconds: number): string {
  if (!Number.isFinite(seconds)) return "0:00";
  const m = Math.floor(seconds / 60);
  const s = Math.floor(seconds % 60);
  return `${m}:${s.toString().padStart(2, "0")}`;
}

export default function AudioPlayer({ src, bounds }: AudioPlayerProps) {
  const {
    audioRef,
    isPlaying,
    currentTime,
    startTime,
    endTime,
    playbackRate,
    toggle,
    replayFromStart,
    seekTo,
    setPlaybackRate,
  } = useAudioPlayer(src, bounds);

  const elapsed = Math.max(0, currentTime - startTime);
  const segmentDuration = Math.max(0, endTime - startTime);

  return (
    <div className="audio-player">
      <audio ref={audioRef} src={src} preload="metadata" />
      <div className="audio-player__controls">
        <button
          type="button"
          className="audio-player__toggle"
          onClick={toggle}
          aria-label={isPlaying ? "Pause" : "Play"}
        >
          {isPlaying ? "⏸" : "▶"}
        </button>
        <div className="audio-player__secondary">
          <button type="button" className="audio-player__replay" onClick={replayFromStart} aria-label="Replay segment">
            ⏮ Replay
          </button>
          <select
            className="audio-player__rate"
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
      </div>
      <div className="audio-player__timeline">
        <span>{formatTime(elapsed)}</span>
        <input
          type="range"
          min={startTime}
          max={endTime || startTime}
          step={0.1}
          value={currentTime}
          onChange={(e) => seekTo(Number(e.target.value))}
        />
        <span>{formatTime(segmentDuration)}</span>
      </div>
    </div>
  );
}
