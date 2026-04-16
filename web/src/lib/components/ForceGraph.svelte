<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import * as d3 from 'd3';
	import type { GraphNode, GraphLink } from '$lib/types';

	let { nodes, links }: { nodes: GraphNode[]; links: GraphLink[] } = $props();

	let container: HTMLDivElement;
	let svg: d3.Selection<SVGSVGElement, unknown, null, undefined>;
	let simulation: d3.Simulation<GraphNode, GraphLink>;
	let linkGroup: d3.Selection<SVGGElement, unknown, null, undefined>;
	let nodeGroup: d3.Selection<SVGGElement, unknown, null, undefined>;
	let width = 600;
	let height = 400;

	const statusColors: Record<string, string> = {
		idle: '#6b7280',
		working: '#06b6d4',
		collaborating: '#f59e0b',
		finished: '#22c55e',
		error: '#ef4444',
	};

	const statusGlows: Record<string, string> = {
		idle: 'rgba(107, 114, 128, 0.3)',
		working: 'rgba(6, 182, 212, 0.5)',
		collaborating: 'rgba(245, 158, 11, 0.5)',
		finished: 'rgba(34, 197, 94, 0.3)',
		error: 'rgba(239, 68, 68, 0.5)',
	};

	onMount(() => {
		const rect = container.getBoundingClientRect();
		width = rect.width;
		height = rect.height;

		svg = d3
			.select(container)
			.append('svg')
			.attr('width', '100%')
			.attr('height', '100%')
			.attr('viewBox', `0 0 ${width} ${height}`);

		// Background gradient
		const defs = svg.append('defs');

		// Glow filter
		const filter = defs.append('filter').attr('id', 'glow');
		filter.append('feGaussianBlur').attr('stdDeviation', '4').attr('result', 'coloredBlur');
		const feMerge = filter.append('feMerge');
		feMerge.append('feMergeNode').attr('in', 'coloredBlur');
		feMerge.append('feMergeNode').attr('in', 'SourceGraphic');

		// Pulse filter for active nodes
		const pulseFilter = defs.append('filter').attr('id', 'pulse-glow');
		pulseFilter.append('feGaussianBlur').attr('stdDeviation', '8').attr('result', 'blur');
		const pulseMerge = pulseFilter.append('feMerge');
		pulseMerge.append('feMergeNode').attr('in', 'blur');
		pulseMerge.append('feMergeNode').attr('in', 'SourceGraphic');

		const g = svg.append('g');

		// Zoom behavior
		const zoom = d3.zoom<SVGSVGElement, unknown>()
			.scaleExtent([0.3, 3])
			.on('zoom', (event: d3.D3ZoomEvent<SVGSVGElement, unknown>) => {
				g.attr('transform', event.transform.toString());
			});

		svg.call(zoom);

		linkGroup = g.append('g').attr('class', 'links');
		nodeGroup = g.append('g').attr('class', 'nodes');

		simulation = d3
			.forceSimulation<GraphNode>()
			.force('link', d3.forceLink<GraphNode, GraphLink>().id((d) => d.id).distance(120).strength(0.5))
			.force('charge', d3.forceManyBody().strength(-300))
			.force('center', d3.forceCenter(width / 2, height / 2))
			.force('collision', d3.forceCollide().radius(45))
			.on('tick', ticked);

		// Handle resize
		const observer = new ResizeObserver((entries) => {
			const entry = entries[0];
			if (entry) {
				width = entry.contentRect.width;
				height = entry.contentRect.height;
				svg.attr('viewBox', `0 0 ${width} ${height}`);
				simulation.force('center', d3.forceCenter(width / 2, height / 2));
				simulation.alpha(0.3).restart();
			}
		});
		observer.observe(container);
	});

	$effect(() => {
		if (!simulation || !nodeGroup || !linkGroup) return;
		updateGraph(nodes, links);
	});

	function updateGraph(nodeData: GraphNode[], linkData: GraphLink[]) {
		// Update links
		const linkSel = linkGroup
			.selectAll<SVGLineElement, GraphLink>('line')
			.data(linkData, (d: GraphLink) => {
				const sourceId = typeof d.source === 'string' ? d.source : d.source?.id;
				const targetId = typeof d.target === 'string' ? d.target : d.target?.id;
				return `${sourceId}-${targetId}`;
			});

		linkSel.exit().transition().duration(300).attr('opacity', 0).remove();

		const linkEnter = linkSel
			.enter()
			.append('line')
			.attr('stroke', 'rgba(255, 255, 255, 0.08)')
			.attr('stroke-width', 1.5)
			.attr('opacity', 0);

		linkEnter.transition().duration(500).attr('opacity', 1);

		const linkMerged = linkEnter.merge(linkSel);
		linkMerged
			.attr('stroke', (d: GraphLink) => (d.active ? 'rgba(239, 68, 68, 0.4)' : 'rgba(255, 255, 255, 0.08)'))
			.attr('stroke-width', (d: GraphLink) => (d.active ? 2.5 : 1.5));

		// Update nodes
		const nodeSel = nodeGroup
			.selectAll<SVGGElement, GraphNode>('.node')
			.data(nodeData, (d: GraphNode) => d.id);

		nodeSel.exit().transition().duration(300).attr('opacity', 0).remove();

		const nodeEnter = nodeSel
			.enter()
			.append('g')
			.attr('class', 'node')
			.attr('opacity', 0)
			.call(
				d3.drag<SVGGElement, GraphNode>()
					.on('start', dragStarted)
					.on('drag', dragged)
					.on('end', dragEnded) as any
			);

		// Outer glow circle
		nodeEnter
			.append('circle')
			.attr('class', 'glow')
			.attr('r', 28)
			.attr('fill', 'transparent');

		// Main circle
		nodeEnter
			.append('circle')
			.attr('class', 'main')
			.attr('r', 20)
			.attr('stroke-width', 2);

		// Inner pokeball line
		nodeEnter
			.append('line')
			.attr('class', 'pokeball-line')
			.attr('x1', -20)
			.attr('y1', 0)
			.attr('x2', 20)
			.attr('y2', 0)
			.attr('stroke', 'rgba(255, 255, 255, 0.15)')
			.attr('stroke-width', 1.5);

		// Center dot
		nodeEnter
			.append('circle')
			.attr('class', 'center-dot')
			.attr('r', 5)
			.attr('stroke-width', 2);

		// Label
		nodeEnter
			.append('text')
			.attr('class', 'label')
			.attr('dy', 34)
			.attr('text-anchor', 'middle')
			.attr('fill', 'rgba(255, 255, 255, 0.6)')
			.attr('font-size', '11px')
			.attr('font-family', 'Inter, sans-serif')
			.attr('font-weight', '500')
			.text((d: GraphNode) => d.name);

		nodeEnter.transition().duration(600).attr('opacity', 1);

		const nodeMerged = nodeEnter.merge(nodeSel);

		// Update colors based on status
		nodeMerged
			.select('.main')
			.transition()
			.duration(300)
			.attr('fill', (d: GraphNode) => {
				const color = statusColors[d.status] || '#6b7280';
				return color;
			})
			.attr('stroke', (d: GraphNode) => {
				const color = statusColors[d.status] || '#6b7280';
				return color;
			})
			.attr('fill-opacity', 0.15)
			.attr('stroke-opacity', 0.8);

		nodeMerged
			.select('.glow')
			.transition()
			.duration(300)
			.attr('stroke', (d: GraphNode) => statusGlows[d.status] || 'transparent')
			.attr('stroke-width', (d: GraphNode) => (d.status === 'working' || d.status === 'collaborating' ? 3 : 0))
			.attr('fill', 'transparent');

		nodeMerged
			.select('.center-dot')
			.transition()
			.duration(300)
			.attr('fill', (d: GraphNode) => statusColors[d.status] || '#6b7280')
			.attr('stroke', (d: GraphNode) => {
				const color = statusColors[d.status] || '#6b7280';
				return color;
			})
			.attr('stroke-opacity', 0.5);

		// Apply working/collaborating animation class
		nodeMerged.select('.glow')
			.classed('pulsing', (d: GraphNode) => d.status === 'working' || d.status === 'collaborating');

		// Update simulation
		simulation.nodes(nodeData);
		const linkForce = simulation.force('link') as d3.ForceLink<GraphNode, GraphLink>;
		linkForce.links(linkData);
		simulation.alpha(0.5).restart();
	}

	function ticked() {
		linkGroup
			.selectAll<SVGLineElement, GraphLink>('line')
			.attr('x1', (d: any) => d.source.x)
			.attr('y1', (d: any) => d.source.y)
			.attr('x2', (d: any) => d.target.x)
			.attr('y2', (d: any) => d.target.y);

		nodeGroup
			.selectAll<SVGGElement, GraphNode>('.node')
			.attr('transform', (d: GraphNode) => `translate(${d.x}, ${d.y})`);
	}

	function dragStarted(event: d3.D3DragEvent<SVGGElement, GraphNode, GraphNode>) {
		if (!event.active) simulation.alphaTarget(0.3).restart();
		event.subject.fx = event.subject.x;
		event.subject.fy = event.subject.y;
	}

	function dragged(event: d3.D3DragEvent<SVGGElement, GraphNode, GraphNode>) {
		event.subject.fx = event.x;
		event.subject.fy = event.y;
	}

	function dragEnded(event: d3.D3DragEvent<SVGGElement, GraphNode, GraphNode>) {
		if (!event.active) simulation.alphaTarget(0);
		event.subject.fx = null;
		event.subject.fy = null;
	}

	onDestroy(() => {
		if (simulation) simulation.stop();
	});
