<script lang="ts">
  import {
    Button,
    SpeedDial,
    SpeedDialButton,
    Modal,
    Label,
    Input,
  } from "flowbite-svelte";
  import Pipelines from "./lib/Pipelines.svelte";
  import Configs from "./lib/Configs.svelte";
  import { Icon } from "flowbite-svelte-icons";
  import { invoke } from "@tauri-apps/api/core";

  let tabValue = "pipelines";
  $: activeTab = tabValue;

  let defaultModal = false;
  $: modalState = defaultModal;

  // Form data for the modal
  let serverName = "";
  let serverUrl = "";
  let apiToken = "";
  let serverType = "gitlab";
  
  // Refresh trigger for CI servers list
  let configRefreshTrigger = 0;

  function setPipelinesActive() {
    tabValue = "pipelines";
  }

  function setConfigActive() {
    tabValue = "ci_servers";
  }

  function showModal() {
    modalState = true;
  }

  function clearForm() {
    serverName = "";
    serverUrl = "";
    apiToken = "";
    serverType = "gitlab";
  }

  async function saveConfig() {
    if (!serverName.trim() || !serverUrl.trim() || !apiToken.trim()) {
      alert("Please fill in all required fields");
      return;
    }

    try {
      await invoke("store_ci_server_data", {
        name: serverName.trim(),
        serverType: serverType,
        urlString: serverUrl.trim(),
        apiKey: apiToken.trim(),
      });
      
      modalState = false;
      clearForm();
      configRefreshTrigger += 1; // Trigger refresh of CI servers list
      alert("CI server added successfully!");
    } catch (error) {
      console.error("Failed to save CI server:", error);
      alert("Failed to save CI server: " + error);
    }
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
      <Configs refreshTrigger={configRefreshTrigger} />
    {/if}
    <Modal title="Add new CI server" bind:open={modalState} autoclose>
      <div class="mb-6">
        <Label for="server-name" class="block mb-2">Server Name *</Label>
        <Input 
          type="text" 
          id="server-name" 
          placeholder="e.g., My GitLab Server" 
          bind:value={serverName}
          required
        />
      </div>
      <div class="mb-6">
        <Label for="server-url" class="block mb-2">Server URL *</Label>
        <Input 
          type="url" 
          id="server-url" 
          placeholder="https://gitlab.example.com" 
          bind:value={serverUrl}
          required
        />
      </div>
      <div class="mb-6">
        <Label for="api-token" class="block mb-2">API Token *</Label>
        <Input 
          type="password" 
          id="api-token" 
          placeholder="glpat-xxxxxxxxxxxxxxxxxxxx" 
          bind:value={apiToken}
          required
        />
      </div>
      <div class="mb-6">
        <Label for="server-type" class="block mb-2">Server Type</Label>
        <select 
          id="server-type" 
          bind:value={serverType}
          class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5"
        >
          <option value="gitlab">GitLab</option>
          <option value="github">GitHub</option>
          <option value="jenkins">Jenkins</option>
        </select>
      </div>

      <svelte:fragment slot="footer">
        <Button on:click={saveConfig}>Add Server</Button>
        <Button color="alternative" on:click={() => modalState = false}>Cancel</Button>
      </svelte:fragment>
    </Modal>
  </div>
</main>
