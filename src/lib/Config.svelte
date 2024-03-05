<script lang="ts">
    import {Modal, Tabs, TabItem, Button} from 'flowbite-svelte';

    export let defaultModal = false;
</script>

<Modal title="Terms of Service" bind:open={defaultModal} autoclose outsideclose>
    <Tabs style="underline">
        <TabItem open title="Profile">
            <p class="text-sm text-gray-500 dark:text-gray-400">
                <b>Profile:</b>
                Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
            </p>
        </TabItem>
    </Tabs>
    <svelte:fragment slot="footer">
        <Button on:click={() => alert('Handle "success"')}>Save</Button>
        <Button color="alternative">Cancel</Button>
    </svelte:fragment>
</Modal>
