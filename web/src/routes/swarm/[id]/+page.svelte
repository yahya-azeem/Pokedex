<script lang="ts">
	import { page } from '$app/state';
	import { onMount } from 'svelte';
	import WorkflowBar from '$lib/components/WorkflowBar.svelte';
	import ForceGraph from '$lib/components/ForceGraph.svelte';
	import AgentCard from '$lib/components/AgentCard.svelte';
	import ChatFeed from '$lib/components/ChatFeed.svelte';
	import { getSwarmState, initSwarmConnection } from '$lib/stores/swarm.svelte';

	const state = getSwarmState();
	const swarmId = page.params.id;

	onMount(() => {
		initSwarmConnection();
	});
</script>

<svelte:head>
	<title>Swarm Dashboard — Pokedex Swarm</title>
</svelte:head>

<div class="dashboard">
	<!-- Top: Workflow Progress Bar -->
	<div class="workflow-section">
		<WorkflowBar currentPhase={state.phase} />
	</div>

	<!-- Main Content: Graph + Sidebar -->
	<div class="main-content">
		<!-- Left: Force Graph Visualization -->
		<div class="graph-section glass-panel">
			<div class="section-header">
				<h2>Swarm Visualization</h2>
				<div class="graph-stats">
					<span class="stat-pill">{state.agents.length} agents</span>
					<span class="stat-pill">{state.graphLinks.length} links</span>
				</div>
			</div>
			<div class="graph-container">
				<ForceGraph nodes={state.graphNodes} links={state.graphLinks} />
			</div>
		</div>

		<!-- Right: Agent Panel -->
		<div class="agent-panel">
			<div class="section-header">
				<h2>Pokeballs</h2>
				<span class="phase-badge">{formatPhase(state.phase)}</span>
			</div>
			<div class="agent-list">
				{#each state.agents as agent (agent.id)}
					<AgentCard {agent} />
				{:else}
					<div class="empty-agents">
						<span class="empty-icon">⚡</span>
						<p>Agents will appear here as they spawn</p>
					</div>
				{/each}
			</div>
		</div>
	</div>

	<!-- Bottom: Chat Feed + Goal Info -->
	<div class="bottom-section">
		<div class="goal-info glass-panel">
			<div class="goal-label">Goal</div>
			<div class="goal-text">{state.goal || 'Loading...'}</div>
			{#if state.duration > 0}
				<div class="duration">
					⏱ {state.duration.toFixed(1)}s
				</div>
			{/if}
		</div>
		<div class="chat-section glass-panel">
			<div class="section-header">
				<h2>Activity Feed</h2>
				<span class="msg-count">{state.chatMessages.length} messages</span>
			</div>
			<div class="chat-container">
				<ChatFeed messages={state.chatMessages} />
			</div>
		</div>
	</div>

	<!-- Report Modal -->
	{#if state.report}
		<div class="report-overlay">
			<div class="report-modal glass-panel">
				<div class="report-header">
					<h2>📊 Swarm Report</h2>
					<button class="close-btn" onclick={() => {}}>✕</button>
				</div>
				<div class="report-content">
					{state.report}
				</div>
			</div>
		</div>
	{/if}
</div>

<script lang="ts" module>
	function formatPhase(phase: string): string {
		const labels: Record<string, string> = {
			manifest: 'Generating Manifest',
			populating_agents: 'Populating Agents',
			simulating: 'Simulating',
			report_generation: 'Generating Report',
			completed: 'Completed ✅',
			failed: 'Failed ❌',
		};
		return labels[phase] || phase;
	}
</script>

<style>
	.dashboard {
		display: flex;
		flex-direction: column;
		gap: 16px;
		padding: 16px 24px 24px;
		height: calc(100vh - 65px);
		overflow: hidden;
	}

	.workflow-section {
		flex-shrink: 0;
	}

	.main-content {
		display: grid;
		grid-template-columns: 1fr 320px;
		gap: 16px;
		flex: 1;
		min-height: 0;
	}

	.graph-section {
		display: flex;
		flex-direction: column;
		padding: 16px;
		overflow: hidden;
	}

	.graph-container {
		flex: 1;
		min-height: 0;
		border-radius: 12px;
		overflow: hidden;
	}

	.agent-panel {
		display: flex;
		flex-direction: column;
		gap: 12px;
		overflow: hidden;
	}

	.agent-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
		overflow-y: auto;
		flex: 1;
		padding-right: 4px;
	}

	.bottom-section {
		display: grid;
		grid-template-columns: 300px 1fr;
		gap: 16px;
		flex-shrink: 0;
		max-height: 240px;
	}

	.goal-info {
		padding: 16px;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.goal-label {
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: rgba(255, 255, 255, 0.3);
	}

	.goal-text {
		font-size: 14px;
		line-height: 1.5;
		color: rgba(255, 255, 255, 0.7);
	}

	.duration {
		font-size: 12px;
		color: rgba(255, 255, 255, 0.3);
		font-family: 'JetBrains Mono', monospace;
		margin-top: auto;
	}

	.chat-section {
		display: flex;
		flex-direction: column;
		padding: 16px;
		overflow: hidden;
	}

	.chat-container {
		flex: 1;
		min-height: 0;
		overflow: hidden;
	}

	.section-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 12px;
		flex-shrink: 0;
	}

	.section-header h2 {
		font-size: 14px;
		font-weight: 600;
		color: rgba(255, 255, 255, 0.8);
	}

	.graph-stats {
		display: flex;
		gap: 8px;
	}

	.stat-pill,
	.msg-count {
		font-size: 11px;
		padding: 4px 8px;
		border-radius: 6px;
		background: rgba(255, 255, 255, 0.05);
		color: rgba(255, 255, 255, 0.4);
		font-family: 'JetBrains Mono', monospace;
	}

	.phase-badge {
		font-size: 11px;
		font-weight: 500;
		padding: 4px 10px;
		border-radius: 8px;
		background: rgba(239, 68, 68, 0.1);
		color: #ef4444;
	}

	.empty-agents {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 10px;
		padding: 40px 20px;
		color: rgba(255, 255, 255, 0.12);
		text-align: center;
	}

	.empty-icon {
		font-size: 28px;
		opacity: 0.4;
	}

	.empty-agents p {
		font-size: 12px;
	}

	.report-overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.6);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 100;
		backdrop-filter: blur(8px);
	}

	.report-modal {
		max-width: 700px;
		width: 90%;
		max-height: 80vh;
		padding: 24px;
		overflow-y: auto;
	}

	.report-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 20px;
	}

	.report-header h2 {
		font-size: 18px;
		font-weight: 700;
	}

	.close-btn {
		width: 32px;
		height: 32px;
		border-radius: 8px;
		border: 1px solid rgba(255, 255, 255, 0.1);
		background: rgba(255, 255, 255, 0.03);
		color: rgba(255, 255, 255, 0.5);
		cursor: pointer;
		font-size: 14px;
		transition: all 0.2s ease;
	}

	.close-btn:hover {
		background: rgba(255, 255, 255, 0.08);
		color: white;
	}

	.report-content {
		font-size: 14px;
		line-height: 1.7;
		color: rgba(255, 255, 255, 0.65);
		white-space: pre-wrap;
	}
</style>
