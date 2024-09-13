<script lang="ts">
    import {Modal, Tabs, TabItem, Button} from 'flowbite-svelte';

    export let defaultModal = false;
</script>

    <Tabs tabStyle="underline">
        <TabItem open title="Profile">
            <p class="text-sm text-gray-500 dark:text-gray-400">
                <b>Profile:</b>
                Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
            </p>
        </TabItem>

        <TabItem open title="Lalala">
            <p class="text-sm text-gray-500 dark:text-gray-400">
                <b>Profile:</b>
                Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
            </p>
        </TabItem>
    </Tabs>

