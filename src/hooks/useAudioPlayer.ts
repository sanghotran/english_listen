import { useCallback, useEffect, useRef, useState } from "react";

export interface AudioPlayerState {
  isPlaying: boolean;
  currentTime: number;
  duration: number;
  playbackRate: number;
}

export interface AudioSegmentBounds {
  /** Seconds into the track where this segment starts. */
  startTime: number;
  /** Seconds into the track where this segment ends (Infinity = play to the end of the track). */
  endTime: number;
}

const FULL_TRACK: AudioSegmentBounds = { startTime: 0, endTime: Number.POSITIVE_INFINITY };

/** Encapsulates an <audio> element: play/pause/seek/rate + current time, keyed by src.
 *
 * `bounds` clamps playback to an exact [startTime, endTime] window in seconds — the segment's
 * own authored cut points (see types/segment.ts), not an estimate. Seeks to the segment start
 * when `bounds` changes, auto-pauses at its end, and clamps manual seeks/replay to the window —
 * so "Play"/"Replay" always loop just that segment's slice of the one audio file. */
export function useAudioPlayer(src: string | null, bounds: AudioSegmentBounds = FULL_TRACK) {
  const audioRef = useRef<HTMLAudioElement | null>(null);
  const [state, setState] = useState<AudioPlayerState>({
    isPlaying: false,
    currentTime: 0,
    duration: 0,
    playbackRate: 1,
  });

  const { startTime } = bounds;
  // Clamp to the loaded duration once known, so a stale/out-of-range end time never leaves the
  // player waiting past the real end of the file.
  const endTime = state.duration > 0 ? Math.min(bounds.endTime, state.duration) : bounds.endTime;
  const hasEnd = Number.isFinite(endTime);

  useEffect(() => {
    const audio = audioRef.current;
    if (!audio) return;

    const onTimeUpdate = () => {
      if (hasEnd && audio.currentTime >= endTime) {
        audio.pause();
        audio.currentTime = endTime;
      }
      setState((s) => ({ ...s, currentTime: audio.currentTime }));
    };
    const onLoadedMetadata = () => setState((s) => ({ ...s, duration: audio.duration || 0 }));
    const onPlay = () => setState((s) => ({ ...s, isPlaying: true }));
    const onPause = () => setState((s) => ({ ...s, isPlaying: false }));
    const onEnded = () => setState((s) => ({ ...s, isPlaying: false }));

    audio.addEventListener("timeupdate", onTimeUpdate);
    audio.addEventListener("loadedmetadata", onLoadedMetadata);
    audio.addEventListener("play", onPlay);
    audio.addEventListener("pause", onPause);
    audio.addEventListener("ended", onEnded);

    return () => {
      audio.removeEventListener("timeupdate", onTimeUpdate);
      audio.removeEventListener("loadedmetadata", onLoadedMetadata);
      audio.removeEventListener("play", onPlay);
      audio.removeEventListener("pause", onPause);
      audio.removeEventListener("ended", onEnded);
    };
  }, [src, endTime, hasEnd]);

  useEffect(() => {
    setState({ isPlaying: false, currentTime: 0, duration: 0, playbackRate: 1 });
  }, [src]);

  // Jump to the new segment's start when navigating segments, and once duration first loads
  // (setting currentTime before metadata is ready can get silently reset by some webviews).
  useEffect(() => {
    const audio = audioRef.current;
    if (!audio || state.duration === 0) return;
    audio.pause();
    audio.currentTime = startTime;
    setState((s) => ({ ...s, currentTime: startTime }));
  }, [startTime, state.duration]);

  const play = useCallback(() => {
    const audio = audioRef.current;
    if (!audio) return;
    // Resuming right at (or past) the segment's end would just re-trigger the auto-pause on
    // the next tick — loop back to the start instead, so Play always (re)plays the segment.
    if (hasEnd && audio.currentTime >= endTime - 0.05) {
      audio.currentTime = startTime;
    }
    audio.play();
  }, [startTime, endTime, hasEnd]);

  const pause = useCallback(() => {
    audioRef.current?.pause();
  }, []);

  const toggle = useCallback(() => {
    if (audioRef.current?.paused) play();
    else pause();
  }, [play, pause]);

  const seekTo = useCallback(
    (seconds: number) => {
      if (!audioRef.current) return;
      const clamped = hasEnd ? Math.min(Math.max(seconds, startTime), endTime) : Math.max(seconds, startTime);
      audioRef.current.currentTime = clamped;
    },
    [startTime, endTime, hasEnd],
  );

  const replayFromStart = useCallback(() => {
    seekTo(startTime);
    play();
  }, [seekTo, play, startTime]);

  const setPlaybackRate = useCallback((rate: number) => {
    if (audioRef.current) audioRef.current.playbackRate = rate;
    setState((s) => ({ ...s, playbackRate: rate }));
  }, []);

  return {
    audioRef,
    ...state,
    startTime,
    endTime: hasEnd ? endTime : state.duration,
    play,
    pause,
    toggle,
    seekTo,
    replayFromStart,
    setPlaybackRate,
  };
}
