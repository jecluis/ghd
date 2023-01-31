<script lang="ts">

    import { invoke } from "@tauri-apps/api/tauri";
  import { onMount } from "svelte";

    let loading = false;
    let apiToken = "";
    let isWorking = false;
    let hasSetToken = false;
    let isSuccess = false;

    onMount(async () => {
        loading = true;
        invoke("get_api_token")
        .then(
            (res) => {
                apiToken = <string>res;
            }
        )
        .finally(() => {
            loading = false;
        });
    });

    async function setToken() {
        console.log("")
        invoke("set_api_token", { token: apiToken })
        .then(
            (res) => {
                hasSetToken = true;
                isSuccess = <boolean>res;
            }
        )
        .finally(() => {
            isWorking = false;
        });
    }

</script>

{#if !loading}
<div>
    <input
        id="api-token-input" placeholder="GitHub API Token"
        bind:value="{apiToken}"
    />
    <button on:click="{setToken}">Submit</button>
    {#if isWorking}
    <p>Setting value...</p>
    {:else if hasSetToken}
        {#if isSuccess}
            <p>Success setting token!</p>
        {:else}
            <p>Error setting token :(</p>
        {/if}
    {/if}
    <p>GitHub API Token: {apiToken}</p>

</div>
{:else}
<div>
    <p>loading...</p>
</div>
{/if}