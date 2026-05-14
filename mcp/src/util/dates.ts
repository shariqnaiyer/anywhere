/**
 * AppleScript's `date "..."` literal does NOT accept bare keywords like "today" or
 * "tomorrow" — it wants a formal date string ("March 25, 2026"). The TUI handles
 * this client-side; we do the same here so prompts read naturally.
 *
 * Unknown strings (including ISO dates, formal English dates, and Things-aware
 * tokens that the AppleScript layer DOES accept) pass through unchanged.
 */

const MONTHS = [
  "January",
  "February",
  "March",
  "April",
  "May",
  "June",
  "July",
  "August",
  "September",
  "October",
  "November",
  "December",
];

const WEEKDAYS = [
  "sunday",
  "monday",
  "tuesday",
  "wednesday",
  "thursday",
  "friday",
  "saturday",
];

function formatCivil(d: Date): string {
  return `${MONTHS[d.getMonth()]} ${d.getDate()}, ${d.getFullYear()}`;
}

function addDays(d: Date, n: number): Date {
  const out = new Date(d);
  out.setDate(out.getDate() + n);
  return out;
}

function nextWeekday(today: Date, target: number): Date {
  // 0 = Sunday, …, 6 = Saturday. Always returns *future* date (1..7 days ahead).
  const todayDow = today.getDay();
  let delta = target - todayDow;
  if (delta <= 0) delta += 7;
  return addDays(today, delta);
}

/**
 * Translate a date keyword into an AppleScript-friendly date literal. Pass-through
 * for anything we don't recognise. Empty string returned as-is (the server treats
 * empty-string as "clear").
 */
export function normalizeDate(input: string): string {
  const s = input.trim().toLowerCase();
  if (!s) return input;

  const now = new Date();
  // Normalise to midnight so AppleScript doesn't carry a wall time we didn't ask for.
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());

  if (s === "today" || s === "now") return formatCivil(today);
  if (s === "tomorrow") return formatCivil(addDays(today, 1));
  if (s === "yesterday") return formatCivil(addDays(today, -1));
  if (s === "this weekend" || s === "weekend") {
    return formatCivil(nextWeekday(today, 6 /* Saturday */));
  }
  if (s === "next week") {
    return formatCivil(nextWeekday(today, 1 /* Monday */));
  }
  if (s === "next month") {
    const d = new Date(today.getFullYear(), today.getMonth() + 1, today.getDate());
    return formatCivil(d);
  }

  // "next <weekday>"
  const nextMatch = /^next\s+(sunday|monday|tuesday|wednesday|thursday|friday|saturday)$/.exec(s);
  if (nextMatch) {
    const idx = WEEKDAYS.indexOf(nextMatch[1]);
    return formatCivil(nextWeekday(today, idx));
  }

  // ISO date YYYY-MM-DD — translate to the English form AppleScript reliably parses.
  const isoMatch = /^(\d{4})-(\d{2})-(\d{2})$/.exec(s);
  if (isoMatch) {
    const y = parseInt(isoMatch[1], 10);
    const m = parseInt(isoMatch[2], 10) - 1;
    const d = parseInt(isoMatch[3], 10);
    if (m >= 0 && m <= 11 && d >= 1 && d <= 31) {
      return formatCivil(new Date(y, m, d));
    }
  }

  // Pass through. Reserved Things keywords like "anytime"/"someday" aren't valid
  // `date` literals; the create_task handler routes those via the `list` field
  // instead — translation up there.
  return input;
}
