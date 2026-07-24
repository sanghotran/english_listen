/** Tauri commands reject with `AppError`'s serde shape (`{ kind, message }`, see
 * src-tauri/src/error.rs), not a JS `Error` instance — extract a readable message from either. */
export function errorMessage(err: unknown): string {
  if (err instanceof Error) return err.message;
  if (err && typeof err === "object" && "message" in err) return String((err as { message: unknown }).message);
  return String(err);
}
