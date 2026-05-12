"use strict";

const fs = require("fs");

const SEVERITIES = new Set(["critical", "moderate", "minor", "none"]);
const CONFIDENCE = new Set(["high", "medium", "low"]);
const SEVERITY_RANK = {
  critical: 0,
  moderate: 1,
  minor: 2,
  none: 3,
  unknown: 4,
};
const MAX_LOG_TAIL_CHARS = 16000;
const MAX_SIGNAL_LINES = 80;

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

function fallbackDiagnosis(message, ciRunUrl) {
  return {
    severity: "moderate",
    headline: message,
    likely_root_cause:
      "The CI run failed, but AI Doctor could not derive a structured diagnosis from the model response.",
    confidence: "low",
    evidence: [ciRunUrl ? `CI run: ${ciRunUrl}` : "CI run URL was unavailable."],
    impact: "The PR remains blocked until the failing CI step is inspected.",
    next_actions: [
      ciRunUrl
        ? `Open the CI run and inspect the first failing step: ${ciRunUrl}`
        : "Open the latest CI run and inspect the first failing step.",
      "Re-run AI Doctor after logs are available if the failure is still unclear.",
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

function parseDiagnosisResponse({ responseFile, ciRunUrl }) {
  const response = readText(responseFile, "");
  if (!response.trim()) {
    return normalizeDiagnosis(
      fallbackDiagnosis("CI failed, but AI inference did not return a response.", ciRunUrl),
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

async function searchIssues(github, query) {
  const res = await github.rest.search.issuesAndPullRequests({
    q: query,
    per_page: 100,
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

function buildDigestPrompt(context) {
  return [
    "# Daily AI Doctor context",
    "",
    "Analyze this activity and return semantic engineering insight.",
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

  return {
    read_of_day: failures
      ? `${failures} CI failure issue(s) changed in the window. Start with repeated severity labels and the latest doctor notes.`
      : "No CI failure issue was updated in this window. There is no strong failure pattern to infer from the available data.",
    signals: [
      `PR flow: ${opened} opened, ${merged} merged, ${context.pr_activity.closed_not_merged_total} closed without merge.`,
      `Git activity: ${context.git.commits} commit(s).`,
    ],
    risks: failures
      ? ["CI failures are present; check whether the same severity or touched area repeats."]
      : ["No clear CI risk is visible from this window alone."],
    recommended_actions: failures
      ? ["Assign owners to the CI failure issues above."]
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
    detailsBlock("CI failure issues", formatCiFailureItems(context.ci_failures)),
    "",
    `<sub>Generated by AI Doctor with ${model || "unknown model"}. Window: ${context.window.since_utc} to ${context.window.until_utc}.</sub>`,
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
  return weeklyContext;
}

function buildWeeklyArchitecturePrompt(context) {
  return [
    "# Weekly AI Architecture Doctor context",
    "",
    "Analyze this week of repository activity as an architectural and CI trend review.",
    "Return strict JSON only with this shape:",
    JSON.stringify(
      {
        read_of_week: "one concise architectural read of the week",
        ci_trends: ["recurring CI failure or flake-check patterns"],
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
    "- Use the CI failure notes, hot directories, and PR flow as evidence.",
    "- Pay special attention to flake checks, loader/kernel/no_std/macro areas, panic/unwrap policy, and dev tooling complexity.",
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
  return {
    read_of_week: failures
      ? `${failures} CI failure issue(s) changed this week. Review repeated severity labels and hot directories before cutting follow-up work.`
      : "No updated CI failure issue is visible in this weekly window. Architectural signal is limited to PR and git activity.",
    ci_trends: failures
      ? ["CI failure issues were updated this week; inspect whether the same check or area repeats."]
      : ["No recurring CI failure trend is visible from the collected issues."],
    architecture_risks: ["No strong architecture risk can be inferred without more failure or PR detail."],
    technical_debt: ["No clear technical debt trend is visible from this weekly context alone."],
    test_gaps: ["Compare hot directories with existing flake checks before adding new tests."],
    recommended_issues: [
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
    detailsBlock("CI failure issues", formatCiFailureItems(context.ci_failures)),
    "",
    `<sub>Generated by AI Doctor with ${model || "unknown model"}. Window: ${context.window.since_utc} to ${context.window.until_utc}.</sub>`,
    "",
  ].join("\n");
}

module.exports = {
  attachGitStats,
  buildDigestPrompt,
  buildFailurePrompt,
  buildWeeklyArchitecturePrompt,
  collectDigestContext,
  collectWeeklyArchitectureContext,
  parseDiagnosisResponse,
  parseDigestResponse,
  parseWeeklyArchitectureResponse,
  renderDigestBody,
  renderWeeklyArchitectureBody,
  writeDiagnosisArtifacts,
};
