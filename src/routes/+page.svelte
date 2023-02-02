<script lang="ts">
  import Greet from "../lib/Greet.svelte";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  let n = 0;

  onMount(async () => {

    await listen("iteration", (ev) => {
        n = <number>ev.payload;
    });
  });


</script>

<h1>Welcome to SvelteKit</h1>
<p>
  Visit <a href="https://kit.svelte.dev">kit.svelte.dev</a> to read the documentation
</p>
<Greet />

<p>n: {n}</p>