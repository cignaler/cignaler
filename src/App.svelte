<script lang="ts">
  import { Button, SpeedDial, SpeedDialButton, Modal } from "flowbite-svelte";
  import Pipelines from "./lib/Pipelines.svelte";
  import Config from "./lib/Config.svelte";
  import { Icon } from "flowbite-svelte-icons";

  let tabValue = "pipelines";
  $: activeTab = tabValue;

  let defaultModal = false;
  $: modalState = defaultModal;

  function setPipelinesActive() {
    tabValue = "pipelines";
  }

  function setConfigActive() {
    tabValue = "ci_servers";
  }

  function showModal() {
    modalState = true;
  }
</script>

<main>
  <div class="flex justify-between mt-4 mx-6 mb-4">
    <div class="flex basis-1/4">
      <Button
        outline={activeTab !== "pipelines"}
        on:click={setPipelinesActive}
        class="mr-4">Pipelines</Button
      >
      <Button
        outline={activeTab !== "ci_servers"}
        on:click={setConfigActive}
        class="w-28">CI Servers</Button
      >
    </div>
    <div class="flex">
      <SpeedDial
        defaultClass="absolute right-6 top-2"
        placement="left"
        tooltip="left"
      >
        <SpeedDialButton name="Add new pipeline watcher" tooltip="left">
          <Icon name="code-pull-request-solid" class="w-5 h-5" />
        </SpeedDialButton>
        <SpeedDialButton
          name="Add new CI server"
          tooltip="left"
          on:click={showModal}
        >
          <Icon name="user-settings-solid" class="w-5 h-5" />
        </SpeedDialButton>
      </SpeedDial>
    </div>
  </div>
  <div class="main-content">
    {#if activeTab === "pipelines"}
      <Pipelines />
    {:else if activeTab === "ci_servers"}
      <Config />
    {/if}
    <Modal title="Terms of Service" bind:open={modalState} autoclose>
      <p class="text-base leading-relaxed text-gray-500 dark:text-gray-400">
        With less than a month to go before the European Union enacts new
        consumer privacy laws for its citizens, companies around the world are
        updating their terms of service agreements to comply.
      </p>
      <p class="text-base leading-relaxed text-gray-500 dark:text-gray-400">
        The European Union’s General Data Protection Regulation (G.D.P.R.) goes
        into effect on May 25 and is meant to ensure a common set of data rights
        in the European Union. It requires organizations to notify users as soon
        as possible of high-risk data breaches that could personally affect
        them.
      </p>
      <svelte:fragment slot="footer">
        <Button on:click={() => alert('Handle "success"')}>Add</Button>
        <Button color="alternative">Abort</Button>
      </svelte:fragment>
    </Modal>
  </div>
</main>
