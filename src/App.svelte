<script lang="ts">
  import {Button, SpeedDial, SpeedDialButton} from 'flowbite-svelte';
  import Pipelines from './lib/Pipelines.svelte'
  import Config from "./lib/Config.svelte";
  import {Icon} from "flowbite-svelte-icons";

  let tabValue = 'pipelines';
  $: activeTab = tabValue;

  function setPipelinesActive() {
    tabValue = 'pipelines';
  }

  function setConfigActive() {
    tabValue = 'ci_servers';
  }
</script>

<main>
  <div class="flex justify-between mt-4 mx-6 mb-4">
    <div class="flex basis-1/4">
      <Button outline={activeTab !== 'pipelines'} on:click={setPipelinesActive} class="mr-4">Pipelines</Button>
      <Button outline={activeTab !== 'ci_servers'} on:click={setConfigActive} class="w-28">CI Servers</Button>
    </div>
    <div class="flex">
      <SpeedDial defaultClass="absolute right-6 top-2" placement="left" tooltip="left">
        <SpeedDialButton name="Add new pipeline watcher" tooltip="left">
          <Icon name="code-pull-request-solid" class="w-5 h-5" />
        </SpeedDialButton>
        <SpeedDialButton name="Add new CI server" tooltip="left" on:click>
          <Icon name="user-settings-solid" class="w-5 h-5" />
        </SpeedDialButton>
      </SpeedDial>
    </div>
  </div>
  <div class="main-content">
    {#if activeTab === 'pipelines'}
      <Pipelines />
    {:else if activeTab === 'ci_servers'}
      <Config />
    {/if}
  </div>
</main>