</script>

<div class="force-graph" bind:this={container}>
	{#if nodes.length === 0}
		<div class="placeholder">
			<div class="pokeball-icon">
				<div class="top-half"></div>
				<div class="center-band"></div>
				<div class="center-button"></div>
				<div class="bottom-half"></div>
			</div>
			<p>Swarm visualization will appear here</p>
		</div>
	{/if}
</div>

<style>
	.force-graph {
		width: 100%;
		height: 100%;
		position: relative;
		border-radius: 16px;
		overflow: hidden;
		background: radial-gradient(ellipse at center, rgba(20, 20, 35, 1) 0%, rgba(10, 10, 15, 1) 100%);
	}

	.force-graph :global(svg) {
		display: block;
	}

	.force-graph :global(.pulsing) {
		animation: pulse-glow 2s ease-in-out infinite;
	}

	@keyframes pulse-glow {
		0%, 100% {
			stroke-opacity: 0.3;
		}
		50% {
			stroke-opacity: 0.8;
		}
	}

	.placeholder {
		position: absolute;
		inset: 0;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 16px;
		color: rgba(255, 255, 255, 0.12);
	}

	.placeholder p {
		font-size: 14px;
		font-weight: 500;
	}

	.pokeball-icon {
		width: 64px;
		height: 64px;
		border-radius: 50%;
		border: 3px solid rgba(255, 255, 255, 0.1);
		position: relative;
		overflow: hidden;
		opacity: 0.3;
	}

	.top-half {
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		height: 50%;
		background: rgba(239, 68, 68, 0.3);
	}

	.bottom-half {
		position: absolute;
		bottom: 0;
		left: 0;
		right: 0;
		height: 50%;
		background: rgba(255, 255, 255, 0.05);
	}

	.center-band {
		position: absolute;
		top: 50%;
		left: 0;
		right: 0;
		height: 4px;
		background: rgba(255, 255, 255, 0.15);
		transform: translateY(-50%);
		z-index: 1;
	}

	.center-button {
		position: absolute;
		top: 50%;
		left: 50%;
		width: 16px;
		height: 16px;
		border-radius: 50%;
		border: 3px solid rgba(255, 255, 255, 0.15);
		background: rgba(10, 10, 15, 1);
		transform: translate(-50%, -50%);
		z-index: 2;
	}
</style>
