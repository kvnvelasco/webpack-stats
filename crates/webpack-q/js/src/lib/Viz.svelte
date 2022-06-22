<!--
  - Copyright [2022] [Kevin Velasco]
  -
  - Licensed under the Apache License, Version 2.0 (the "License");
  - you may not use this file except in compliance with the License.
  - You may obtain a copy of the License at
  -
  - http://www.apache.org/licenses/LICENSE-2.0
  -
  - Unless required by applicable law or agreed to in writing, software
  - distributed under the License is distributed on an "AS IS" BASIS,
  - WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
  - See the License for the specific language governing permissions and
  - limitations under the License.
  -->

<script lang="ts">
    import {
        forceCenter,
        forceCollide,
        forceLink,
        forceManyBody,
        forceSimulation,
        json,
        rollup,
        scaleOrdinal,
        schemeDark2,
        select,
        zoom
    } from 'd3'
    import {draw} from './draw.js';
    import {createEventDispatcher, onMount} from "svelte";

    const dispatch = createEventDispatcher();

    let width = window.innerWidth;
    let height = window.innerHeight;
    let nodeFromCache = {};
    export let nodes = [], links = [], selected = {}, search;

    let canvas, transform, simulation;


    onMount(loadViz);

    async function loadViz() {
        simulation = forceSimulation(nodes)
            .force("link", forceLink(links).id((d) => d.id))

            .force("charge", forceManyBody().strength(-40))
            .force("collide", forceCollide().radius(6))
            .force("center", forceCenter(width / 2, height / 2))
            .force("chunk", chunkForce().strength(0.8))
            .on("tick", () => {
                nodes = [...nodes];
                links = [...links];
            })
            .on("end", () => {
                links.forEach(link => {
                    nodeFromCache[link.source.id] = [...(nodeFromCache[link.source.id] || []), link.target]
                })
            });

        function chunkForce() {
            let nodes = [];
            let strength = 0.1;

            function force(alpha) {
                const centroids = rollup(nodes, centroidAndExtents, d => d.chunk);

                for (let node of nodes) {
                    const centroidAndExtent = centroids.get(node.chunk);
                    const x = centroidAndExtent.x;
                    const y = centroidAndExtent.y;

                    node.vx += (x - node.x) * strength * alpha;
                    node.vy += (y - node.y) * strength * alpha;
                }
            }

            force.initialize = _ => nodes = _;
            force.strength = function (_) {
                strength = _;
                return this
            };
            return force
        }

        function centroidAndExtents(nodes) {
            let x = 0;
            let y = 0;
            let z = 0;
            let maxD = 0;
            for (const d of nodes) {
                x += d.x;
                y += d.y;
                z += 1;
            }

            return {x: x / z, y: y / z};
        }
    }

    $: canvas && select(canvas)
        .call(zoom().on("zoom", ({transform: _}) => {
            transform = _;
        }))
        .attr('width', width)
        .attr('height', height);


    let hoverTimeout;
    let hoverNode, hoverX, hoverY;
    let highlightedNode;

    export let hideEdges = true;

    function nodevisible(transform, n) {
        if (n.x == null || n.y == null) return true;
        let realPosition = [transform ? transform.applyX(n.x) : n.x, transform ? transform.applyY(n.y) : n.y];
        return !(realPosition[0] > width || realPosition[0] < 0 || realPosition[1] > height || realPosition[0] < 0)
    }

    $: visibleNodes = nodes.filter(n => {
        return nodevisible(transform, n);
    })

    $: visibleLinks = links.filter(n => {
        return nodevisible(transform, n.source) || nodevisible(transform, n.target);
    });

    $: canvas && draw(visibleNodes, visibleLinks, selected, transform, canvas, search, width, height, highlightedNode, !hideEdges);

    function hover(e) {
        if (hoverTimeout) clearTimeout(hoverTimeout);
        const x = e.clientX;
        const y = e.clientY;

        const node = getNodeForMouseEvent({clientX: x, clientY: y});
        highlightedNode = node?.id;
        if (hoverTimeout) clearTimeout(hoverTimeout);
        hoverTimeout = setTimeout(() => {
            const node = getNodeForMouseEvent({clientX: x, clientY: y});
            if (node) {
                hoverNode = node;
                hoverX = e.clientX;
                hoverY = e.clientY;
            }
        }, 200)
    }

    $: dispatch("hover", {node: hoverNode, x: hoverX, y: hoverY});

    function getNodeForMouseEvent(e) {
        if (simulation) {
            let x, y;
            if (transform) {
                x = transform.invertX(e.clientX);
                y = transform.invertY(e.clientY);
            } else {
                x = e.clientX;
                y = e.clientY;
            }
            return simulation.find(x, y, 6)
        }

        return null
    }

    function selectNode(e) {
        let node = getNodeForMouseEvent(e);
        if (node) {
            selected[node.id] = node;
        }
    }

    function deselectNode(e) {
        const node = getNodeForMouseEvent(e);
        if (node) {
            e.preventDefault();
            delete selected[node.id];
            selected = {...selected};
        }
    }

</script>

<svelte:window
        bind:innerWidth={width}
        bind:innerHeight={height}
/>
<canvas bind:this={canvas}
        on:contextmenu={deselectNode}
        on:click={selectNode} on:mousemove={hover}></canvas>

