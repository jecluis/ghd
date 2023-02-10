<script lang="ts">
  import Greet from "../lib/Greet.svelte";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  type PREntry = {
    id: number;
    title: string;
    age_str: string;
  };

  type PRList = {
    entries: PREntry[];
  };

  let n = 0;
  let prs: PRList = { entries: [] };

  onMount(async () => {

    await listen("iteration", (ev) => {
        n = <number>ev.payload;
    });

    await listen("pull_requests_update", (ev) => {
        prs = <PRList>ev.payload;
    });
  });


</script>

<h1>Welcome to SvelteKit</h1>
<p>
  Visit <a href="https://kit.svelte.dev">kit.svelte.dev</a> to read the documentation
</p>
<Greet />

<p>n: {n}</p>

<p>Pull Requests:</p>
<ul>
    {#each prs.entries as {id, title, age_str}}
    <li>{id}: {title} ({age_str} ago)</li>
    {/each}
</ul>