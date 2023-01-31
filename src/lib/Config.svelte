<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { onMount } from "svelte";
  import { Button, Form, FormGroup, Input, Label } from "sveltestrap";

  let loading = false;
  let apiToken = "";
  let isWorking = false;
  let hasSetToken = false;
  let isSuccess = false;

  onMount(async () => {
    loading = true;
    invoke("get_api_token")
      .then((res) => {
        apiToken = <string>res;
      })
      .finally(() => {
        loading = false;
      });
  });

  async function setToken() {
    console.log("");
    isWorking = true;
    invoke("set_api_token", { token: apiToken })
      .then((res) => {
        hasSetToken = true;
        isSuccess = <boolean>res;
      })
      .finally(() => {
        isWorking = false;
      });
  }
</script>

{#if loading}
  <p>Loading...</p>
{:else}
  <div>
    <Form>
      <FormGroup>
        <Label for="api-token-input">GitHub Token</Label>
        <Input
          id="api-token-input"
          name="apiTokenInput"
          placeholder="GitHub API Token"
          bind:value={apiToken}
        />
        <Button color="primary" on:click={setToken} disabled={isWorking}
          >Submit</Button
        >
      </FormGroup>
    </Form>
  </div>
{/if}
