import type { TranslateMetadataRequest } from "@/types";

export const AI_CONTEXT_LIMITS = [50000, 100000, 200000] as const;

export function normalizeContextLimit(value: number): number {
  return AI_CONTEXT_LIMITS.includes(value as (typeof AI_CONTEXT_LIMITS)[number]) ? value : 50000;
}

export function estimateTranslationTokens(request: TranslateMetadataRequest): number {
  const text = JSON.stringify(request);
  let quarterTokens = 0;
  for (const character of text) {
    if (/^[a-zA-Z0-9\s]$/.test(character)) quarterTokens += 1;
    else if (character.charCodeAt(0) <= 0x7f) quarterTokens += 2;
    else quarterTokens += 4;
  }
  return Math.ceil(quarterTokens / 4) + 32;
}

export function groupTranslationRequests<T>(
  items: T[],
  toRequest: (item: T) => TranslateMetadataRequest,
  contextLimit: number,
): T[][] {
  // Accuracy first: only one eighth of the context is used for input. The
  // remainder covers complete output and hidden reasoning tokens while also
  // reducing cross-entry contamination in large batches.
  const inputBudget = Math.floor(normalizeContextLimit(contextLimit) / 8);
  const promptOverhead = 512;
  const batches: T[][] = [];
  let current: T[] = [];
  let currentTokens = promptOverhead;

  for (const item of items) {
    const tokens = estimateTranslationTokens(toRequest(item));
    if (current.length > 0 && currentTokens + tokens > inputBudget) {
      batches.push(current);
      current = [];
      currentTokens = promptOverhead;
    }
    current.push(item);
    currentTokens += tokens;
  }
  if (current.length > 0) batches.push(current);
  return batches;
}
