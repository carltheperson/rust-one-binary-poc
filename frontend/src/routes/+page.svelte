<script lang="ts">
  import RustLogo from "$lib/rust-logo.svelte";
  import SvelteLogo from "$lib/svelte-logo.svelte";

  let counter = $state(1);

  let items = fetch("/api/items").then((res) => res.json());
</script>

<div class="simple">
  <RustLogo />
  <SvelteLogo />
  <div class="title">Frontend POC</div>
  <div
    class="sub-text"
    style="display: flex; align-items: center; gap: 5px; height: 40px"
  >
    Click <button onclick={() => counter++}>here</button> to update the counter:
    {counter}
  </div>
  <div class="sub-text">Items fetched from backend:</div>
  {#await items}
    Loading ...
  {:then items}
    <div class="listing">
      {#each items as item}
        <a href={`/items/${item.id}`}>
          --> {item.name}
        </a>
      {/each}
    </div>
  {/await}
</div>

<style>
  .simple {
    padding: 10px;
    font-family: Georgia, "Times New Roman", Times, serif;
    width: 500px;
    height: 500px;
    border: 7px solid black;
  }

  .sub-text {
    font-weight: 300;
    font-size: 20px;
  }

  .title {
    font-weight: 700;
    font-size: 65px;
  }

  .listing {
    margin-top: 5px;
    font-size: 20px;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .listing a {
    font-size: 20px;
    text-decoration: none;
    margin-left: 10px;
    color: black;
  }
</style>
