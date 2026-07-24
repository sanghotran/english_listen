import { useCallback, useEffect, useRef, useState } from "react";

export interface AudioPlayerState {
  isPlaying: boolean;
  currentTime: number;
  duration: number;
  playbackRate: number;
}

export interface AudioSegmentBounds {
  /** Fraction (0-1) of the track's total duration where this segment starts. */
  startFraction: number;
  /** Fraction (0-1) of the track's total duration where this segment ends. */
  endFraction: number;
}

const FULL_TRACK: AudioSegmentBounds = { startFraction: 0, endFraction: 1 };

/** Encapsulates an <audio> element: play/pause/seek/rate + current time, keyed by src.
 *
 * `bounds` clamps playback to a fraction-of-duration window: VOA gives no per-sentence
 * timestamps, so a dictation segment's audio start/end are approximated proportionally from
 * its share of the transcript's word count rather than split into separate files. Seeks to the
 * segment start when `bounds` changes, auto-pauses at its end, and clamps manual seeks/replay
 * to the window — so "Play"/"Replay" always loop just that segment's slice of the one file. */
export function useAudioPlayer(src: string | null, bounds: AudioSegmentBounds = FULL_TRACK) {
  const audioRef = useRef<HTMLAudioElement | null>(null);
  const [state, setState] = useState<AudioPlayerState>({
    isPlaying: false,
    currentTime: 0,
    duration: 0,
    playbackRate: 1,
  });

  const { startFraction, endFraction } = bounds;
  const startTime = state.duration * startFraction;
  const endTime = state.duration > 0 ? state.duration * endFraction : 0;

  useEffect(() => {
    const audio = audioRef.current;
    if (!audio) return;

    const onTimeUpdate = () => {
      if (endTime > 0 && audio.currentTime >= endTime) {
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
  }, [src, endTime]);

  useEffect(() => {
    setState({ isPlaying: false, currentTime: 0, duration: 0, playbackRate: 1 });
  }, [src]);

  // Jump to the new segment's start when navigating segments, and once duration first loads.
  useEffect(() => {
    const audio = audioRef.current;
    if (!audio || state.duration === 0) return;
    audio.pause();
    audio.currentTime = startTime;
    setState((s) => ({ ...s, currentTime: startTime }));
    // startTime itself is derived from state.duration/startFraction each render, so depending
    // on those two primitives (not the derived value) is what actually gates re-seeking.
  }, [startFraction, state.duration]);

  const play = useCallback(() => {
    const audio = audioRef.current;
    if (!audio) return;
    // Resuming right at (or past) the segment's end would just re-trigger the auto-pause on
    // the next tick — loop back to the start instead, so Play always (re)plays the segment.
    if (endTime > 0 && audio.currentTime >= endTime - 0.05) {
      audio.currentTime = startTime;
    }
    audio.play();
  }, [startTime, endTime]);

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
      const clamped = endTime > 0 ? Math.min(Math.max(seconds, startTime), endTime) : seconds;
      audioRef.current.currentTime = clamped;
    },
    [startTime, endTime],
  );

  const replayFromStart = useCallback(() => {
    seekTo(startTime);
    play();
  }, [seekTo, play, startTime]);

  const setPlaybackRate = useCallback((rate: number) => {
    if (audioRef.current) audioRef.current.playbackRate = rate;
    setState((s) => ({ ...s, playbackRate: rate }));
  }, []);

  return { audioRef, ...state, startTime, endTime, play, pause, toggle, seekTo, replayFromStart, setPlaybackRate };
}
