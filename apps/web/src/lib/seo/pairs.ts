/**
 * SEO conversion-pair catalog for free web landings.
 * Slugs are URL paths: /png-to-webp → deep-links to /convert?from=png&to=webp
 *
 * Lane C owns this module + landing routes. Lane B owns /convert UI.
 */

export type ConversionCategory = "image" | "data" | "document";

export type ConversionPair = {
  /** URL slug, e.g. "png-to-webp" */
  slug: string;
  from: string;
  to: string;
  fromLabel: string;
  toLabel: string;
  category: ConversionCategory;
  /** Available in browser WASM today (or planned for M8). */
  webReady: boolean;
  /** Short SEO headline fragment after the brand. */
  blurb: string;
};

export const CONVERSION_PAIRS: ConversionPair[] = [
  // Image — primary M8 web targets
  {
    slug: "png-to-webp",
    from: "png",
    to: "webp",
    fromLabel: "PNG",
    toLabel: "WebP",
    category: "image",
    webReady: true,
    blurb: "Shrink PNGs to WebP in your browser — files never leave your device.",
  },
  {
    slug: "jpg-to-webp",
    from: "jpg",
    to: "webp",
    fromLabel: "JPG",
    toLabel: "WebP",
    category: "image",
    webReady: true,
    blurb: "Convert JPG photos to WebP for faster pages, entirely client-side.",
  },
  {
    slug: "jpeg-to-webp",
    from: "jpeg",
    to: "webp",
    fromLabel: "JPEG",
    toLabel: "WebP",
    category: "image",
    webReady: true,
    blurb: "Free JPEG→WebP conversion that runs locally in your browser.",
  },
  {
    slug: "webp-to-png",
    from: "webp",
    to: "png",
    fromLabel: "WebP",
    toLabel: "PNG",
    category: "image",
    webReady: true,
    blurb: "Turn WebP images back into PNG without uploading anywhere.",
  },
  {
    slug: "webp-to-jpg",
    from: "webp",
    to: "jpg",
    fromLabel: "WebP",
    toLabel: "JPG",
    category: "image",
    webReady: true,
    blurb: "Export WebP to JPG in the browser — free and ad-supported.",
  },
  {
    slug: "png-to-jpg",
    from: "png",
    to: "jpg",
    fromLabel: "PNG",
    toLabel: "JPG",
    category: "image",
    webReady: true,
    blurb: "Convert PNG to JPG locally with a free web tool.",
  },
  {
    slug: "jpg-to-png",
    from: "jpg",
    to: "png",
    fromLabel: "JPG",
    toLabel: "PNG",
    category: "image",
    webReady: true,
    blurb: "Lossless-friendly JPG→PNG conversion that stays on your machine.",
  },
  {
    slug: "png-to-avif",
    from: "png",
    to: "avif",
    fromLabel: "PNG",
    toLabel: "AVIF",
    category: "image",
    webReady: true,
    blurb: "Encode PNG to AVIF in-browser for modern image delivery.",
  },
  {
    slug: "jpg-to-avif",
    from: "jpg",
    to: "avif",
    fromLabel: "JPG",
    toLabel: "AVIF",
    category: "image",
    webReady: true,
    blurb: "Convert JPG to AVIF without sending photos to a server.",
  },
  {
    slug: "avif-to-png",
    from: "avif",
    to: "png",
    fromLabel: "AVIF",
    toLabel: "PNG",
    category: "image",
    webReady: true,
    blurb: "Decode AVIF to PNG locally for maximum compatibility.",
  },
  {
    slug: "avif-to-jpg",
    from: "avif",
    to: "jpg",
    fromLabel: "AVIF",
    toLabel: "JPG",
    category: "image",
    webReady: true,
    blurb: "Open AVIF files as JPG — free, private, browser-based.",
  },
  // Image — desktop-preferred / limited web
  {
    slug: "heic-to-jpg",
    from: "heic",
    to: "jpg",
    fromLabel: "HEIC",
    toLabel: "JPG",
    category: "image",
    webReady: false,
    blurb: "HEIC→JPG is best on desktop FilePrism for large albums and batch jobs.",
  },
  {
    slug: "heic-to-png",
    from: "heic",
    to: "png",
    fromLabel: "HEIC",
    toLabel: "PNG",
    category: "image",
    webReady: false,
    blurb: "Convert iPhone HEIC shots to PNG with the desktop app’s offline engine.",
  },
  // Data
  {
    slug: "json-to-csv",
    from: "json",
    to: "csv",
    fromLabel: "JSON",
    toLabel: "CSV",
    category: "data",
    webReady: true,
    blurb: "Flatten JSON to CSV in your browser — no upload required.",
  },
  {
    slug: "csv-to-json",
    from: "csv",
    to: "json",
    fromLabel: "CSV",
    toLabel: "JSON",
    category: "data",
    webReady: true,
    blurb: "Turn CSV tables into JSON objects, entirely client-side.",
  },
  {
    slug: "json-to-yaml",
    from: "json",
    to: "yaml",
    fromLabel: "JSON",
    toLabel: "YAML",
    category: "data",
    webReady: true,
    blurb: "Convert JSON config to YAML without leaving the page.",
  },
  {
    slug: "yaml-to-json",
    from: "yaml",
    to: "json",
    fromLabel: "YAML",
    toLabel: "JSON",
    category: "data",
    webReady: true,
    blurb: "Parse YAML into JSON locally with FilePrism Web.",
  },
  {
    slug: "csv-to-xlsx",
    from: "csv",
    to: "xlsx",
    fromLabel: "CSV",
    toLabel: "XLSX",
    category: "data",
    webReady: true,
    blurb: "Build an Excel workbook from CSV in the browser.",
  },
  {
    slug: "xlsx-to-csv",
    from: "xlsx",
    to: "csv",
    fromLabel: "XLSX",
    toLabel: "CSV",
    category: "data",
    webReady: true,
    blurb: "Export spreadsheet sheets to CSV without cloud sync.",
  },
  // Documents — mostly desktop (pdfium)
  {
    slug: "md-to-html",
    from: "md",
    to: "html",
    fromLabel: "Markdown",
    toLabel: "HTML",
    category: "document",
    webReady: true,
    blurb: "Render Markdown to HTML in your browser for free.",
  },
  {
    slug: "pdf-to-png",
    from: "pdf",
    to: "png",
    fromLabel: "PDF",
    toLabel: "PNG",
    category: "document",
    webReady: false,
    blurb: "PDF page rasterization ships on desktop FilePrism (no ads, offline).",
  },
  {
    slug: "pdf-to-jpg",
    from: "pdf",
    to: "jpg",
    fromLabel: "PDF",
    toLabel: "JPG",
    category: "document",
    webReady: false,
    blurb: "Export PDF pages to JPG with the desktop app’s local pipeline.",
  },
  {
    slug: "pdf-to-text",
    from: "pdf",
    to: "txt",
    fromLabel: "PDF",
    toLabel: "Text",
    category: "document",
    webReady: false,
    blurb: "Extract PDF text offline with desktop FilePrism.",
  },
  {
    slug: "md-to-pdf",
    from: "md",
    to: "pdf",
    fromLabel: "Markdown",
    toLabel: "PDF",
    category: "document",
    webReady: false,
    blurb: "Markdown→PDF is a desktop feature for reliable local print output.",
  },
];

