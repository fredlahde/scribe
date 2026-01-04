/**
 * Application-wide constants for the Scribe transcription app.
 */

/**
 * Keys used for registering component callbacks
 */
export const CALLBACK_KEYS = {
  HISTORY_VIEW: "history-view",
} as const;

/**
 * Default hotkey assignments
 */
export const DEFAULT_HOTKEYS = {
  ENGLISH: "F2",
  MUTE: "F4",
} as const;

/**
 * Store keys for persisting settings
 */
export const STORE_KEYS = {
  HOTKEY: "hotkey",
  HOTKEY_DE: "hotkey_de",
  HOTKEY_MUTE: "hotkey_mute",
  MODEL_PATH: "model_path",
  AUDIO_DEVICE: "audio_device",
} as const;

/**
 * Store filename for settings persistence
 */
export const SETTINGS_STORE_FILE = "settings.json";

/**
 * Available icon names for the Icon component
 */
export const ICON_NAMES = [
  "back",
  "settings",
  "copy",
  "check",
  "trash",
  "error",
  "microphone",
  "refresh",
] as const;

/**
 * Type-safe icon name type derived from available icons
 */
export type IconName = typeof ICON_NAMES[number];

/**
 * Supported transcription languages
 */
export const LANGUAGES = {
  en: {
    code: "en",
    label: "EN",
    name: "English",
  },
  de: {
    code: "de",
    label: "DE",
    name: "German",
  },
} as const;

export type LanguageCode = keyof typeof LANGUAGES;

/**
 * Get display label for a language code
 */
export function getLanguageLabel(code: string): string {
  const lang = LANGUAGES[code as LanguageCode];
  return lang?.label ?? code.toUpperCase();
}

/**
 * Get full name for a language code
 */
export function getLanguageName(code: string): string {
  const lang = LANGUAGES[code as LanguageCode];
  return lang?.name ?? code;
}
