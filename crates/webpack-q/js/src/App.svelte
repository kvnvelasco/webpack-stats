<script>
    import Viz from './lib/Viz.svelte';
    import {onMount} from "svelte";
    import {json, scaleOrdinal, schemeDark2} from "d3";

    let nodes = [], links = [], x = 0, y = 0, node, selected;
    let key;
    let search;
    let hideEdges;

    async function fetchData() {
        const data = await json("./data.json");
        let color = scaleOrdinal(nodes.map(n => n.chunk), schemeDark2)

        links = data.edges;
        nodes = data.nodes;
        nodes.forEach(node => node.color = color(node.chunk));
        if (links.length > 1000) {
            hideEdges = true
        }
        key = Math.random();
    }

    function hover(e) {
        const detail = e.detail;
        select(detail.x, detail.y, detail.node);
    }

    function select(_x, _y, _node) {
        x = _x + 10;
        y = _y + 10;
        node = _node;
    }

    function selectNode(node) {
        // find the coordinates;
        selected[node.id] = node;
    }

    function prune() {
        let nodeCache = {}
        links = links.filter(link => selected[link.source.id] || selected[link.target.id])

        links.forEach(link => {
            nodeCache[link.source.id] = link.source;
            nodeCache[link.target.id] = link.target;
        })

        nodes = Object.values(nodeCache)


        key = Math.random();
    }

    $: nodeEdges = node ? links.filter(link => link.source.id === node.id || link.target.id === node.id) : []

    $: outgoingEdges = nodeEdges.filter(edge => edge.source === node);
    $: incomingedges = nodeEdges.filter(edge => edge.target === node);

</script>

{#if node}
    <div id="hover">
        <label>{node.label} [{node.chunk}]
            {#if selected[node.id]} (selected){/if}
        </label>
        {#if outgoingEdges.length > 0}
            <p><strong> Outgoing Edges: </strong></p>
            <ul>
                {#each outgoingEdges as edge}
                    <li on:click={() => selectNode(edge.target)}>
                        <p> {edge.target.label} [{edge.target.chunk}] </p>
                        {#if edge.importer}
                            <p class="import">imported by: {edge.importer}</p>
                        {/if}
                    </li>
                {/each}
            </ul>
        {/if}
        {#if incomingedges.length > 0}
            <p><strong> Incoming Edges: </strong></p>
            <ul>
                {#each incomingedges as edge}
                    <li on:click={() => selectNode(edge.source)}>
                        <p>{edge.source.label} [{edge.source.chunk}]</p>
                        {#if edge.importer}
                            <p class="import">imported by: {edge.importer}</p>
                        {/if}
                    </li>
                {/each}
            </ul>
        {/if}
    </div>
{/if}

<div id="search">
    <input bind:value={search} placeholder="Search for nodes"/>
</div>


<div id="tools">
    <div>
        <label for="hide-edges">Hide Edges</label> <input id="hide-edges" bind:checked={hideEdges} type="checkbox"/>
    </div>
    <div>
        tools:
        <button on:click={prune}> Prune</button>
        <button on:click={() => selected = {}}>Clear Selection</button>
        <button on:click={fetchData}> Reset</button>
    </div>
</div>
{#await fetchData()}
{:then _}
    {#key key}
        <Viz bind:hideEdges search={search} bind:nodes={nodes} bind:links={links} on:hover={hover}
             bind:selected={selected}/>
    {/key}
{/await}

<style>
    #hover {
        font-family: sans-serif;
    }
    .import {
        font-size: 12px;
        text-decoration: underline;
        opacity: 0.6;
    }
    #hover {
        position: fixed;
        background: white;
        left: 0;
        top: 0;
        bottom: 0;
        padding: 20px;
        border-radius: 10px;
        box-shadow: 1px 2px 3px rgba(0, 0, 0, 0.1);
        width: 300px;
        box-sizing: border-box;
        overflow: scroll;
    }

    #hover label {
        font-weight: bold;
        text-decoration: underline;
    }

    ul {
        padding-left: 10px;
    }
    #hover li {
        cursor: pointer;
        margin-bottom: 5px;
        margin-left: 2px;
    }

    #search {
        position: fixed;
        top: 20px;
        right: 20px;
    }

    #tools {
        position: fixed;
        bottom: 20px;
        right: 20px;
    }

    :global(body) {
        overflow: hidden;
        font-family: sans-serif;
        font-size: 14px;
    }
</style>