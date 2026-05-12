"use strict";

const fs = require("fs");

const SEVERITIES = new Set(["critical", "moderate", "minor", "none"]);
const CONFIDENCE = new Set(["high", "medium", "low"]);
const LENS_SEVERITIES = new Set(["high", "medium", "low", "none"]);
const SEVERITY_RANK = {
  critical: 0,
  moderate: 1,
  minor: 2,
  none: 3,
  unknown: 4,
};
const MAX_LOG_TAIL_CHARS = 16000;
const MAX_SIGNAL_LINES = 80;
const WEEKLY_LENS_MARKER = "ai-weekly-lens:v1";
const WEEKDAY_LENSES = {
  1: {
    id: "layer_boundary_health",
    name: "Layer Boundary Health",
    purpose: "Find coupling, ownership, and layer-crossing risks in changed areas.",
  },
  2: {
    id: "ci_signal_quality",
    name: "CI Signal Quality",
    purpose: "Find CI coverage, observability, flakiness, and workflow signal-quality issues.",
  },
  3: {
    id: "low_layer_safety_policy",
    name: "Low-layer Safety/Policy",
    purpose: "Find unsafe, unwrap, panic, no_std, loader, kernel, and policy drift signals.",
  },
  4: {
    id: "project_bottleneck",
    name: "Project Bottleneck",
    purpose: "Find coordination, review, ownership, or throughput bottlenecks that are hard to see day to day.",
  },
  5: {
    id: "test_gap_and_technical_debt",
    name: "Test Gap And Technical Debt",
    purpose: "Find test coverage gaps, debt accumulation, and follow-up work hidden in normal activity.",
  },
};
const LOW_LAYER_FILE_PATTERN =
  /(^|\/)(kernel|loader|boot|uefi|arch|hal|mm|memory|paging|interrupt|drivers?|firmware)(\/|$)|no_std|aarch64|x86_64/i;
const TEST_FILE_PATTERN = /(^|\/)(tests?|benches?)\/|(^|\/)test_|_test\.rs$|\.snap$/i;
const WORKFLOW_FILE_PATTERN = /^\.github\/workflows\/|^\.github\/actions\//;
const RUST_SOURCE_PATTERN = /\.rs$/;

