import { error } from "@sveltejs/kit";
import { CONVERSION_PAIRS, getPair } from "$lib/seo/pairs";
import type { EntryGenerator, PageLoad } from "./$types";

export const prerender = true;

export const entries: EntryGenerator = () =>
  CONVERSION_PAIRS.map((pair) => ({ pair: pair.slug }));

export const load: PageLoad = ({ params }) => {
  const pair = getPair(params.pair);
  if (!pair) {
    error(404, `Unknown conversion pair: ${params.pair}`);
  }
  return { pair };
};
