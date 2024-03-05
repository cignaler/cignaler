<script lang="ts">
    import Pipeline from "./Pipeline.svelte";
    import {invoke} from "@tauri-apps/api/tauri";

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
    invoke('get_pipelines')
        .then((data) => pipes = data as PipelineInterface[])
        .catch((err) => console.log(err))

</script>

<div class="flex justify-center">
    <div class="container mx-auto flex flex-col px-4 drop-shadow-lg">
        {#each pipes as item}
            <div class="flex items-center">
                <Pipeline name={item.ref} state={item.status} lastExecuted={item.created_at} webUrl={item.web_url}/>
            </div>
        {/each}
    </div>
</div>
```
