"use strict";

const fs = require("fs");

const DEFAULT_FEEDS = [
  {
    source: "Rust Blog",
    url: "https://blog.rust-lang.org/feed.xml",
  },
  {
    source: "RustSec Advisories",
    url: "https://rustsec.org/advisories/feed.xml",
  },
  {
    source: "GitHub Changelog",
    url: "https://github.blog/changelog/feed/",
  },
  {
    source: "GitHub Security",
    url: "https://github.blog/security/feed/",
  },
];

const RELEVANT_TERMS = [
  "rust",
  "cargo",
  "clippy",
  "rustdoc",
  "rustfmt",
  "rustsec",
  "github actions",
  "workflow",
  "runner",
  "nix",
  "flake",
  "nixpkgs",
  "llvm",
  "qemu",
  "ovmf",
  "uefi",
  "bootloader",
  "kernel",
  "no_std",
  "vulnerability",
  "security advisory",
];

function stripTags(value) {
  return String(value ?? "")
    .replace(/<!\[CDATA\[(.*?)\]\]>/gs, "$1")
    .replace(/<[^>]+>/g, " ")
    .replace(/\s+/g, " ")
    .trim();
}

function decodeEntities(value) {
  return String(value ?? "")
    .replace(/&amp;/g, "&")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'");
}

function textOf(block, tag) {
  const match = block.match(new RegExp(`<${tag}[^>]*>([\\s\\S]*?)</${tag}>`, "i"));
  return match ? decodeEntities(stripTags(match[1])) : "";
}

function linkOf(block) {
  const href = block.match(/<link[^>]*href=["']([^"']+)["'][^>]*\/?>/i);
  if (href) {
    return decodeEntities(href[1]).trim();
  }
  return textOf(block, "link");
}

function dateOf(block) {
  return textOf(block, "updated") || textOf(block, "published") || textOf(block, "pubDate");
}

function extractBlocks(xml, tag) {
  const blocks = [];
  const pattern = new RegExp(`<${tag}\\b[^>]*>([\\s\\S]*?)</${tag}>`, "gi");
  let match;
  while ((match = pattern.exec(xml))) {
    blocks.push(match[0]);
  }
  return blocks;
}

function parseFeedItems(xml, source) {
  const blocks = [...extractBlocks(xml, "item"), ...extractBlocks(xml, "entry")];
  return blocks
    .map((block) => {
      const title = textOf(block, "title");
      const summary = textOf(block, "summary") || textOf(block, "description") || textOf(block, "content");
      const published = dateOf(block);
      return {
        source,
        title,
        url: linkOf(block),
        published_at: published,
        summary: summary.slice(0, 600),
      };
    })
    .filter((item) => item.title || item.url);
}

function itemTime(item) {
  const time = Date.parse(item.published_at || "");
  return Number.isFinite(time) ? time : 0;
}

function inWindow(item, since, until) {
  const time = itemTime(item);
  if (!time) {
    return true;
  }
  return time >= Date.parse(since) && time < Date.parse(until);
}

function termScore(item) {
  const haystack = `${item.title} ${item.summary}`.toLowerCase();
  const matches = RELEVANT_TERMS.filter((term) => haystack.includes(term));
  return {
    score: matches.length,
    matched_terms: matches,
  };
}

async function fetchFeed(feed, fetchImpl) {
  const response = await fetchImpl(feed.url, {
    headers: {
      Accept: "application/rss+xml, application/atom+xml, application/xml, text/xml;q=0.9, */*;q=0.8",
      "User-Agent": "poison-girl-ai-doctor/1.0",
    },
    signal: AbortSignal.timeout(8000),
  });
  if (!response.ok) {
    throw new Error(`${feed.source} returned HTTP ${response.status}`);
  }
  return response.text();
}

async function collectNewsCandidates({
  since,
  until,
  outPath = "news_candidates.json",
  promptPath = "news_prompt.txt",
  fetchImpl = globalThis.fetch,
  feeds = DEFAULT_FEEDS,
}) {
  const errors = [];
  const candidates = [];

  if (typeof fetchImpl !== "function") {
    const result = {
      window: { since_utc: since, until_utc: until },
      candidates: [],
      errors: ["fetch is unavailable"],
    };
    fs.writeFileSync(outPath, `${JSON.stringify(result, null, 2)}\n`);
    fs.writeFileSync(promptPath, `${buildNewsRelevancePrompt(result)}\n`);
    return result;
  }

  for (const feed of feeds) {
    try {
      const xml = await fetchFeed(feed, fetchImpl);
      for (const item of parseFeedItems(xml, feed.source)) {
        if (!inWindow(item, since, until)) {
          continue;
        }
        const score = termScore(item);
        if (score.score === 0) {
          continue;
        }
        candidates.push({
          ...item,
          matched_terms: score.matched_terms,
        });
      }
    } catch (error) {
      errors.push(`${feed.source}: ${error && error.message ? error.message : "failed to fetch feed"}`);
    }
  }

  const deduped = [];
  const seen = new Set();
  for (const item of candidates.sort((a, b) => itemTime(b) - itemTime(a))) {
    const key = item.url || `${item.source}:${item.title}`;
    if (seen.has(key)) {
      continue;
    }
    seen.add(key);
    deduped.push(item);
  }

  const result = {
    window: { since_utc: since, until_utc: until },
    candidates: deduped.slice(0, 20),
    errors,
  };
  fs.writeFileSync(outPath, `${JSON.stringify(result, null, 2)}\n`);
  fs.writeFileSync(promptPath, `${buildNewsRelevancePrompt(result)}\n`);
  return result;
}

