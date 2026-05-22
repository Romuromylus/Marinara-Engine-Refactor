import { convertFileSrc } from "@tauri-apps/api/core";
import { invokeTauri, platform } from "./tauri-client";

export const USER_BACKGROUND_URL_PREFIX = "marinara-background:";
export const GAME_ASSET_URL_PREFIX = "marinara-game-asset:";
export const LOREBOOK_IMAGE_URL_PREFIX = "marinara-lorebook-image:";
export const LOCAL_FONT_URL_PREFIX = "marinara-font:";

// On the server target the Rust binary serves managed asset files under
// /assets/<bucket>/<filename>. The desktop Tauri build still uses custom
// URL schemes that the tauri protocol-asset handler intercepts. Records
// stored before this change may still contain `marinara-*:` URLs, so the
// frontend always serializes through the per-platform `*Url(filename)`
// helpers below — and `rewriteForWeb` patches up any stored legacy URLs
// at read time.
const WEB_BUCKET_FOR_PREFIX: Record<string, string> = {
  [USER_BACKGROUND_URL_PREFIX]: "/assets/backgrounds/",
  [GAME_ASSET_URL_PREFIX]: "/assets/game-assets/",
  [LOREBOOK_IMAGE_URL_PREFIX]: "/assets/lorebook-images/",
  [LOCAL_FONT_URL_PREFIX]: "/assets/fonts/",
};

type PathResponse = { path?: string | null };

function hasScheme(value: string): boolean {
  return /^[a-z][a-z0-9+.-]*:/i.test(value);
}

function isAbsoluteFilesystemPath(value: string): boolean {
  return /^[a-z]:[\\/]/i.test(value) || value.startsWith("\\\\") || value.startsWith("/");
}

function canConvertFileSrc(): boolean {
  return typeof window !== "undefined" && !!(window as { __TAURI_INTERNALS__?: { convertFileSrc?: unknown } })
    .__TAURI_INTERNALS__?.convertFileSrc;
}

function webAssetUrl(prefix: string, filename: string): string {
  const bucket = WEB_BUCKET_FOR_PREFIX[prefix];
  // Filenames inside the bucket may contain spaces / unicode. The server's
  // ServeDir percent-decodes path segments before reading disk, so encode
  // each segment individually rather than the whole filename.
  const safe = filename
    .split("/")
    .map((segment) => encodeURIComponent(segment))
    .join("/");
  return `${bucket}${safe}`;
}

/**
 * Rewrite any persisted `marinara-*:` URL into a `/assets/<bucket>/...`
 * path when running in the browser. On Tauri this is a no-op — the protocol
 * handler still intercepts the custom scheme.
 */
function rewriteForWeb(url: string): string {
  if (!platform.isWeb) return url;
  for (const prefix of Object.keys(WEB_BUCKET_FOR_PREFIX)) {
    if (url.startsWith(prefix)) {
      return webAssetUrl(prefix, decodeURIComponent(url.slice(prefix.length)));
    }
  }
  return url;
}

export function encodeLocalAssetPath(path: string): string {
  return encodeURIComponent(path.replace(/\\/g, "/"));
}

export function decodeLocalAssetPath(path: string): string {
  try {
    return decodeURIComponent(path);
  } catch {
    return path;
  }
}

export function userBackgroundUrl(filename: string): string {
  if (platform.isWeb) return webAssetUrl(USER_BACKGROUND_URL_PREFIX, filename);
  return `${USER_BACKGROUND_URL_PREFIX}${encodeLocalAssetPath(filename)}`;
}

export function gameAssetUrl(path: string): string {
  if (platform.isWeb) return webAssetUrl(GAME_ASSET_URL_PREFIX, path);
  return `${GAME_ASSET_URL_PREFIX}${encodeLocalAssetPath(path)}`;
}

export function lorebookImageUrl(filename: string): string {
  if (platform.isWeb) return webAssetUrl(LOREBOOK_IMAGE_URL_PREFIX, filename);
  return `${LOREBOOK_IMAGE_URL_PREFIX}${encodeLocalAssetPath(filename)}`;
}

export function fontUrl(filename: string): string {
  if (platform.isWeb) return webAssetUrl(LOCAL_FONT_URL_PREFIX, filename);
  return `${LOCAL_FONT_URL_PREFIX}${encodeLocalAssetPath(filename)}`;
}

