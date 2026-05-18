// ──────────────────────────────────────────────
// XML Wrapper Utility
// ──────────────────────────────────────────────

/**
 * Convert a display name to a valid XML tag slug.
 * "World Info (Before)" → "world_info_before"
 */
export function nameToXmlTag(name: string): string {
  return name
    .toLowerCase()
    .replace(/[^a-z0-9\s_-]/g, "")
    .trim()
    .replace(/[\s-]+/g, "_")
    .replace(/_+/g, "_");
}

/**
 * Wrap content in XML tags.
 *
 * @example
 * wrapInXml("instructions", "Follow these rules...")
 * // → "<instructions>\nFollow these rules...\n</instructions>"
 */
export function wrapInXml(tagName: string, content: string, attributes?: Record<string, string>): string {
  if (!content.trim()) return "";

  const attrStr = attributes
    ? " " +
      Object.entries(attributes)
        .map(([k, v]) => `${k}="${v}"`)
        .join(" ")
    : "";

  return `<${tagName}${attrStr}>\n${content}\n</${tagName}>`;
}

/**
 * Strip XML tag wrappers from content (preserving inner text).
 */
export function stripXmlTags(content: string, tagName: string): string {
  const openPattern = new RegExp(`<${tagName}[^>]*>\\s*`, "gi");
  const closePattern = new RegExp(`\\s*</${tagName}>`, "gi");
  return content.replace(openPattern, "").replace(closePattern, "");
}