function buildNewsRelevancePrompt(newsContext) {
  return [
    "# General news relevance candidates",
    "",
    "Rank only the supplied candidates for likely relevance to this repository.",
    "The repository is a Rust/Nix UEFI bootloader/kernel project with CI centered on Nix flake checks.",
    "Return strict JSON only with this shape:",
    JSON.stringify(
      {
        relevant_items: [
          {
            title: "candidate title",
            url: "candidate url",
            source: "candidate source",
            relevance: "why it may matter to this repository",
            action_hint: "small follow-up or watch item",
          },
        ],
        ignored_count: 0,
      },
      null,
      2,
    ),
    "",
    "Rules:",
    "- Use only the supplied candidates; do not add news from memory.",
    "- Keep at most five relevant_items.",
    "- Include a candidate only if it could affect Rust, Cargo, Clippy, RustSec, Nix, GitHub Actions, LLVM, QEMU, OVMF, UEFI, or low-level safety policy.",
    "- If no candidate is meaningfully relevant, return an empty relevant_items array.",
    "",
    "# Candidate JSON",
    "```json",
    JSON.stringify(newsContext, null, 2),
    "```",
  ].join("\n");
}

function readText(path, fallback = "") {
  try {
    if (path && fs.existsSync(path)) {
      return fs.readFileSync(path, "utf8");
    }
  } catch (_) {
    return fallback;
  }
  return fallback;
}

function stripFence(text) {
  return String(text ?? "")
    .trim()
    .replace(/^```(?:json)?\s*/i, "")
    .replace(/\s*```$/i, "")
    .trim();
}

function parseJsonObject(text) {
  const cleaned = stripFence(text);
  if (!cleaned) {
    return null;
  }
  try {
    return JSON.parse(cleaned);
  } catch (_) {
    const start = cleaned.indexOf("{");
    const end = cleaned.lastIndexOf("}");
    if (start >= 0 && end > start) {
      return JSON.parse(cleaned.slice(start, end + 1));
    }
  }
  return null;
}

function asArray(value) {
  return Array.isArray(value) ? value.filter(Boolean) : [];
}

function normalizeNewsItem(item) {
  if (!item || typeof item !== "object") {
    return null;
  }
  const title = String(item.title || "").trim();
  const url = String(item.url || "").trim();
  if (!title && !url) {
    return null;
  }
  return {
    title: title || url,
    url,
    source: String(item.source || "unknown").trim(),
    relevance: String(item.relevance || item.reason || "").trim(),
    action_hint: String(item.action_hint || item.action || "").trim(),
  };
}

function fallbackNewsRelevance(newsContext, reason) {
  return {
    relevant_items: [],
    ignored_count: newsContext.candidates ? newsContext.candidates.length : 0,
    model_gate: {
      should_call_model: false,
      reason: reason || "news relevance model was not called",
    },
    candidates_seen: newsContext.candidates ? newsContext.candidates.length : 0,
    feed_errors: newsContext.errors || [],
  };
}

function parseNewsRelevanceResponse({ responseFile, candidateFile, modelGate }) {
  const newsContext = JSON.parse(readText(candidateFile, '{"candidates":[],"errors":[]}'));
  const skippedReason = modelGate?.should_call_model === false ? modelGate.reason : "";
  const response = readText(responseFile, "");
  if (!response.trim()) {
    return fallbackNewsRelevance(newsContext, skippedReason || "news relevance model returned no response");
  }

  try {
    const parsed = parseJsonObject(response);
    if (!parsed || typeof parsed !== "object") {
      throw new Error("invalid JSON");
    }
    const relevantItems = asArray(parsed?.relevant_items || parsed?.relevantItems)
      .map(normalizeNewsItem)
      .filter(Boolean)
      .slice(0, 5);
    return {
      relevant_items: relevantItems,
      ignored_count: Number(parsed?.ignored_count || parsed?.ignoredCount || 0) || 0,
      model_gate: {
        should_call_model: true,
        reason: "available",
      },
      candidates_seen: newsContext.candidates ? newsContext.candidates.length : 0,
      feed_errors: newsContext.errors || [],
    };
  } catch (_) {
    return fallbackNewsRelevance(newsContext, "news relevance model returned invalid JSON");
  }
}

module.exports = {
  DEFAULT_FEEDS,
  buildNewsRelevancePrompt,
  collectNewsCandidates,
  parseNewsRelevanceResponse,
};
