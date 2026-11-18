<script lang="ts">
  import ConversionLanding from "$lib/seo/ConversionLanding.svelte";
  import {
    converterHref,
    pairDescription,
    pairTitle,
  } from "$lib/seo/pairs";

  let { data } = $props();

  const title = $derived(pairTitle(data.pair));
  const description = $derived(pairDescription(data.pair));
  const canonical = $derived(`/${data.pair.slug}`);
  const convertUrl = $derived(converterHref(data.pair));
</script>

<svelte:head>
  <title>{title} | FilePrism</title>
  <meta name="description" content={description} />
  <meta name="robots" content="index,follow" />
  <link rel="canonical" href={canonical} />
  <meta property="og:title" content={`${title} | FilePrism`} />
  <meta property="og:description" content={description} />
  <meta property="og:type" content="website" />
  <meta property="og:url" content={canonical} />
  <meta name="twitter:card" content="summary" />
  <meta name="twitter:title" content={`${title} | FilePrism`} />
  <meta name="twitter:description" content={description} />
  <link rel="alternate" href={convertUrl} />
</svelte:head>

<ConversionLanding pair={data.pair} />
