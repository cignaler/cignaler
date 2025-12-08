<script lang="ts">
  import { invoke } from "@tauri-apps/api/core"

  let name = $state("");
  let greetMsg = $state("")

  async function greet(){
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    greetMsg = await invoke("greet", { name })
  }
</script>

<div>
  <form class="row" onsubmit={(e) => { e.preventDefault(); greet(); }}>
    <input id="greet-input" placeholder="Enter a name..." bind:value={name} />
    <button type="submit">Greet</button>
  </form>
  <p>{greetMsg}</p>
</div>