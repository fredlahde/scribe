/**
 * Format a date as a human-readable relative time string.
 *
 * @param date - The date to format (ISO string or Date object)
 * @param now - Optional reference time (defaults to current time)
 * @returns Formatted string like "Just now", "5m ago", "2h ago", etc.
 *
 * @example
 * formatRelativeTime("2024-01-01T12:00:00Z") // "3d ago"
 * formatRelativeTime(new Date(Date.now() - 30000)) // "Just now"
 */
export function formatRelativeTime(
  date: string | Date,
  now: Date = new Date()
): string {
  const created = typeof date === "string" ? new Date(date) : date;
  const diffMs = now.getTime() - created.getTime();
  const diffSec = Math.floor(diffMs / 1000);
  const diffMin = Math.floor(diffSec / 60);
  const diffHour = Math.floor(diffMin / 60);
  const diffDay = Math.floor(diffHour / 24);

  if (diffSec < 60) return "Just now";
  if (diffMin < 60) return `${diffMin}m ago`;
  if (diffHour < 24) return `${diffHour}h ago`;
  if (diffDay < 7) return `${diffDay}d ago`;

  return created.toLocaleDateString();
}
