<script lang="ts">
  import { getContext } from "svelte";
  import HomeView from "$lib/components/HomeView.svelte";
  import type { NavId } from "$lib/components/Sidebar.svelte";

  const onNavigate = getContext<(id: NavId) => void>("fp-navigate");
</script>

<HomeView onNavigate={(id) => onNavigate?.(id)} />
