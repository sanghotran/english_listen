/** One dictated sentence within a lesson, with exact audio cut points authored by the source
 * (dailydictation.com) — not estimated client-side. */
export interface Segment {
  id: number;
  lessonId: string;
  position: number;
  content: string;
  timeStart: number;
  timeEnd: number;
}