function stripControlText(value) {
  return String(value ?? "")
    .replace(/\u001b\[[0-9;?]*[ -/]*[@-~]/g, "")
    .replace(/[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f]/g, "");
}

function sanitizeLog(value) {
  return stripControlText(value).replace(/[^\x09\x0a\x0d\x20-\x7e]/g, "");
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

function truncate(value, maxChars) {
  const text = stripControlText(value).trim();
  if (text.length <= maxChars) {
    return text;
  }
  return `${text.slice(0, Math.max(0, maxChars - 16)).trimEnd()}\n[truncated]`;
}

function firstLine(value, fallback = "No clear headline returned.", maxChars = 180) {
  const line = stripControlText(value)
    .split(/\r?\n/)
    .map((part) => part.trim())
    .find(Boolean);
  return truncate(line || fallback, maxChars);
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

function normalizeSeverity(value, fallback = "moderate") {
  const severity = String(value || "").toLowerCase();
  return SEVERITIES.has(severity) ? severity : fallback;
}

function normalizeLensSeverity(value, fallback = "low") {
  const severity = String(value || "").toLowerCase();
  return LENS_SEVERITIES.has(severity) ? severity : fallback;
}

function normalizeBoolean(value, fallback = false) {
  if (typeof value === "boolean") {
    return value;
  }
  if (typeof value === "string") {
    const clean = value.trim().toLowerCase();
    if (["true", "yes", "1"].includes(clean)) {
      return true;
    }
    if (["false", "no", "0"].includes(clean)) {
      return false;
    }
  }
  if (typeof value === "number") {
    return value !== 0;
  }
  return fallback;
}

function severityName(value) {
  const severity = String(value || "")
    .toLowerCase()
    .replace(/^severity:/, "");
  return SEVERITIES.has(severity) ? severity : "unknown";
}

function normalizeConfidence(value) {
  const confidence = String(value || "").toLowerCase();
  return CONFIDENCE.has(confidence) ? confidence : "medium";
}

function labelNames(labels) {
  return (labels || [])
    .map((label) => (typeof label === "string" ? label : label.name))
    .filter(Boolean);
}

function extractLogSignals(logText, maxLines = MAX_SIGNAL_LINES) {
  const clean = sanitizeLog(logText);
  const pattern =
    /(error(\[|:|\b)|fatal|failed|failure|panicked|panic|traceback|exception|denied|unresolved|mismatched|not found|undefined reference|could not|cannot|no such file|builder for .* failed|warning:.*-D warnings|exited with code)/i;

  const hits = [];
  clean.split(/\r?\n/).forEach((line, index) => {
    const trimmed = line.trim();
    if (!trimmed || trimmed.length > 500) {
      return;
    }
    if (pattern.test(trimmed)) {
      hits.push(`${index + 1}: ${trimmed}`);
    }
  });

  return hits.slice(-maxLines);
}

function groupFilesByTopLevel(files) {
  const groups = new Map();
  for (const file of files) {
    const area = file.filename.includes("/")
      ? file.filename.split("/")[0]
      : "(repo root)";
    const current = groups.get(area) || { area, files: 0, changes: 0, examples: [] };
    current.files += 1;
    current.changes += Number(file.changes || 0);
    if (current.examples.length < 5) {
      current.examples.push(file.filename);
    }
    groups.set(area, current);
  }

  return Array.from(groups.values()).sort((a, b) => b.changes - a.changes);
}

async function listChangedFiles({ github, context, pullNumber }) {
  const files = await github.paginate(github.rest.pulls.listFiles, {
    owner: context.repo.owner,
    repo: context.repo.repo,
    pull_number: pullNumber,
    per_page: 100,
  });

  return files.map((file) => ({
    filename: file.filename,
    status: file.status,
    additions: file.additions,
    deletions: file.deletions,
    changes: file.changes,
  }));
}

async function buildFailurePrompt({ github, context, pr, ciRunUrl, logPath, outPath }) {
  const rawLog = readText(logPath, "CI log file was not available.");
  const cleanLog = sanitizeLog(rawLog);
  const files = await listChangedFiles({ github, context, pullNumber: pr.number });
  const logSignals = extractLogSignals(cleanLog);

  const payload = {
    repository: `${context.repo.owner}/${context.repo.repo}`,
    pr: {
      number: pr.number,
      title: pr.title,
      author: pr.user?.login || "unknown",
      url: pr.html_url,
      base: pr.base?.ref,
      head: pr.head?.ref,
      head_sha: pr.head?.sha,
      draft: Boolean(pr.draft),
      additions: pr.additions,
      deletions: pr.deletions,
      changed_files: pr.changed_files,
    },
    ci: {
      workflow: "ci.yml",
      run_url: ciRunUrl || "",
    },
    changed_areas: groupFilesByTopLevel(files).slice(0, 12),
    changed_files: files.slice(0, 80),
    log_signals: logSignals,
    log_tail: cleanLog.slice(-MAX_LOG_TAIL_CHARS),
  };

  const prompt = [
    "# CI failure context",
    "",
    "Analyze this CI failure as an engineering diagnosis, not as a metric summary.",
    "Return strict JSON only with this shape:",
    JSON.stringify(
      {
        severity: "critical|moderate|minor|none",
        headline: "one-sentence read of the failure",
        likely_root_cause: "best hypothesis with caveats",
        confidence: "high|medium|low",
        evidence: ["specific log or change evidence"],
        impact: "what is blocked or at risk",
        next_actions: ["concrete checks or fixes"],
        experiment_ideas: ["small experiments that could uncover more signal"],
        owner_hint: "likely code area or role to look first",
      },
      null,
      2,
    ),
    "",
    "Rules:",
    "- Separate facts from hypotheses.",
    "- Cite log signals or changed areas as evidence.",
    "- Prefer a useful next action over repeating the error text.",
    "- Use severity none only when the CI signal is not actionable.",
    "- If evidence is weak, say so and keep confidence low.",
    "",
    "# Context JSON",
    "```json",
    JSON.stringify(payload, null, 2),
    "```",
  ].join("\n");

  fs.writeFileSync(outPath, `${prompt}\n`);
  writeJson("failure_context.json", payload);
  return payload;
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

function fallbackDiagnosis(message, ciRunUrl, details = {}) {
  const skippedReason = stripControlText(details.skippedReason || "").trim();
  return {
    severity: "moderate",
    headline: message,
    likely_root_cause: skippedReason
      ? `AI inference was skipped before diagnosis: ${skippedReason}. Inspect the deterministic CI logs and changed-file context instead.`
      : "The CI run failed, but AI Doctor could not derive a structured diagnosis from the model response.",
    confidence: "low",
    evidence: [
      ciRunUrl ? `CI run: ${ciRunUrl}` : "CI run URL was unavailable.",
      skippedReason ? `Model gate: ${skippedReason}` : "",
    ].filter(Boolean),
    impact: "The PR remains blocked until the failing CI step is inspected.",
    next_actions: [
      ciRunUrl
        ? `Open the CI run and inspect the first failing step: ${ciRunUrl}`
        : "Open the latest CI run and inspect the first failing step.",
      skippedReason
        ? "Update the repository model variable or choose an available GitHub Models catalog entry, then re-run AI Doctor."
        : "Re-run AI Doctor after logs are available if the failure is still unclear.",
    ],
    experiment_ideas: [
      "Compare the failing job with the previous successful run for the same PR head or branch.",
    ],
    owner_hint: "CI owner or the author of the touched code area.",
  };
}

function normalizeDiagnosis(raw, ciRunUrl) {
  const source = raw && typeof raw === "object" ? raw : fallbackDiagnosis(
    "CI failed, but AI Doctor did not return valid JSON.",
    ciRunUrl,
  );

  const severity = normalizeSeverity(source.severity);
  const headline = firstLine(source.headline || source.summary);
  const likelyRootCause = stripControlText(
    source.likely_root_cause ||
      source.likelyRootCause ||
      source.root_cause ||
      source.summary ||
      "No root-cause hypothesis returned.",
  ).trim();

  const evidence = asArray(source.evidence).slice(0, 6);
  const nextActions = asArray(source.next_actions || source.nextActions || source.actions).slice(0, 6);
  const experimentIdeas = asArray(
    source.experiment_ideas || source.experimentIdeas || source.ideas,
  ).slice(0, 4);

  return {
    severity,
    headline,
    likely_root_cause: likelyRootCause,
    confidence: normalizeConfidence(source.confidence),
    evidence: evidence.length
      ? evidence
      : [ciRunUrl ? `CI run: ${ciRunUrl}` : "No specific evidence was returned."],
    impact: stripControlText(source.impact || "The PR remains blocked by CI.").trim(),
    next_actions: nextActions.length
      ? nextActions
      : [
          ciRunUrl
            ? `Open the CI run and inspect the first failing step: ${ciRunUrl}`
            : "Open the latest CI run and inspect the first failing step.",
        ],
    experiment_ideas: experimentIdeas.length
      ? experimentIdeas
      : ["Re-run the failing job after the smallest suspected fix to separate flaky failure from deterministic failure."],
    owner_hint: stripControlText(source.owner_hint || source.ownerHint || "Touched code area owner.").trim(),
  };
}

function parseDiagnosisResponse({ responseFile, ciRunUrl, modelGate }) {
  const response = readText(responseFile, "");
  const skippedReason = modelGate?.should_call_model === false
    ? modelGate.reason
    : "";
  if (!response.trim()) {
    return normalizeDiagnosis(
      fallbackDiagnosis(
        skippedReason
          ? `CI failed, but AI inference was skipped: ${skippedReason}.`
          : "CI failed, but AI inference did not return a response.",
        ciRunUrl,
        { skippedReason },
      ),
      ciRunUrl,
    );
  }

  try {
    return normalizeDiagnosis(parseJsonObject(response), ciRunUrl);
  } catch (_) {
    return normalizeDiagnosis(
      fallbackDiagnosis("CI failed, but AI Doctor returned invalid JSON.", ciRunUrl),
      ciRunUrl,
    );
  }
}

function markdownList(items, emptyText = "- None") {
  const clean = asArray(items).map((item) => truncate(item, 400));
  if (!clean.length) {
    return emptyText;
  }
  return clean.map((item) => `- ${item}`).join("\n");
}

function markdownChecklist(items) {
  const clean = asArray(items).map((item) => truncate(item, 400));
  if (!clean.length) {
    return "- [ ] Inspect the failing CI run.";
  }
  return clean.map((item) => `- [ ] ${item}`).join("\n");
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

function parseDateOnly(value) {
  const match = String(value || "").match(/^(\d{4})-(\d{2})-(\d{2})$/);
  if (!match) {
    return null;
  }
  return new Date(Date.UTC(Number(match[1]), Number(match[2]) - 1, Number(match[3])));
}

function isoWeekFromDateJst(dateJst) {
  const date = parseDateOnly(dateJst);
  if (!date) {
    return "unknown";
  }
  const day = date.getUTCDay() || 7;
  date.setUTCDate(date.getUTCDate() + 4 - day);
  const yearStart = new Date(Date.UTC(date.getUTCFullYear(), 0, 1));
  const week = Math.ceil(((date - yearStart) / 86400000 + 1) / 7);
  return `${date.getUTCFullYear()}-W${String(week).padStart(2, "0")}`;
}

function lensForDate(dateJst) {
  const date = parseDateOnly(dateJst);
  if (!date) {
    return null;
  }
  return WEEKDAY_LENSES[date.getUTCDay()] || null;
}

function countRegex(text, regex) {
  const matches = String(text || "").match(regex);
  return matches ? matches.length : 0;
}

function parseSimpleLines(path) {
  return readText(path, "")
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
}

function topLevelOf(file) {
  const clean = String(file || "").trim();
  if (!clean) {
    return "unknown";
  }
  return clean.includes("/") ? clean.split("/")[0] : "(repo root)";
}

function countBy(items, keyFn) {
  const counts = new Map();
  for (const item of items) {
    const key = keyFn(item);
    counts.set(key, (counts.get(key) || 0) + 1);
  }
  return Array.from(counts.entries())
    .map(([name, count]) => ({ name, count }))
    .sort((a, b) => b.count - a.count || a.name.localeCompare(b.name));
}

function markdownTable(headers, rows) {
  const header = `| ${headers.map(escapeTableCell).join(" | ")} |`;
  const divider = `| ${headers.map(() => "---").join(" | ")} |`;
  const body = rows.map((row) => `| ${row.map(escapeTableCell).join(" | ")} |`);
  return [header, divider, ...body].join("\n");
}

function detailsBlock(summary, body) {
  const cleanBody = stripControlText(body).trim() || "None";
  return [
    `<details>`,
    `<summary>${stripControlText(summary).trim()}</summary>`,
    "",
    cleanBody,
    "",
    `</details>`,
  ].join("\n");
}

function sortedCiFailures(items) {
  return [...items].sort((a, b) => {
    const severityDiff =
      (SEVERITY_RANK[severityName(a.severity)] ?? SEVERITY_RANK.unknown) -
      (SEVERITY_RANK[severityName(b.severity)] ?? SEVERITY_RANK.unknown);
    if (severityDiff !== 0) {
      return severityDiff;
    }
    return String(a.title || "").localeCompare(String(b.title || ""));
  });
}

function formatFailureComment({ diagnosis, ciRunUrl, timestamp }) {
  const ciLink = markdownLink("run", ciRunUrl);
  const triage = markdownTable(
    ["Severity", "Confidence", "Owner Hint", "CI"],
    [[
      codeValue(diagnosis.severity),
      codeValue(diagnosis.confidence),
      diagnosis.owner_hint,
      ciLink,
    ]],
  );
  const details = [
    "#### Evidence",
    markdownList(diagnosis.evidence),
    "",
    "#### Ideas To Try",
    markdownList(diagnosis.experiment_ideas),
  ].join("\n");

  return [
    "### CI Failure Diagnosis",
    "",
    `> ${diagnosis.headline}`,
    "",
    triage,
    "",
    "#### Next Actions",
    markdownChecklist(diagnosis.next_actions),
    "",
    "#### Likely Cause",
    diagnosis.likely_root_cause,
    "",
    "#### Impact",
    diagnosis.impact,
    "",
    detailsBlock("Evidence and experiment ideas", details),
    "",
    `<sub>Generated: ${timestamp}</sub>`,
  ].join("\n");
}

function writeDiagnosisArtifacts({ diagnosis, ciRunUrl, timestamp }) {
  writeJson("ai_diagnosis.json", diagnosis);
  fs.writeFileSync("failure_comment.md", formatFailureComment({ diagnosis, ciRunUrl, timestamp }));
}

function summarizePrSearchItem(item) {
  return {
    number: item.number,
    title: item.title,
    url: item.html_url,
    author: item.user?.login || "unknown",
    state: item.state,
    labels: labelNames(item.labels),
    created_at: item.created_at,
    updated_at: item.updated_at,
    closed_at: item.closed_at,
  };
}

async function searchIssues(github, query, options = {}) {
  const res = await github.rest.search.issuesAndPullRequests({
    q: query,
    per_page: 100,
    ...options,
  });
  return {
    total_count: res.data.total_count,
    items: res.data.items,
  };
}

async function latestDoctorNote({ github, context, issueNumber }) {
  const comments = await github.rest.issues.listComments({
    owner: context.repo.owner,
    repo: context.repo.repo,
    issue_number: issueNumber,
    per_page: 20,
  });

  const notes = comments.data.filter((comment) =>
    /CI Failure (Diagnosis|Analysis)/.test(comment.body || ""),
  );
  const latest = notes.at(-1) || comments.data.at(-1);
  return latest ? truncate(latest.body || "", 1200) : "";
}

async function collectDigestContext({ github, context, since, until, dateJst }) {
  const repo = `${context.repo.owner}/${context.repo.repo}`;
  const [opened, merged, closed, failures] = await Promise.all([
    searchIssues(github, `repo:${repo} is:pr created:>=${since} created:<${until}`),
    searchIssues(github, `repo:${repo} is:pr is:merged merged:>=${since} merged:<${until}`),
    searchIssues(github, `repo:${repo} is:pr is:closed -is:merged closed:>=${since} closed:<${until}`),
    searchIssues(github, `repo:${repo} is:issue label:ai-review label:ci-failure updated:>=${since} updated:<${until}`),
  ]);

  const ciFailures = await Promise.all(
    failures.items.map(async (item) => {
      const labels = labelNames(item.labels);
      const severity = labels.find((label) => label.startsWith("severity:")) || "severity:unknown";
      return {
        number: item.number,
        title: item.title,
        url: item.html_url,
        updated_at: item.updated_at,
        severity,
        labels,
        latest_note: await latestDoctorNote({ github, context, issueNumber: item.number }),
      };
    }),
  );

  return {
    repository: repo,
    window: {
      timezone: "Asia/Tokyo",
      date_jst: dateJst,
      since_utc: since,
      until_utc: until,
    },
    pr_activity: {
      opened_total: opened.total_count,
      merged_total: merged.total_count,
      closed_not_merged_total: closed.total_count,
      opened: opened.items.map(summarizePrSearchItem).slice(0, 30),
      merged: merged.items.map(summarizePrSearchItem).slice(0, 30),
      closed_not_merged: closed.items.map(summarizePrSearchItem).slice(0, 30),
    },
    ci_failures: ciFailures,
    git: {
      commits: 0,
      top_authors: [],
      hot_directories: [],
    },
  };
}

function readCountNameFile(path, nameKey) {
  return readText(path, "")
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) => {
      const [count, ...nameParts] = line.split("\t");
      return {
        count: Number(count) || 0,
        [nameKey]: nameParts.join("\t") || "unknown",
      };
    });
}

function attachGitStats(context, { commits, topAuthorsPath, topDirsPath }) {
  context.git = {
    commits: Number(commits) || 0,
    top_authors: readCountNameFile(topAuthorsPath, "author"),
    hot_directories: readCountNameFile(topDirsPath, "directory"),
  };
  return context;
}

function parseRustTokenDelta(path) {
  const delta = {
    unsafe: { added: 0, removed: 0 },
    unwrap: { added: 0, removed: 0 },
    expect: { added: 0, removed: 0 },
    panic: { added: 0, removed: 0 },
    todo: { added: 0, removed: 0 },
  };
  const patterns = {
    unsafe: /\bunsafe\b/g,
    unwrap: /\.unwrap\s*\(|\bunwrap\s*\(/g,
    expect: /\.expect\s*\(|\bexpect\s*\(/g,
    panic: /\bpanic!\s*\(/g,
    todo: /\b(TODO|FIXME|HACK)\b/gi,
  };

  for (const line of readText(path, "").split(/\r?\n/)) {
    if (!line || line.startsWith("+++") || line.startsWith("---")) {
      continue;
    }
    const bucket = line.startsWith("+") ? "added" : line.startsWith("-") ? "removed" : "";
    if (!bucket) {
      continue;
    }
    const text = line.slice(1);
    for (const [name, pattern] of Object.entries(patterns)) {
      delta[name][bucket] += countRegex(text, pattern);
    }
  }

  return delta;
}

function parseCargoMetadata(path) {
  const raw = readText(path, "");
  if (!raw.trim()) {
    return { available: false, reason: "cargo metadata was not collected" };
  }
  try {
    const metadata = JSON.parse(raw);
    if (metadata && metadata.unavailable) {
      return {
        available: false,
        reason: stripControlText(metadata.reason || "cargo metadata unavailable"),
      };
    }
    return {
      available: true,
      packages: Array.isArray(metadata.packages) ? metadata.packages.length : 0,
      workspace_members: Array.isArray(metadata.workspace_members)
        ? metadata.workspace_members.length
        : 0,
      targets: Array.isArray(metadata.packages)
        ? metadata.packages.reduce((sum, pkg) => sum + (Array.isArray(pkg.targets) ? pkg.targets.length : 0), 0)
        : 0,
    };
  } catch (_) {
    return { available: false, reason: "cargo metadata was not valid JSON" };
  }
}

function attachChangeSignals(
  context,
  {
    changedFilesPath = "changed_files.txt",
    nameStatusPath = "changed_name_status.txt",
    numstatPath = "changed_numstat.txt",
    rustDiffPath = "rust_diff.patch",
    diffShortstatPath = "diff_shortstat.txt",
    cargoMetadataPath = "cargo_metadata.json",
  } = {},
) {
  const changedFiles = parseSimpleLines(changedFilesPath);
  const nameStatus = parseSimpleLines(nameStatusPath);
  const numstat = parseSimpleLines(numstatPath);
  const lowLayerFiles = changedFiles.filter((file) => LOW_LAYER_FILE_PATTERN.test(file));
  const workflowFiles = changedFiles.filter((file) => WORKFLOW_FILE_PATTERN.test(file));
  const testFiles = changedFiles.filter((file) => TEST_FILE_PATTERN.test(file));
  const rustSourceFiles = changedFiles.filter((file) => RUST_SOURCE_PATTERN.test(file) && !TEST_FILE_PATTERN.test(file));
  const topLevels = countBy(changedFiles, topLevelOf).slice(0, 10);

  context.change_signals = {
    changed_file_count: changedFiles.length,
    changed_files: changedFiles.slice(0, 80),
    changed_top_levels: topLevels,
    changed_name_status: nameStatus.slice(0, 80),
    changed_numstat: numstat.slice(0, 80),
    changed_low_layer_files: lowLayerFiles.slice(0, 30),
    changed_workflow_files: workflowFiles.slice(0, 30),
    changed_test_files: testFiles.slice(0, 30),
    changed_source_files: rustSourceFiles.slice(0, 50),
    low_layer_changed: lowLayerFiles.length > 0,
    workflow_changed: workflowFiles.length > 0,
    cargo_lock_changed: changedFiles.includes("Cargo.lock"),
    cargo_manifest_changed: changedFiles.some((file) => file === "Cargo.toml" || file.endsWith("/Cargo.toml")),
    flake_changed: changedFiles.some((file) => file === "flake.nix" || file === "flake.lock" || file.endsWith(".nix")),
    source_without_test_change: rustSourceFiles.length > 0 && testFiles.length === 0,
    rust_token_delta: parseRustTokenDelta(rustDiffPath),
    diff_shortstat: readText(diffShortstatPath, "").trim(),
    cargo_metadata: parseCargoMetadata(cargoMetadataPath),
  };
  return context;
}

function tokenDeltaHasAdded(delta, names) {
  return names.some((name) => Number(delta?.[name]?.added || 0) > 0);
}

function compactChangeSignals(context) {
  const change = context.change_signals || {};
  return {
    changed_file_count: change.changed_file_count || 0,
    changed_top_levels: change.changed_top_levels || [],
    changed_low_layer_files: change.changed_low_layer_files || [],
    changed_workflow_files: change.changed_workflow_files || [],
    changed_test_files: change.changed_test_files || [],
    changed_source_files: change.changed_source_files || [],
    low_layer_changed: Boolean(change.low_layer_changed),
    workflow_changed: Boolean(change.workflow_changed),
    cargo_lock_changed: Boolean(change.cargo_lock_changed),
    cargo_manifest_changed: Boolean(change.cargo_manifest_changed),
    flake_changed: Boolean(change.flake_changed),
    source_without_test_change: Boolean(change.source_without_test_change),
    rust_token_delta: change.rust_token_delta || {},
    diff_shortstat: change.diff_shortstat || "",
    cargo_metadata: change.cargo_metadata || { available: false },
  };
}

function addSignalCard(cards, kind, signal, evidence, weight = "medium") {
  const cleanEvidence = asArray(evidence).slice(0, 5);
  cards.push({
    kind,
    signal: truncate(signal, 220),
    evidence: cleanEvidence,
    weight,
  });
}

function buildWeeklyLensSignalCards(context, lensId) {
  const cards = [];
  const change = context.change_signals || {};
  const activity = context.pr_activity || {};
  const failures = context.ci_failures || [];
  const rustDelta = change.rust_token_delta || {};
  const topDirs = context.git?.hot_directories || [];
  const topAuthors = context.git?.top_authors || [];
  const opened = Number(activity.opened_total || 0);
  const merged = Number(activity.merged_total || 0);
  const closed = Number(activity.closed_not_merged_total || 0);
  const commits = Number(context.git?.commits || 0);

  if (failures.length) {
    addSignalCard(
      cards,
      "ci_failure",
      `${failures.length} CI failure issue(s) changed in the window.`,
      failures.slice(0, 3).map((item) => `${severityName(item.severity)}: #${item.number} ${item.title}`),
      "high",
    );
  }

  if (lensId === "layer_boundary_health") {
    if ((change.changed_top_levels || []).length >= 3) {
      addSignalCard(
        cards,
        "layer_spread",
        "Changes crossed three or more top-level areas.",
        (change.changed_top_levels || []).slice(0, 5).map((item) => `${item.name}: ${item.count} file(s)`),
        "medium",
      );
    }
    if ((change.changed_low_layer_files || []).length) {
      addSignalCard(
        cards,
        "low_layer_touch",
        "Low-layer files changed and may need boundary review.",
        change.changed_low_layer_files,
        "high",
      );
    }
    if (change.cargo_manifest_changed || change.cargo_lock_changed) {
      addSignalCard(
        cards,
        "dependency_surface",
        "Cargo manifest or lockfile changed, which can shift crate boundaries.",
        [
          change.cargo_manifest_changed ? "Cargo.toml changed" : "",
          change.cargo_lock_changed ? "Cargo.lock changed" : "",
        ].filter(Boolean),
        "medium",
      );
    }
  }

  if (lensId === "ci_signal_quality") {
    if ((change.changed_workflow_files || []).length || change.flake_changed) {
      addSignalCard(
        cards,
        "ci_definition_changed",
        "CI or flake definition changed.",
        [...(change.changed_workflow_files || []), change.flake_changed ? "Nix flake file changed" : ""].filter(Boolean),
        "high",
      );
    }
    if (failures.length) {
      addSignalCard(
        cards,
        "failure_signal",
        "CI failure issues provide signal to evaluate for repetition or noise.",
        failures.slice(0, 5).map((item) => item.title),
        "high",
      );
    }
    if (!failures.length && commits > 0 && !(change.changed_workflow_files || []).length && !change.flake_changed) {
      addSignalCard(
        cards,
        "quiet_ci",
        "Repository activity had no updated CI failure issue.",
        [`${commits} commit(s), ${opened + merged + closed} PR event(s)`],
        "low",
      );
    }
  }

  if (lensId === "low_layer_safety_policy") {
    if ((change.changed_low_layer_files || []).length) {
      addSignalCard(
        cards,
        "low_layer_policy_surface",
        "Low-layer files changed, so unsafe/no_std/panic policy should be checked.",
        change.changed_low_layer_files,
        "high",
      );
    }
    if (tokenDeltaHasAdded(rustDelta, ["unsafe", "unwrap", "expect", "panic", "todo"])) {
      addSignalCard(
        cards,
        "rust_policy_tokens",
        "Safety-sensitive Rust tokens were added in the diff.",
        Object.entries(rustDelta)
          .filter(([, value]) => Number(value.added || 0) > 0 || Number(value.removed || 0) > 0)
          .map(([name, value]) => `${name}: +${value.added || 0}/-${value.removed || 0}`),
        "high",
      );
    }
    if (change.source_without_test_change) {
      addSignalCard(
        cards,
        "source_without_tests",
        "Rust source changed without a matching test file change.",
        (change.changed_source_files || []).slice(0, 8),
        "medium",
      );
    }
  }

  if (lensId === "project_bottleneck") {
    if (topDirs.length) {
      addSignalCard(
        cards,
        "hot_directories",
        "Work concentrated in a small set of directories.",
        topDirs.slice(0, 5).map((item) => `${item.directory}: ${item.count}`),
        "medium",
      );
    }
    if (topAuthors.length && commits > 0 && Number(topAuthors[0].count || 0) / commits >= 0.7 && commits >= 3) {
      addSignalCard(
        cards,
        "author_concentration",
        "Most commits came from one author.",
        [`${topAuthors[0].author}: ${topAuthors[0].count}/${commits} commit(s)`],
        "medium",
      );
    }
    if (opened > merged + closed && opened >= 3) {
      addSignalCard(
        cards,
        "pr_queue_growth",
        "Opened PRs exceeded merged and closed PRs.",
        [`opened=${opened}, merged=${merged}, closed_without_merge=${closed}`],
        "medium",
      );
    }
    if ((change.changed_file_count || 0) >= 20) {
      addSignalCard(
        cards,
        "large_change_surface",
        "The change surface was broad for one daily window.",
        [`${change.changed_file_count} changed file(s)`, change.diff_shortstat || ""].filter(Boolean),
        "medium",
      );
    }
  }

  if (lensId === "test_gap_and_technical_debt") {
    if (change.source_without_test_change) {
      addSignalCard(
        cards,
        "test_gap",
        "Rust source changed without test file changes.",
        (change.changed_source_files || []).slice(0, 10),
        "high",
      );
    }
    if (tokenDeltaHasAdded(rustDelta, ["todo", "unwrap", "expect", "panic"])) {
      addSignalCard(
        cards,
        "debt_tokens",
        "Debt or policy-sensitive tokens increased.",
        Object.entries(rustDelta)
          .filter(([, value]) => Number(value.added || 0) > 0)
          .map(([name, value]) => `${name}: +${value.added || 0}/-${value.removed || 0}`),
        "medium",
      );
    }
    if ((change.changed_file_count || 0) >= 12 && !(change.changed_test_files || []).length) {
      addSignalCard(
        cards,
        "broad_untested_surface",
        "A broad change surface had no test file change.",
        [`${change.changed_file_count} changed file(s)`, change.diff_shortstat || ""].filter(Boolean),
        "medium",
      );
    }
  }

  return cards.slice(0, 12);
}

function createWeeklyLensGate(
  context,
  {
    weeklyLensModel = "",
    bottleneckModel = "",
  } = {},
) {
  const lens = lensForDate(context.window?.date_jst);
  if (!lens) {
    return {
      should_call_model: false,
      reason: "target date has no weekday lens",
      lens: null,
      signal_cards: [],
      model: "",
    };
  }

  const signalCards = buildWeeklyLensSignalCards(context, lens.id);
  const model = lens.id === "project_bottleneck" && bottleneckModel
    ? bottleneckModel
    : weeklyLensModel;
  return {
    should_call_model: signalCards.length > 0,
    reason: signalCards.length ? "" : `no ${lens.name} signal cards for target date`,
    lens,
    signal_cards: signalCards,
    model,
  };
}

function compactLensInput(context, gate) {
  const activity = context.pr_activity || {};
  return {
    repository: context.repository,
    window: context.window,
    lens: gate.lens,
    signal_cards: gate.signal_cards || [],
    pr_activity: {
      opened_total: activity.opened_total || 0,
      merged_total: activity.merged_total || 0,
      closed_not_merged_total: activity.closed_not_merged_total || 0,
    },
    ci_failures: (context.ci_failures || []).map((item) => ({
      number: item.number,
      title: item.title,
      url: item.url,
      severity: item.severity,
    })).slice(0, 10),
    git: context.git,
    change_signals: compactChangeSignals(context),
    relevant_news: (context.news?.relevant_items || []).slice(0, 5),
  };
}

function buildWeeklyLensPrompt({ context, gate }) {
  return [
    "# Daily weekday lens for weekly synthesis",
    "",
    `Lens: ${gate.lens?.name || "Unknown lens"}`,
    gate.lens?.purpose || "",
    "",
    "Your job is to organize the supplied deterministic signal cards for the weekly synthesis model.",
    "Return strict JSON only with this shape:",
    JSON.stringify(
      {
        lens: gate.lens?.id || "lens_id",
        target_date_jst: context.window?.date_jst || "YYYY-MM-DD",
        week_jst: context.window?.week_jst || "YYYY-Www",
        should_include_in_weekly: true,
        severity: "none|low|medium|high",
        summary: "one concise finding or 'no notable signal'",
        evidence: ["specific deterministic evidence"],
        risks: ["risk or uncertainty to preserve for weekly synthesis"],
        questions_for_weekly_synthesis: ["question the weekly model should consider"],
        recommended_followups: ["issue-sized follow-up candidate"],
      },
      null,
      2,
    ),
    "",
    "Rules:",
    "- Do not include chain-of-thought or hidden reasoning.",
    "- Preserve evidence that a later model can verify from this compact artifact.",
    "- If the signal cards are weak, set severity to low or none and explain the limitation.",
    "- Keep every array to at most five items.",
    "- The weekly synthesis model will not see raw diffs, so keep the evidence precise.",
    "",
    "# Compact input JSON",
    "```json",
    JSON.stringify(compactLensInput(context, gate), null, 2),
    "```",
  ].join("\n");
}

function fallbackWeeklyLensArtifact({ context, gate, reason }) {
  if (!gate?.lens) {
    return null;
  }
  const cards = gate.signal_cards || [];
  const hasCards = cards.length > 0;
  return {
    lens: gate.lens.id,
    lens_name: gate.lens.name,
    target_date_jst: context.window?.date_jst || "unknown",
    week_jst: context.window?.week_jst || isoWeekFromDateJst(context.window?.date_jst),
    should_include_in_weekly: hasCards,
    severity: hasCards ? "low" : "none",
    summary: hasCards
      ? `Deterministic ${gate.lens.name} signals were collected, but model synthesis was unavailable${reason ? `: ${reason}` : ""}.`
      : `No notable ${gate.lens.name} signals were collected.`,
    evidence: cards.slice(0, 5).map((card) => `${card.signal}: ${card.evidence.join("; ")}`),
    risks: hasCards ? ["Review the signal cards manually because lens synthesis was not available."] : [],
    questions_for_weekly_synthesis: hasCards ? [`Does this ${gate.lens.name} signal recur across the week?`] : [],
    recommended_followups: [],
    signal_cards: cards,
    model_gate: {
      should_call_model: false,
      reason: reason || gate.reason || "weekly lens model was not called",
    },
  };
}

function normalizeWeeklyLensArtifact(raw, context, gate, reason = "") {
  const fallback = fallbackWeeklyLensArtifact({ context, gate, reason });
  if (!fallback) {
    return null;
  }
  const source = raw && typeof raw === "object" ? raw : fallback;
  const cards = gate.signal_cards || [];
  return {
    lens: stripControlText(source.lens || gate.lens.id).trim() || gate.lens.id,
    lens_name: stripControlText(source.lens_name || source.lensName || gate.lens.name).trim() || gate.lens.name,
    target_date_jst: stripControlText(source.target_date_jst || source.targetDateJst || context.window?.date_jst).trim(),
    week_jst: stripControlText(source.week_jst || source.weekJst || context.window?.week_jst || isoWeekFromDateJst(context.window?.date_jst)).trim(),
    should_include_in_weekly: normalizeBoolean(
      source.should_include_in_weekly ?? source.shouldIncludeInWeekly,
      cards.length > 0,
    ),
    severity: normalizeLensSeverity(source.severity, fallback.severity),
    summary: firstLine(source.summary || source.read || fallback.summary, fallback.summary, 260),
    evidence: asArray(source.evidence).slice(0, 5).length ? asArray(source.evidence).slice(0, 5) : fallback.evidence,
    risks: asArray(source.risks).slice(0, 5),
    questions_for_weekly_synthesis: asArray(source.questions_for_weekly_synthesis || source.questions).slice(0, 5),
    recommended_followups: asArray(source.recommended_followups || source.followups || source.actions).slice(0, 5),
    signal_cards: cards,
    model_gate: {
      should_call_model: reason ? false : true,
      reason: reason || "available",
    },
  };
}

function parseWeeklyLensResponse({ responseFile, context, gate, modelGate }) {
  if (!gate?.lens) {
    return null;
  }
  const skippedReason = modelGate?.should_call_model === false ? modelGate.reason : "";
  const response = readText(responseFile, "");
  if (!response.trim()) {
    return normalizeWeeklyLensArtifact(null, context, gate, skippedReason || "weekly lens model returned no response");
  }

  try {
    return normalizeWeeklyLensArtifact(parseJsonObject(response), context, gate);
  } catch (_) {
    return normalizeWeeklyLensArtifact(null, context, gate, "weekly lens model returned invalid JSON");
  }
}

function renderWeeklyLensHiddenArtifact(artifact) {
  if (!artifact || !artifact.should_include_in_weekly) {
    return "";
  }
  const encoded = Buffer.from(JSON.stringify(artifact), "utf8").toString("base64");
  return `<!-- ${WEEKLY_LENS_MARKER}:${encoded} -->`;
}

function extractWeeklyLensArtifactsFromMarkdown(markdown) {
  const artifacts = [];
  const pattern = new RegExp(`<!--\\s*${WEEKLY_LENS_MARKER}:([A-Za-z0-9+/=]+)\\s*-->`, "g");
  let match;
  while ((match = pattern.exec(markdown || ""))) {
    try {
      const artifact = JSON.parse(Buffer.from(match[1], "base64").toString("utf8"));
      if (artifact && artifact.lens && artifact.target_date_jst) {
        artifacts.push(artifact);
      }
    } catch (_) {
      // Ignore malformed historical markers.
    }
  }
  return artifacts;
}

function artifactInWindow(artifact, since, until) {
  const time = Date.parse(`${artifact.target_date_jst}T00:00:00+09:00`);
  return Number.isFinite(time) && time >= Date.parse(since) && time < Date.parse(until);
}

async function collectWeeklyLensArtifacts({ github, context, since, until }) {
  const repo = `${context.repo.owner}/${context.repo.repo}`;
  const search = await searchIssues(
    github,
    `repo:${repo} is:issue label:ai-digest in:title "Daily AI Digest"`,
    { sort: "updated", order: "desc" },
  );
  const artifacts = [];

  for (const item of search.items.slice(0, 30)) {
    const issue = await github.rest.issues.get({
      owner: context.repo.owner,
      repo: context.repo.repo,
      issue_number: item.number,
    });
    for (const artifact of extractWeeklyLensArtifactsFromMarkdown(issue.data.body || "")) {
      if (artifact.should_include_in_weekly && artifactInWindow(artifact, since, until)) {
        artifacts.push({
          ...artifact,
          source_issue: {
            number: issue.data.number,
            url: issue.data.html_url,
            title: issue.data.title,
          },
        });
      }
    }
  }

  return artifacts.sort((a, b) =>
    String(a.target_date_jst).localeCompare(String(b.target_date_jst)) ||
    String(a.lens).localeCompare(String(b.lens)),
  );
}

function evaluateDigestModelGate(context) {
  const activity = context.pr_activity || {};
  const failures = (context.ci_failures || []).length;
  const commits = Number(context.git?.commits || 0);
  const change = context.change_signals || {};
  const relevantNews = context.news?.relevant_items?.length || 0;
  const lens = context.weekly_lens;
  const triggers = [];

  if (failures > 0) {
    triggers.push(`${failures} CI failure issue(s) changed`);
  }
  if (relevantNews > 0) {
    triggers.push(`${relevantNews} relevant general news item(s)`);
  }
  if (lens && lens.should_include_in_weekly && ["medium", "high"].includes(lens.severity)) {
    triggers.push(`${lens.lens_name || lens.lens} lens severity is ${lens.severity}`);
  }
  if (change.workflow_changed || change.flake_changed) {
    triggers.push("CI or Nix workflow definition changed");
  }
  if (tokenDeltaHasAdded(change.rust_token_delta || {}, ["unsafe", "panic"])) {
    triggers.push("safety-sensitive Rust token additions");
  }
  if (change.source_without_test_change && (change.changed_source_files || []).length >= 3) {
    triggers.push("multiple Rust source files changed without test file changes");
  }
  if (commits >= 10 || (change.changed_file_count || 0) >= 20) {
    triggers.push("unusually broad daily change surface");
  }

  const hasActivity =
    commits > 0 ||
    failures > 0 ||
    relevantNews > 0 ||
    Number(activity.opened_total || 0) > 0 ||
    Number(activity.merged_total || 0) > 0 ||
    Number(activity.closed_not_merged_total || 0) > 0 ||
    (change.changed_file_count || 0) > 0;

  return {
    should_call_model: triggers.length > 0,
    reason: triggers.length ? triggers.join("; ") : hasActivity ? "no digest model trigger in window" : "no activity in digest window",
    triggers,
  };
}

function evaluateWeeklyArchitectureGate(context) {
  const activity = context.pr_activity || {};
  const failures = (context.ci_failures || []).length;
  const commits = Number(context.git?.commits || 0);
  const change = context.change_signals || {};
  const lensArtifacts = context.weekly_lens_artifacts || [];
  const triggers = [];

  if (lensArtifacts.length > 0) {
    triggers.push(`${lensArtifacts.length} weekday lens artifact(s)`);
  }
  if (failures > 0) {
    triggers.push(`${failures} CI failure issue(s) changed`);
  }
  if (change.workflow_changed || change.flake_changed) {
    triggers.push("CI or Nix workflow definition changed");
  }
  if (change.cargo_lock_changed || change.cargo_manifest_changed) {
    triggers.push("Cargo dependency surface changed");
  }
  if (tokenDeltaHasAdded(change.rust_token_delta || {}, ["unsafe", "unwrap", "expect", "panic", "todo"])) {
    triggers.push("Rust safety/debt token delta changed");
  }
  if (change.source_without_test_change && (change.changed_source_files || []).length >= 5) {
    triggers.push("weekly source changes lack matching test changes");
  }
  if (commits >= 10 || (change.changed_file_count || 0) >= 25) {
    triggers.push("broad weekly change surface");
  }

  const hasActivity =
    commits > 0 ||
    failures > 0 ||
    Number(activity.opened_total || 0) > 0 ||
    Number(activity.merged_total || 0) > 0 ||
    Number(activity.closed_not_merged_total || 0) > 0 ||
    (change.changed_file_count || 0) > 0;

  return {
    should_call_model: triggers.length > 0,
    reason: triggers.length ? triggers.join("; ") : hasActivity ? "no weekly architecture model trigger in window" : "no activity in weekly window",
    triggers,
  };
}

function buildDigestPrompt(context) {
  return [
    "# Daily AI Doctor context",
    "",
    "Analyze this activity and return semantic engineering insight.",
    "Use previous-day repository statistics, relevant general news, and the weekday lens artifact when present.",
    "Return strict JSON only with this shape:",
    JSON.stringify(
      {
        read_of_day: "one concise read of the day",
        signals: ["patterns or changes worth noticing"],
        risks: ["risks, questions, or weak signals to watch"],
        recommended_actions: ["concrete follow-up actions"],
        experiment_ideas: ["small experiments that could uncover more signal"],
      },
      null,
      2,
    ),
    "",
    "Rules:",
    "- Do not restate the metrics as the main content; use metrics only as evidence.",
    "- News relevance is already pre-filtered; do not invent additional news.",
    "- The weekday lens is a compact first-layer artifact for weekly review; mention it only when it changes the daily read.",
    "- Separate observations from hypotheses.",
    "- If the evidence is thin, say that no clear pattern is visible yet.",
    "- Prefer concrete next actions and ideas over generic advice.",
    "- Keep each array to at most five items.",
    "",
    "# Context JSON",
    "```json",
    JSON.stringify(context, null, 2),
    "```",
  ].join("\n");
}

function formatPrItems(items) {
  if (!items.length) {
    return "- None";
  }
  return items
    .slice(0, 10)
    .map((item) => `- ${markdownLink(`#${item.number} ${item.title}`, item.url)} (@${item.author})`)
    .join("\n");
}

function formatCiFailureItems(items) {
  if (!items.length) {
    return "- None";
  }
  return sortedCiFailures(items)
    .slice(0, 10)
    .map((item) => `- ${codeValue(severityName(item.severity))} ${markdownLink(item.title, item.url)}`)
    .join("\n");
}

function formatNewsItems(news) {
  const items = news?.relevant_items || [];
  if (!items.length) {
    const errors = news?.feed_errors || [];
    return errors.length
      ? [`- No relevant news selected.`, ...errors.slice(0, 4).map((error) => `- Feed warning: ${error}`)].join("\n")
      : "- None";
  }
  return items
    .slice(0, 5)
    .map((item) => {
      const source = item.source ? ` (${item.source})` : "";
      const relevance = item.relevance ? ` - ${item.relevance}` : "";
      const action = item.action_hint ? ` Action: ${item.action_hint}` : "";
      return `- ${markdownLink(item.title, item.url)}${source}${relevance}${action}`;
    })
    .join("\n");
}

function formatWeeklyLensArtifact(artifact) {
  if (!artifact || !artifact.lens) {
    return "- None";
  }
  return [
    `- Lens: ${artifact.lens_name || artifact.lens}`,
    `- Severity: ${codeValue(artifact.severity)}`,
    `- Include in weekly: ${artifact.should_include_in_weekly ? "yes" : "no"}`,
    `- Summary: ${artifact.summary}`,
    "",
    "#### Lens Evidence",
    markdownList(artifact.evidence),
    "",
    "#### Lens Questions",
    markdownList(artifact.questions_for_weekly_synthesis),
  ].join("\n");
}

function formatWeeklyLensArtifacts(items) {
  if (!items || !items.length) {
    return "- None";
  }
  return items
    .slice(0, 10)
    .map((item) => {
      const source = item.source_issue?.url
        ? ` ${markdownLink(`#${item.source_issue.number}`, item.source_issue.url)}`
        : "";
      return `- ${codeValue(item.target_date_jst)} ${item.lens_name || item.lens} ${codeValue(item.severity)}${source}: ${item.summary}`;
    })
    .join("\n");
}

function formatChangeSignalSummary(change) {
  if (!change) {
    return "- None";
  }
  const rustDelta = change.rust_token_delta || {};
  const tokenLines = Object.entries(rustDelta)
    .filter(([, value]) => Number(value.added || 0) > 0 || Number(value.removed || 0) > 0)
    .map(([name, value]) => `${name} +${value.added || 0}/-${value.removed || 0}`);
  const rows = [
    `Changed files: ${change.changed_file_count || 0}`,
    `Low-layer files: ${(change.changed_low_layer_files || []).length}`,
    `Workflow files: ${(change.changed_workflow_files || []).length}`,
    `Source without test change: ${change.source_without_test_change ? "yes" : "no"}`,
    `Cargo lock changed: ${change.cargo_lock_changed ? "yes" : "no"}`,
    `Rust token deltas: ${tokenLines.length ? tokenLines.join(", ") : "none"}`,
  ];
  if (change.diff_shortstat) {
    rows.push(`Diff shortstat: ${change.diff_shortstat}`);
  }
  return rows.map((row) => `- ${row}`).join("\n");
}

function formatNamedCounts(items, nameKey, unit) {
  if (!items.length) {
    return "- None";
  }
  return items
    .slice(0, 10)
    .map((item) => `- ${item.count} ${unit}: ${item[nameKey]}`)
    .join("\n");
}

function fallbackDigestInsight(context) {
  const failures = context.ci_failures.length;
  const opened = context.pr_activity.opened_total;
  const merged = context.pr_activity.merged_total;
  const newsCount = context.news?.relevant_items?.length || 0;
  const lens = context.weekly_lens;
  const skippedReason = context.ai_model_gate?.should_call_model === false
    ? context.ai_model_gate.reason
    : "";
  const signals = [
    `PR flow: ${opened} opened, ${merged} merged, ${context.pr_activity.closed_not_merged_total} closed without merge.`,
    `Git activity: ${context.git.commits} commit(s).`,
  ];
  if (newsCount) {
    signals.push(`${newsCount} relevant general news item(s) were flagged for this repository.`);
  }
  if (lens?.should_include_in_weekly) {
    signals.push(`${lens.lens_name || lens.lens} lens artifact: ${lens.summary}`);
  }

  return {
    read_of_day: skippedReason
      ? `Model review skipped: ${skippedReason}. The deterministic digest is limited to PR flow, git activity, news relevance, lens artifacts, and CI failure issue counts.`
      : failures
      ? `${failures} CI failure issue(s) changed in the window. Start with repeated severity labels and the latest doctor notes.`
      : newsCount
      ? `${newsCount} relevant general news item(s) may affect repository maintenance. Review the news section before planning follow-up.`
      : "No CI failure issue was updated in this window. There is no strong failure pattern to infer from the available data.",
    signals,
    risks: failures
      ? ["CI failures are present; check whether the same severity or touched area repeats."]
      : lens?.should_include_in_weekly
      ? [`Preserve the ${lens.lens_name || lens.lens} artifact for weekly synthesis.`]
      : ["No clear CI risk is visible from this window alone."],
    recommended_actions: failures
      ? ["Assign owners to the CI failure issues above."]
      : newsCount
      ? ["Review the relevant news items and decide whether dependency, CI, or policy follow-up is needed."]
      : ["No CI follow-up needed from this digest."],
    experiment_ideas: [
      "Re-run this digest after the next CI sweep to compare whether the same areas or severities repeat.",
    ],
  };
}

function normalizeDigestInsight(raw, context) {
  const fallback = fallbackDigestInsight(context);
  const source = raw && typeof raw === "object" ? raw : fallback;
  return {
    read_of_day: firstLine(source.read_of_day || source.summary, fallback.read_of_day, 260),
    signals: asArray(source.signals).slice(0, 5).length
      ? asArray(source.signals).slice(0, 5)
      : fallback.signals,
    risks: asArray(source.risks || source.questions).slice(0, 5).length
      ? asArray(source.risks || source.questions).slice(0, 5)
      : fallback.risks,
    recommended_actions: asArray(source.recommended_actions || source.actions).slice(0, 5).length
      ? asArray(source.recommended_actions || source.actions).slice(0, 5)
      : fallback.recommended_actions,
    experiment_ideas: asArray(source.experiment_ideas || source.ideas).slice(0, 5).length
      ? asArray(source.experiment_ideas || source.ideas).slice(0, 5)
      : fallback.experiment_ideas,
  };
}

function parseDigestResponse({ responseFile, context }) {
  const response = readText(responseFile, "");
  if (!response.trim()) {
    return normalizeDigestInsight(null, context);
  }

  try {
    return normalizeDigestInsight(parseJsonObject(response), context);
  } catch (_) {
    return normalizeDigestInsight(null, context);
  }
}

function formatDigestInsight(insight) {
  return [
    "### Read of the Day",
    "",
    `> ${insight.read_of_day}`,
    "",
    "### Signals Worth Noticing",
    markdownList(insight.signals),
    "",
    "### Risks And Questions",
    markdownList(insight.risks),
    "",
    "### Follow-up Checklist",
    markdownChecklist(insight.recommended_actions),
    "",
    "### Ideas To Try",
    markdownList(insight.experiment_ideas),
  ].join("\n");
}

function renderDigestBody({ context, digestInsight, model }) {
  const insight = digestInsight || fallbackDigestInsight(context);
  const activity = context.pr_activity;
  const metricTable = markdownTable(
    ["Metric", "Count"],
    [
      ["PRs opened", activity.opened_total],
      ["PRs merged", activity.merged_total],
      ["PRs closed without merge", activity.closed_not_merged_total],
      ["Commits", context.git.commits],
      ["CI failure issues updated", context.ci_failures.length],
      ["Relevant news items", context.news?.relevant_items?.length || 0],
      ["Changed files", context.change_signals?.changed_file_count || 0],
    ],
  );
  const prDetails = [
    "#### Opened PRs",
    formatPrItems(activity.opened),
    "",
    "#### Merged PRs",
    formatPrItems(activity.merged),
  ].join("\n");
  const hotspotDetails = [
    "#### Top Authors",
    formatNamedCounts(context.git.top_authors, "author", "commits"),
    "",
    "#### Hot Directories",
    formatNamedCounts(context.git.hot_directories, "directory", "files changed"),
  ].join("\n");

  return [
    `### AI Doctor Daily Digest (${context.window.date_jst} JST Window)`,
    "",
    metricTable,
    "",
    formatDigestInsight(insight),
    "",
    "---",
    "",
    "### Evidence Snapshot",
    "",
    detailsBlock("PR activity", prDetails),
    "",
    detailsBlock("Authors and hot directories", hotspotDetails),
    "",
    detailsBlock("General news relevance", formatNewsItems(context.news)),
    "",
    detailsBlock("Weekday lens artifact", formatWeeklyLensArtifact(context.weekly_lens)),
    "",
    detailsBlock("Deterministic change signals", formatChangeSignalSummary(context.change_signals)),
    "",
    detailsBlock("CI failure issues", formatCiFailureItems(context.ci_failures)),
    "",
    `<sub>Generated by AI Doctor with ${model || "unknown model"}. Window: ${context.window.since_utc} to ${context.window.until_utc}.</sub>`,
    "",
    renderWeeklyLensHiddenArtifact(context.weekly_lens),
    "",
  ].join("\n");
}

async function collectWeeklyArchitectureContext({ github, context, since, until, weekJst }) {
  const weeklyContext = await collectDigestContext({
    github,
    context,
    since,
    until,
    dateJst: weekJst,
  });
  weeklyContext.window.kind = "weekly";
  weeklyContext.window.week_jst = weekJst;
  weeklyContext.weekly_lens_artifacts = await collectWeeklyLensArtifacts({
    github,
    context,
    since,
    until,
  });
  return weeklyContext;
}

function buildWeeklyArchitecturePrompt(context) {
  return [
    "# Weekly AI Architecture Doctor context",
    "",
    "Analyze this week of repository activity as a second-layer weekly synthesis.",
    "The first layer is the daily weekday lens artifacts. Use them as the primary qualitative input, then cross-check against deterministic weekly stats.",
    "Return strict JSON only with this shape:",
    JSON.stringify(
      {
        read_of_week: "one concise architectural read of the week",
        ci_trends: ["recurring CI failure or flake-check patterns"],
        layer_boundary_health: ["layer or ownership boundary signals"],
        ci_signal_quality: ["CI observability, coverage, noise, or flake quality signals"],
        low_layer_safety_policy: ["unsafe, unwrap, panic, no_std, loader/kernel policy signals"],
        project_bottlenecks: ["coordination, ownership, review, or throughput bottlenecks"],
        architecture_risks: ["design, coupling, no_std, loader, kernel, macro, or tooling risks"],
        technical_debt: ["debt signals or complexity that accumulated this week"],
        test_gaps: ["areas where tests or checks look thin"],
        recommended_issues: [
          {
            title: "issue-sized follow-up",
            reason: "why it matters",
            first_step: "small first action",
          },
        ],
        next_week_focus: ["prioritized focus items"],
      },
      null,
      2,
    ),
    "",
    "Rules:",
    "- Treat daily lens artifacts as first-layer summaries, not final truth.",
    "- Use CI failure notes, hot directories, PR flow, Cargo metadata, Cargo.lock/manifest changes, and Rust token deltas as cross-check evidence.",
    "- Pay special attention to flake checks, loader/kernel/no_std/macro areas, panic/unwrap policy, and dev tooling complexity.",
    "- Surface human-hard-to-notice issues: layer boundary health, CI signal quality, low-layer safety/policy, and project bottlenecks.",
    "- Separate observed facts from hypotheses.",
    "- Recommend issue-sized follow-ups, not broad projects.",
    "- If the evidence is thin, say that clearly and keep recommendations conservative.",
    "- Keep each array to at most six items.",
    "",
    "# Context JSON",
    "```json",
    JSON.stringify(context, null, 2),
    "```",
  ].join("\n");
}

function fallbackWeeklyArchitectureInsight(context) {
  const failures = context.ci_failures.length;
  const lensArtifacts = context.weekly_lens_artifacts || [];
  const skippedReason = context.ai_model_gate?.should_call_model === false
    ? context.ai_model_gate.reason
    : "";
  return {
    read_of_week: skippedReason
      ? `Model review skipped: ${skippedReason}. The weekly architecture summary is limited to deterministic PR, git, CI, change-signal, and lens artifact evidence.`
      : lensArtifacts.length
      ? `${lensArtifacts.length} weekday lens artifact(s) were collected. Review their recurring risks before cutting weekly follow-up work.`
      : failures
      ? `${failures} CI failure issue(s) changed this week. Review repeated severity labels and hot directories before cutting follow-up work.`
      : "No updated CI failure issue is visible in this weekly window. Architectural signal is limited to PR and git activity.",
    ci_trends: failures
      ? ["CI failure issues were updated this week; inspect whether the same check or area repeats."]
      : ["No recurring CI failure trend is visible from the collected issues."],
    layer_boundary_health: lensArtifacts.filter((item) => item.lens === "layer_boundary_health").map((item) => item.summary).slice(0, 4),
    ci_signal_quality: lensArtifacts.filter((item) => item.lens === "ci_signal_quality").map((item) => item.summary).slice(0, 4),
    low_layer_safety_policy: lensArtifacts.filter((item) => item.lens === "low_layer_safety_policy").map((item) => item.summary).slice(0, 4),
    project_bottlenecks: lensArtifacts.filter((item) => item.lens === "project_bottleneck").map((item) => item.summary).slice(0, 4),
    architecture_risks: ["No strong architecture risk can be inferred without more failure or PR detail."],
    technical_debt: ["No clear technical debt trend is visible from this weekly context alone."],
    test_gaps: ["Compare hot directories with existing flake checks before adding new tests."],
    recommended_issues: skippedReason ? [] : [
      {
        title: "Review weekly AI Doctor evidence",
        reason: "The model response was unavailable or unstructured.",
        first_step: "Open the CI failure issues and hot directories listed below.",
      },
    ],
    next_week_focus: ["Keep CI behavior in flake outputs and avoid duplicating build logic in workflows."],
  };
}

function normalizeRecommendedIssues(value, fallback) {
  const source = Array.isArray(value) ? value : [];
  const normalized = source
    .map((item) => {
      if (typeof item === "string") {
        return {
          title: firstLine(item, "Follow-up issue", 160),
          reason: "Recommended by weekly architecture review.",
          first_step: "Convert this recommendation into a small issue.",
        };
      }
      if (!item || typeof item !== "object") {
        return null;
      }
      return {
        title: firstLine(item.title || item.name, "Follow-up issue", 160),
        reason: truncate(item.reason || item.why || "", 500) || "No reason returned.",
        first_step: truncate(item.first_step || item.firstStep || item.action || "", 400) || "Define the smallest next check.",
      };
    })
    .filter(Boolean)
    .slice(0, 6);

  return normalized.length ? normalized : fallback;
}

function normalizeWeeklyArchitectureInsight(raw, context) {
  const fallback = fallbackWeeklyArchitectureInsight(context);
  const source = raw && typeof raw === "object" ? raw : fallback;
  return {
    read_of_week: firstLine(source.read_of_week || source.summary, fallback.read_of_week, 300),
    ci_trends: asArray(source.ci_trends || source.ciTrends).slice(0, 6).length
      ? asArray(source.ci_trends || source.ciTrends).slice(0, 6)
      : fallback.ci_trends,
    layer_boundary_health: asArray(source.layer_boundary_health || source.layerBoundaryHealth).slice(0, 6).length
      ? asArray(source.layer_boundary_health || source.layerBoundaryHealth).slice(0, 6)
      : fallback.layer_boundary_health,
    ci_signal_quality: asArray(source.ci_signal_quality || source.ciSignalQuality).slice(0, 6).length
      ? asArray(source.ci_signal_quality || source.ciSignalQuality).slice(0, 6)
      : fallback.ci_signal_quality,
    low_layer_safety_policy: asArray(source.low_layer_safety_policy || source.lowLayerSafetyPolicy).slice(0, 6).length
      ? asArray(source.low_layer_safety_policy || source.lowLayerSafetyPolicy).slice(0, 6)
      : fallback.low_layer_safety_policy,
    project_bottlenecks: asArray(source.project_bottlenecks || source.projectBottlenecks || source.bottlenecks).slice(0, 6).length
      ? asArray(source.project_bottlenecks || source.projectBottlenecks || source.bottlenecks).slice(0, 6)
      : fallback.project_bottlenecks,
    architecture_risks: asArray(source.architecture_risks || source.architectureRisks || source.risks).slice(0, 6).length
      ? asArray(source.architecture_risks || source.architectureRisks || source.risks).slice(0, 6)
      : fallback.architecture_risks,
    technical_debt: asArray(source.technical_debt || source.technicalDebt || source.debt).slice(0, 6).length
      ? asArray(source.technical_debt || source.technicalDebt || source.debt).slice(0, 6)
      : fallback.technical_debt,
    test_gaps: asArray(source.test_gaps || source.testGaps || source.tests).slice(0, 6).length
      ? asArray(source.test_gaps || source.testGaps || source.tests).slice(0, 6)
      : fallback.test_gaps,
    recommended_issues: normalizeRecommendedIssues(source.recommended_issues || source.recommendedIssues, fallback.recommended_issues),
    next_week_focus: asArray(source.next_week_focus || source.nextWeekFocus || source.focus).slice(0, 6).length
      ? asArray(source.next_week_focus || source.nextWeekFocus || source.focus).slice(0, 6)
      : fallback.next_week_focus,
  };
}

function parseWeeklyArchitectureResponse({ responseFile, context }) {
  const response = readText(responseFile, "");
  if (!response.trim()) {
    return normalizeWeeklyArchitectureInsight(null, context);
  }

  try {
    return normalizeWeeklyArchitectureInsight(parseJsonObject(response), context);
  } catch (_) {
    return normalizeWeeklyArchitectureInsight(null, context);
  }
}

function formatRecommendedIssues(items) {
  if (!items.length) {
    return "- None";
  }
  return items
    .slice(0, 6)
    .map((item) => `- **${escapeTableCell(item.title)}**: ${item.reason} First step: ${item.first_step}`)
    .join("\n");
}

function formatWeeklyArchitectureInsight(insight) {
  return [
    "### Read of the Week",
    "",
    `> ${insight.read_of_week}`,
    "",
    "### CI Trends",
    markdownList(insight.ci_trends),
    "",
    "### Layer Boundary Health",
    markdownList(insight.layer_boundary_health),
    "",
    "### CI Signal Quality",
    markdownList(insight.ci_signal_quality),
    "",
    "### Low-layer Safety And Policy",
    markdownList(insight.low_layer_safety_policy),
    "",
    "### Project Bottlenecks",
    markdownList(insight.project_bottlenecks),
    "",
    "### Architecture Risks",
    markdownList(insight.architecture_risks),
    "",
    "### Technical Debt",
    markdownList(insight.technical_debt),
    "",
    "### Test Gaps",
    markdownList(insight.test_gaps),
    "",
    "### Recommended Issues",
    formatRecommendedIssues(insight.recommended_issues),
    "",
    "### Next Week Focus",
    markdownChecklist(insight.next_week_focus),
  ].join("\n");
}

function renderWeeklyArchitectureBody({ context, weeklyInsight, model }) {
  const insight = weeklyInsight || fallbackWeeklyArchitectureInsight(context);
  const activity = context.pr_activity;
  const metricTable = markdownTable(
    ["Metric", "Count"],
    [
      ["PRs opened", activity.opened_total],
      ["PRs merged", activity.merged_total],
      ["PRs closed without merge", activity.closed_not_merged_total],
      ["Commits", context.git.commits],
      ["CI failure issues updated", context.ci_failures.length],
      ["Weekday lens artifacts", context.weekly_lens_artifacts?.length || 0],
      ["Changed files", context.change_signals?.changed_file_count || 0],
    ],
  );
  const prDetails = [
    "#### Opened PRs",
    formatPrItems(activity.opened),
    "",
    "#### Merged PRs",
    formatPrItems(activity.merged),
    "",
    "#### Closed Without Merge",
    formatPrItems(activity.closed_not_merged),
  ].join("\n");
  const hotspotDetails = [
    "#### Top Authors",
    formatNamedCounts(context.git.top_authors, "author", "commits"),
    "",
    "#### Hot Directories",
    formatNamedCounts(context.git.hot_directories, "directory", "files changed"),
  ].join("\n");

  return [
    `### AI Doctor Weekly Architecture Review (${context.window.week_jst} JST Window)`,
    "",
    metricTable,
    "",
    formatWeeklyArchitectureInsight(insight),
    "",
    "---",
    "",
    "### Evidence Snapshot",
    "",
    detailsBlock("PR activity", prDetails),
    "",
    detailsBlock("Authors and hot directories", hotspotDetails),
    "",
    detailsBlock("Weekday lens artifacts", formatWeeklyLensArtifacts(context.weekly_lens_artifacts)),
    "",
    detailsBlock("Deterministic change signals", formatChangeSignalSummary(context.change_signals)),
    "",
    detailsBlock("CI failure issues", formatCiFailureItems(context.ci_failures)),
    "",
    `<sub>Generated by AI Doctor with ${model || "unknown model"}. Window: ${context.window.since_utc} to ${context.window.until_utc}.</sub>`,
    "",
  ].join("\n");
}

module.exports = {
  attachChangeSignals,
  attachGitStats,
  buildDigestPrompt,
  buildFailurePrompt,
  buildWeeklyLensPrompt,
  buildWeeklyArchitecturePrompt,
  collectDigestContext,
  collectWeeklyArchitectureContext,
  createWeeklyLensGate,
  evaluateDigestModelGate,
  evaluateWeeklyArchitectureGate,
  parseDiagnosisResponse,
  parseDigestResponse,
  parseWeeklyLensResponse,
  parseWeeklyArchitectureResponse,
  renderDigestBody,
  renderWeeklyArchitectureBody,
  writeDiagnosisArtifacts,
};
