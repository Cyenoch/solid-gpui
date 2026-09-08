import { highlights } from "virtual:component-catalog";
import { snippetKey } from "./snippets";

const results = new Map(highlights.map((result) => [snippetKey(result.code, result.language), result]));

export function highlight(source: string, language = "tsx") {
  const result = results.get(snippetKey(source, language));
  if (!result) throw new Error(`Missing build-time Shiki result for ${language} code`);
  return result;
}
