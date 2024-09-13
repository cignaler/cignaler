<script lang="ts">
    import Pipeline from "./Pipeline.svelte";
    import {invoke} from "@tauri-apps/api/tauri";
    import { Toast } from 'flowbite-svelte';


    interface PipelineInterface {
        ref: string
        web_url: string
        id: number
        status: string
        created_at: string,
        updated_at: string | null,
        finished_at: string | null,
    }
    let pipes: PipelineInterface[] = []

    async function get_pipelines() {
        return await invoke('get_pipelines')
    }

    let clear
    let ms = 10000
    $: {
        clearInterval(clear)
        clear = setInterval(get_pipelines_sync, ms)
    }

    function get_pipelines_sync() {
        invoke('get_pipelines')
            .then((data) => pipes = data as PipelineInterface[])
            .catch((err) => console.log(err))
    }
    invoke('get_pipelines')
        .then((data) => pipes = data as PipelineInterface[])
        .catch((err) => console.log(err))

</script>

<div class="flex justify-center">
    <div class="container mx-auto flex flex-col px-4 drop-shadow-lg">
        {#await get_pipelines()}
            <div class="flex justify-center">
                <p>Loading...</p>
            </div>
        {:then data}
            {#each pipes as item}
                <div class="flex items-center">
                    <Pipeline name={item.ref} state={item.status} lastExecuted={item.created_at} webUrl={item.web_url}/>
                </div>
            {/each}
        {:catch error}
            <Toast color="red" class="flex items-center">
                <svelte:fragment slot="icon">
                    <span class="sr-only">Warning icon</span>
                </svelte:fragment>
                {error}
            </Toast>
        {/await}
    </div>
</div>
