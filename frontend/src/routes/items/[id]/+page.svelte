<script lang="ts">
  import { page } from "$app/state";

  let id = page.params.id;
  let item = fetch(`/api/items/${id}`).then((res) => res.json());
</script>

<div>
  <h1>Item {id}</h1>
  {#await item}
    Loading ...
  {:then item}
    {item.name}
  {/await}
</div>