export function isManagedLocalAssetUrl(url: string | null | undefined): boolean {
  return (
    !!url &&
    (url.startsWith(USER_BACKGROUND_URL_PREFIX) ||
      url.startsWith(GAME_ASSET_URL_PREFIX) ||
      url.startsWith(LOREBOOK_IMAGE_URL_PREFIX) ||
      url.startsWith(LOCAL_FONT_URL_PREFIX))
  );
}

export function filePathToAssetUrl(path: string | null | undefined): string {
  if (!path) return "";
  if (path.startsWith("asset:") || path.startsWith("http://asset.localhost")) return path;
  if (hasScheme(path) && !isAbsoluteFilesystemPath(path)) return rewriteForWeb(path);
  // On the web target an absolute filesystem path (e.g. /data/backgrounds/x.png)
  // came back from the server's *_file_path handler. The browser cannot fetch
  // it directly; the only mapping we have is via the `marinara-*:` rewrite,
  // which requires knowing which bucket the file lives in. The caller almost
  // always already builds the URL via `userBackgroundUrl(filename)` /
  // `gameAssetUrl(path)` instead of using the absolute path; this branch only
  // protects against legacy callers — return empty so the FE renders a
  // missing-asset placeholder rather than a broken navigation.
  if (platform.isWeb) return "";
  if (!canConvertFileSrc()) return path;
  try {
    return convertFileSrc(path);
  } catch {
    return path;
  }
}

export function gameAssetFileUrlFromPath(path: string, absolutePath?: string | null): string {
  return absolutePath ? filePathToAssetUrl(absolutePath) : gameAssetUrl(path);
}

export function backgroundFileUrlFromPath(filename: string, absolutePath?: string | null): string {
  return absolutePath ? filePathToAssetUrl(absolutePath) : userBackgroundUrl(filename);
}

/**
 * `absolutePath` points at a real font file on the host filesystem and only
 * resolves to an asset URL on the Tauri desktop binary (via `convertFileSrc`).
 * On the web target we ignore it and build the `/assets/fonts/<filename>`
 * URL directly — the path lives on the server container, not the browser.
 */
export function fontFileUrlFromPath(filename: string, absolutePath?: string | null): string {
  if (platform.isWeb) return fontUrl(filename);
  return absolutePath ? filePathToAssetUrl(absolutePath) : fontUrl(filename);
}

export async function resolveGameAssetFileUrl(path: string): Promise<string> {
  const response = await invokeTauri<PathResponse>("game_assets_file_path", { path });
  return filePathToAssetUrl(response.path ?? "");
}

export async function resolveBackgroundFileUrl(filename: string): Promise<string> {
  const response = await invokeTauri<PathResponse>("background_file_path", { filename });
  return filePathToAssetUrl(response.path ?? "");
}

export async function resolveLorebookImageFileUrl(filename: string): Promise<string> {
  const response = await invokeTauri<PathResponse>("lorebook_image_file_path", { filename });
  return filePathToAssetUrl(response.path ?? "");
}

export async function resolveManagedLocalAssetUrl(url: string | null | undefined): Promise<string | null> {
  if (!url) return null;
  // Web target short-circuits the resolver entirely: the bucket URL is
  // serveable as-is by the server's /assets/* ServeDir mount; we don't
  // need to round-trip through the *_file_path handler at all.
  if (platform.isWeb) {
    for (const prefix of Object.keys(WEB_BUCKET_FOR_PREFIX)) {
      if (url.startsWith(prefix)) {
        return webAssetUrl(prefix, decodeLocalAssetPath(url.slice(prefix.length)));
      }
    }
    return url;
  }
  if (url.startsWith(USER_BACKGROUND_URL_PREFIX)) {
    return resolveBackgroundFileUrl(decodeLocalAssetPath(url.slice(USER_BACKGROUND_URL_PREFIX.length)));
  }
  if (url.startsWith(GAME_ASSET_URL_PREFIX)) {
    return resolveGameAssetFileUrl(decodeLocalAssetPath(url.slice(GAME_ASSET_URL_PREFIX.length)));
  }
  if (url.startsWith(LOREBOOK_IMAGE_URL_PREFIX)) {
    return resolveLorebookImageFileUrl(decodeLocalAssetPath(url.slice(LOREBOOK_IMAGE_URL_PREFIX.length)));
  }
  return filePathToAssetUrl(url);
}