const bySlug = new Map(CONVERSION_PAIRS.map((p) => [p.slug, p]));

export function getPair(slug: string): ConversionPair | undefined {
  return bySlug.get(slug);
}

export function converterHref(pair: Pick<ConversionPair, "from" | "to">): string {
  const q = new URLSearchParams({ from: pair.from, to: pair.to });
  return `/convert?${q.toString()}`;
}

export function pairTitle(pair: ConversionPair): string {
  return `Convert ${pair.fromLabel} to ${pair.toLabel} free online`;
}

export function pairDescription(pair: ConversionPair): string {
  if (pair.webReady) {
    return `${pair.blurb} FilePrism Web is free and ad-supported; conversion runs in your browser.`;
  }
  return `${pair.blurb} Use desktop FilePrism for offline batch work without ads.`;
}

/** Featured pairs for the hub page and related-link rails. */
export const FEATURED_SLUGS = [
  "png-to-webp",
  "jpg-to-webp",
  "webp-to-png",
  "png-to-avif",
  "json-to-csv",
  "md-to-html",
] as const;

export function featuredPairs(): ConversionPair[] {
  return FEATURED_SLUGS.map((s) => bySlug.get(s)!).filter(Boolean);
}

export function relatedPairs(pair: ConversionPair, limit = 6): ConversionPair[] {
  return CONVERSION_PAIRS.filter(
    (p) =>
      p.slug !== pair.slug &&
      (p.category === pair.category || p.from === pair.from || p.to === pair.to),
  ).slice(0, limit);
}
