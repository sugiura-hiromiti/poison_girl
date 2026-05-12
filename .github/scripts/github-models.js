"use strict";

const CATALOG_URL = "https://models.github.ai/catalog/models";
const API_VERSION = "2026-03-10";

function clean(value) {
  return String(value ?? "").trim();
}

function safeReason(value, fallback) {
  const reason = clean(value).replace(/\s+/g, " ");
  return reason || fallback;
}

async function checkModelAvailability({ token, model, fetchImpl = globalThis.fetch }) {
  const modelId = clean(model);
  if (!modelId) {
    return {
      available: false,
      model: "",
      reason: "model id is empty",
    };
  }

  if (!token) {
    return {
      available: false,
      model: modelId,
      reason: "GitHub token is unavailable for model catalog check",
    };
  }

  if (typeof fetchImpl !== "function") {
    return {
      available: false,
      model: modelId,
      reason: "fetch is unavailable for model catalog check",
    };
  }

  try {
    const response = await fetchImpl(CATALOG_URL, {
      headers: {
        Accept: "application/vnd.github+json",
        Authorization: `Bearer ${token}`,
        "X-GitHub-Api-Version": API_VERSION,
      },
    });

    if (!response.ok) {
      return {
        available: false,
        model: modelId,
        reason: `GitHub Models catalog returned HTTP ${response.status}`,
      };
    }

    const catalog = await response.json();
    if (!Array.isArray(catalog)) {
      return {
        available: false,
        model: modelId,
        reason: "GitHub Models catalog returned an unexpected response",
      };
    }

    const entry = catalog.find((item) => clean(item.id) === modelId);
    if (!entry) {
      return {
        available: false,
        model: modelId,
        reason: `model ${modelId} is not listed in the GitHub Models catalog`,
      };
    }

    return {
      available: true,
      model: modelId,
      reason: "available",
      catalog_entry: {
        id: entry.id,
        name: entry.name || "",
        publisher: entry.publisher || "",
        registry: entry.registry || "",
      },
    };
  } catch (error) {
    return {
      available: false,
      model: modelId,
      reason: safeReason(error && error.message, "GitHub Models catalog check failed"),
    };
  }
}

module.exports = {
  API_VERSION,
  CATALOG_URL,
  checkModelAvailability,
};
