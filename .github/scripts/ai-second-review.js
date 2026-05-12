"use strict";

const fs = require("fs");

const COMMENT_MARKER = "<!-- ai-second-review:deepseek -->";
const MAX_FILES = 80;
const MAX_PATCH_CHARS = 60000;
const MAX_PATCH_CHARS_PER_FILE = 12000;
const REVIEW_SEVERITIES = new Set(["critical", "moderate", "minor"]);
const VERDICTS = new Set(["approve", "comment", "changes_requested"]);
const RISK_LEVELS = new Set(["high", "medium", "low"]);

function stripControlText(value) {
  return String(value ?? "")
    .replace(/\u001b\[[0-9;?]*[ -/]*[@-~]/g, "")
    .replace(/[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f]/g, "");
}

function truncate(value, maxChars) {
  const text = stripControlText(value).trim();
  if (text.length <= maxChars) {
    return text;
  }
  return `${text.slice(0, Math.max(0, maxChars - 16)).trimEnd()}\n[truncated]`;
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

function writeJson(path, value) {
  fs.writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`);
}

function stripFence(text) {
  const trimmed = stripControlText(text).trim();
  return trimmed
    .replace(/^```(?:json|markdown|md)?\s*/i, "")
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
  if (Array.isArray(value)) {
    return value.map((item) => stripControlText(item).trim()).filter(Boolean);
  }
  if (typeof value === "string") {
    return value
      .split(/\r?\n/)
      .map((line) => line.replace(/^[-*]\s*/, "").trim())
      .filter(Boolean);
  }
  return [];
}

function firstLine(value, fallback = "No summary returned.", maxChars = 220) {
  const line = stripControlText(value)
    .split(/\r?\n/)
    .map((part) => part.trim())
    .find(Boolean);
  return truncate(line || fallback, maxChars);
}

function normalizeChoice(value, choices, fallback) {
  const candidate = String(value || "").toLowerCase();
  return choices.has(candidate) ? candidate : fallback;
}

function markdownList(items, emptyText = "- None") {
  const clean = asArray(items).map((item) => truncate(item, 500));
  if (!clean.length) {
    return emptyText;
  }
  return clean.map((item) => `- ${item}`).join("\n");
}

function escapeTableCell(value) {
  return stripControlText(value)
    .replace(/\|/g, "\\|")
    .replace(/\r?\n/g, "<br>")
    .trim();
}

function escapeLinkLabel(value) {
  return stripControlText(value)
    .replace(/\\/g, "\\\\")
    .replace(/\[/g, "\\[")
    .replace(/\]/g, "\\]")
    .trim();
}

function markdownLink(label, url) {
  const cleanUrl = stripControlText(url).trim();
  const cleanLabel = escapeLinkLabel(label);
  return cleanUrl ? `[${cleanLabel || cleanUrl}](${cleanUrl})` : cleanLabel || "unavailable";
}

function codeValue(value) {
  const clean = stripControlText(value || "unknown").replace(/`/g, "'").trim();
  return `\`${clean || "unknown"}\``;
}

function markdownTable(headers, rows) {
  const header = `| ${headers.map(escapeTableCell).join(" | ")} |`;
  const divider = `| ${headers.map(() => "---").join(" | ")} |`;
  const body = rows.map((row) => `| ${row.map(escapeTableCell).join(" | ")} |`);
  return [header, divider, ...body].join("\n");
}

function isDocsOnlyPath(filename) {
  return (
    /\.(md|mdx|org|txt|rst)$/i.test(filename) ||
    /^docs\//.test(filename) ||
    /^\.github\/(ISSUE_TEMPLATE|PULL_REQUEST_TEMPLATE)\//.test(filename) ||
    /(^|\/)(README|LICENSE|CHANGELOG|CONTRIBUTING)(\..*)?$/i.test(filename)
  );
}

function classifyReviewGate(files) {
  const names = files.map((file) => file.filename);
  if (names.length && names.every(isDocsOnlyPath)) {
    return {
      should_review: false,
      reason: "docs-only",
      risk_level: "low",
      signals: ["All changed files are documentation or repository text metadata."],
    };
  }

  const highRiskPatterns = [
    /^crates\/kernel\//,
    /^crates\/loader\//,
    /^crates\/error\/no_std\//,
    /^crates\/macro\//,
    /^flake\.nix$/,
    /^Cargo\.(toml|lock)$/,
    /^\.github\/workflows\//,
    /aarch64-unknown/,
  ];
  const mediumRiskPatterns = [/^crates\/dev\//, /^\.github\/scripts\//, /^src\//];
  const highSignals = names.filter((name) => highRiskPatterns.some((pattern) => pattern.test(name)));
  const mediumSignals = names.filter((name) => mediumRiskPatterns.some((pattern) => pattern.test(name)));

  if (highSignals.length) {
    return {
      should_review: true,
      reason: "high-risk paths changed",
      risk_level: "high",
      signals: highSignals.slice(0, 10),
    };
  }

  if (mediumSignals.length) {
    return {
      should_review: true,
      reason: "developer tooling or application paths changed",
      risk_level: "medium",
      signals: mediumSignals.slice(0, 10),
    };
  }

  return {
    should_review: true,
    reason: "non-documentation code or configuration changed",
    risk_level: "low",
    signals: names.slice(0, 10),
  };
}

function compactFile(file) {
  return {
    filename: file.filename,
    status: file.status,
    additions: file.additions,
    deletions: file.deletions,
    changes: file.changes,
    previous_filename: file.previous_filename || "",
  };
}

function buildPatchPayload(files) {
  let remaining = MAX_PATCH_CHARS;
  const patches = [];

  for (const file of files.slice(0, MAX_FILES)) {
    if (remaining <= 0) {
      break;
    }

    const maxForFile = Math.min(MAX_PATCH_CHARS_PER_FILE, remaining);
    const patch = truncate(file.patch || "(binary file, rename, or patch unavailable)", maxForFile);
    remaining -= patch.length;
    patches.push({
      ...compactFile(file),
      patch,
    });
  }

  return patches;
}

async function buildSecondReviewPrompt({ github, context, pullNumber, outPath }) {
  const pull = await github.rest.pulls.get({
    owner: context.repo.owner,
    repo: context.repo.repo,
    pull_number: pullNumber,
  });
  const files = await github.paginate(github.rest.pulls.listFiles, {
    owner: context.repo.owner,
    repo: context.repo.repo,
    pull_number: pullNumber,
    per_page: 100,
  });

  const changedFiles = files.map(compactFile);
  const reviewGate = classifyReviewGate(changedFiles);
  const reviewContext = {
    repository: `${context.repo.owner}/${context.repo.repo}`,
    pr: {
      number: pull.data.number,
      title: pull.data.title,
      author: pull.data.user?.login || "unknown",
      url: pull.data.html_url,
      base: pull.data.base?.ref,
      head: pull.data.head?.ref,
      head_sha: pull.data.head?.sha,
      draft: Boolean(pull.data.draft),
      additions: pull.data.additions,
      deletions: pull.data.deletions,
      changed_files: pull.data.changed_files,
    },
    review_gate: reviewGate,
    changed_files: changedFiles.slice(0, MAX_FILES),
    patches: buildPatchPayload(files),
  };

  writeJson("second_review_context.json", reviewContext);

  if (!reviewGate.should_review) {
    return reviewContext;
  }

  const prompt = [
    "# DeepSeek second review context",
    "",
    "You are a second-opinion reviewer for a Codex/OpenAI-authored pull request in a Rust/Nix UEFI bootloader/kernel project.",
    "Your job is to find issues that an implementation-focused first reviewer might miss.",
    "",
    "Return strict JSON only with this shape:",
    JSON.stringify(
      {
        verdict: "approve|comment|changes_requested",
        risk_level: "high|medium|low",
        summary: "one concise second-opinion read",
        findings: [
          {
            severity: "critical|moderate|minor",
            file: "path/to/file",
            line: 123,
            title: "short finding title",
            detail: "why this is a real issue",
            suggestion: "concrete fix or check",
          },
        ],
        questions: ["questions for the author, if any"],
        tests: ["targeted checks that should be run or added"],
      },
      null,
      2,
    ),
    "",
    "Review priorities:",
    "- Is the implementation over-broad or inconsistent with existing abstractions?",
    "- Does it conflict with the Rust/Nix/xtask/CI structure?",
    "- Is Rust error handling consistent with the repository policy against panicking APIs?",
    "- Are no_std, loader, kernel, macro, or aarch64 target constraints respected?",
    "- Are workflow changes thin orchestration instead of duplicated build logic?",
    "- Flag unsafe-code concerns only when there is concrete diff evidence; do not make final unsafe-safety claims.",
    "",
    "Rules:",
    "- Use findings only for actionable issues grounded in the diff.",
    "- At most eight findings.",
    "- If the diff looks acceptable, use verdict comment and an empty findings array.",
    "- Separate facts from hypotheses.",
    "- Prefer concrete tests or checks over generic advice.",
    "",
    "# Context JSON",
    "```json",
    JSON.stringify(reviewContext, null, 2),
    "```",
  ].join("\n");

  fs.writeFileSync(outPath, `${prompt}\n`);
  return reviewContext;
}

function normalizeFinding(value) {
  const source = value && typeof value === "object" ? value : {};
  const file = stripControlText(source.file || source.path || "").trim();
  const line = Number(source.line || source.start_line || 0) || 0;
  const title = firstLine(source.title || source.headline, "Finding", 160);
  const detail = truncate(source.detail || source.reason || source.body || "", 800);
  const suggestion = truncate(source.suggestion || source.fix || source.next_action || "", 500);

  return {
    severity: normalizeChoice(source.severity, REVIEW_SEVERITIES, "moderate"),
    file,
    line,
    title,
    detail: detail || "No detail returned.",
    suggestion: suggestion || "Inspect this change before merging.",
  };
}

function fallbackReview(message) {
  return {
    verdict: "comment",
    risk_level: "medium",
    summary: message,
    findings: [],
    questions: [],
    tests: ["Inspect the PR diff manually because the AI second review did not return structured output."],
  };
}

function normalizeSecondReview(raw) {
  const source = raw && typeof raw === "object" ? raw : fallbackReview(
    "DeepSeek second review did not return valid JSON.",
  );
  const findings = Array.isArray(source.findings)
    ? source.findings.map(normalizeFinding).filter((finding) => finding.detail).slice(0, 8)
    : [];

  return {
    verdict: normalizeChoice(source.verdict, VERDICTS, findings.length ? "changes_requested" : "comment"),
    risk_level: normalizeChoice(source.risk_level || source.risk, RISK_LEVELS, "medium"),
    summary: firstLine(source.summary || source.read || source.headline, "No summary returned.", 260),
    findings,
    questions: asArray(source.questions).slice(0, 6),
    tests: asArray(source.tests || source.test_suggestions || source.checks).slice(0, 6),
  };
}

function parseSecondReviewResponse({ responseFile }) {
  const response = readText(responseFile, "");
  if (!response.trim()) {
    return normalizeSecondReview(fallbackReview("DeepSeek second review did not return a response."));
  }

  try {
    return normalizeSecondReview(parseJsonObject(response));
  } catch (_) {
    return normalizeSecondReview(fallbackReview("DeepSeek second review returned invalid JSON."));
  }
}

function formatFinding(finding, index) {
  const location = finding.file
    ? `${finding.file}${finding.line ? `:${finding.line}` : ""}`
    : "general";
  return [
    `#### ${index + 1}. ${finding.title}`,
    "",
    `- Severity: ${codeValue(finding.severity)}`,
    `- Location: ${codeValue(location)}`,
    `- Detail: ${finding.detail}`,
    `- Suggestion: ${finding.suggestion}`,
  ].join("\n");
}

function renderSecondReviewComment({ reviewContext, review, model }) {
  const pr = reviewContext.pr;
  const summaryTable = markdownTable(
    ["Model", "Verdict", "Risk", "Head SHA"],
    [[
      codeValue(model),
      codeValue(review.verdict),
      codeValue(review.risk_level),
      codeValue(pr.head_sha ? pr.head_sha.slice(0, 12) : "unknown"),
    ]],
  );
  const findings = review.findings.length
    ? review.findings.map(formatFinding).join("\n\n")
    : "No actionable findings were returned from the diff-only second review.";

  return [
    COMMENT_MARKER,
    "### DeepSeek PR Second Review",
    "",
    `> ${review.summary}`,
    "",
    summaryTable,
    "",
    "This is a diff-only second opinion for design and implementation risk. Treat unsafe-code conclusions as prompts for human review, not final safety judgments.",
    "",
    "### Findings",
    "",
    findings,
    "",
    "### Questions",
    "",
    markdownList(review.questions),
    "",
    "### Suggested Checks",
    "",
    markdownList(review.tests),
    "",
    `<sub>PR ${markdownLink(`#${pr.number}`, pr.url)} reviewed with ${model || "unknown model"}.</sub>`,
    "",
  ].join("\n");
}

function renderSkippedComment({ reviewContext, model }) {
  const pr = reviewContext.pr;
  return [
    COMMENT_MARKER,
    "### DeepSeek PR Second Review",
    "",
    `Skipped model review: ${reviewContext.review_gate.reason}.`,
    "",
    markdownTable(
      ["Model Call", "Gate", "Risk", "Head SHA"],
      [[
        codeValue("not called"),
        codeValue(reviewContext.review_gate.reason),
        codeValue(reviewContext.review_gate.risk_level),
        codeValue(pr.head_sha ? pr.head_sha.slice(0, 12) : "unknown"),
      ]],
    ),
    "",
    markdownList(reviewContext.review_gate.signals),
    "",
  ].join("\n");
}

async function upsertSecondReviewComment({ github, context, pullNumber, body }) {
  const comments = await github.paginate(github.rest.issues.listComments, {
    owner: context.repo.owner,
    repo: context.repo.repo,
    issue_number: pullNumber,
    per_page: 100,
  });
  const existing = comments.find((comment) =>
    comment.user?.type === "Bot" && String(comment.body || "").includes(COMMENT_MARKER),
  );

  if (existing) {
    await github.rest.issues.updateComment({
      owner: context.repo.owner,
      repo: context.repo.repo,
      comment_id: existing.id,
      body,
    });
    return { action: "updated", comment_id: existing.id };
  }

  const created = await github.rest.issues.createComment({
    owner: context.repo.owner,
    repo: context.repo.repo,
    issue_number: pullNumber,
    body,
  });
  return { action: "created", comment_id: created.data.id };
}

module.exports = {
  buildSecondReviewPrompt,
  parseSecondReviewResponse,
  renderSecondReviewComment,
  renderSkippedComment,
  upsertSecondReviewComment,
};
