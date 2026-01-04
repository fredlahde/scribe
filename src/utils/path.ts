/**
 * Extracts the filename from a file path, handling both Unix and Windows path separators.
 * @param path - The file path (e.g., "/path/to/file.txt" or "C:\\path\\to\\file.txt")
 * @returns The filename, or empty string if path is null/empty
 */
export function getFilename(path: string | null): string {
  if (!path) return "";
  const parts = path.split(/[/\\]/);
  return parts.pop() || path;
}
