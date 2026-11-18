<script lang="ts">
  import SettingsView from "$lib/components/SettingsView.svelte";
  import {
    applySettingsToDom,
    loadHistoryEnabled,
    loadUiSettings,
    persistHistoryEnabled,
    persistUiSettings,
    type UiSettings,
  } from "$lib/settings.svelte";

  let settings = $state<UiSettings>(loadUiSettings());
  let historyEnabled = $state(loadHistoryEnabled());

  function onchange(next: UiSettings) {
    settings = next;
    persistUiSettings(next);
    applySettingsToDom(next);
  }

  $effect(() => {
    persistHistoryEnabled(historyEnabled);
  });
</script>

<SettingsView bind:settings bind:historyEnabled {onchange} />
