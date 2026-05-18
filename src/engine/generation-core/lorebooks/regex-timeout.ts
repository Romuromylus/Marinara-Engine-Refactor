import { isPatternSafe } from "../../shared/regex/regex-safety.js";

const MAX_REGEX_SCAN_TEXT_LENGTH = 100_000;

/**
 * Browser-safe lorebook regex executor.
 *
 * The original Node server used `vm.runInContext` with a hard timeout. The
 * Tauri frontend cannot synchronously interrupt JavaScript RegExp execution, so
 * this executor keeps the same call boundary while relying on the shared static
 * safety check plus a scan-size cap. Unsafe or oversized scans fail closed and
 * the keyword matcher falls back to literal matching where appropriate.
 */
export function vmRegexExecutor(regex: RegExp, text: string): boolean {
  if (text.length > MAX_REGEX_SCAN_TEXT_LENGTH || !isPatternSafe(regex.source)) {
    return false;
  }
  const flags = regex.flags.replace(/[gy]/g, "");
  return new RegExp(regex.source, flags).test(text);
}
