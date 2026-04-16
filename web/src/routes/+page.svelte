<script lang="ts">
	import { goto } from '$app/navigation';
	import GoalInput from '$lib/components/GoalInput.svelte';
	import { createSwarm, initSwarmConnection } from '$lib/stores/swarm.svelte';
	import { onMount } from 'svelte';

	onMount(() => {
		initSwarmConnection();
	});

	async function handleGoalSubmit(goal: string) {
		const swarmId = await createSwarm(goal);
		if (swarmId) {
			goto(`/swarm/${swarmId}`);
		}
	}
</script>

<svelte:head>
	<title>Pokedex Swarm — AI Agent Orchestration</title>
	<meta name="description" content="Generate teams of specialized AI agents working together towards your goal. Powered by multi-agent orchestration." />
</svelte:head>

<div class="landing">
	<div class="hero">
		<div class="hero-badge">
			<span class="badge-dot"></span>
			Multi-Agent Orchestration
		</div>

		<h1 class="hero-title">
			Deploy your<br />
			<span class="gradient-text">Agent Swarm</span>
		</h1>

		<p class="hero-subtitle">
			One goal. Multiple specialized AI agents. Real-time collaboration.<br />
			Describe what you want to achieve and watch your swarm come alive.
		</p>

		<GoalInput onSubmit={handleGoalSubmit} />
	</div>

	<div class="features">
		<div class="feature">
			<div class="feature-icon">🧬</div>
			<h3>Persona-Driven</h3>
			<p>Agents inherit specialized personas from a curated library of 70+ roles</p>
		</div>
		<div class="feature">
			<div class="feature-icon">🔗</div>
			<h3>Collaborative</h3>
			<p>Agents communicate, share context, and build on each other's outputs</p>
		</div>
		<div class="feature">
			<div class="feature-icon">📊</div>
			<h3>Visualized</h3>
			<p>Watch your swarm in real-time with a D3.js force-directed graph</p>
		</div>
		<div class="feature">
			<div class="feature-icon">⚡</div>
			<h3>Optimized</h3>
			<p>Model selection powered by promptfoo benchmarks per agent role</p>
		</div>
	</div>
</div>

<style>
	.landing {
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: 60px 24px 80px;
		min-height: calc(100vh - 65px);
	}

	.hero {
		text-align: center;
		max-width: 800px;
		margin-bottom: 80px;
	}

	.hero-badge {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		font-size: 13px;
		font-weight: 500;
		padding: 8px 16px;
		border-radius: 24px;
		background: rgba(239, 68, 68, 0.08);
		border: 1px solid rgba(239, 68, 68, 0.15);
		color: rgba(239, 68, 68, 0.9);
		margin-bottom: 28px;
	}

	.badge-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: #ef4444;
		animation: pulse 2s ease-in-out infinite;
	}

	@keyframes pulse {
		0%, 100% { opacity: 1; }
		50% { opacity: 0.3; }
	}

	.hero-title {
		font-size: clamp(36px, 6vw, 64px);
		font-weight: 800;
		line-height: 1.1;
		letter-spacing: -0.03em;
		margin-bottom: 20px;
		color: white;
	}

	.gradient-text {
		background: linear-gradient(135deg, #ef4444, #f97316, #ef4444);
		background-size: 200% auto;
		-webkit-background-clip: text;
		-webkit-text-fill-color: transparent;
		background-clip: text;
		animation: gradient-shift 3s ease-in-out infinite;
	}

	@keyframes gradient-shift {
		0%, 100% { background-position: 0% center; }
		50% { background-position: 100% center; }
	}

	.hero-subtitle {
		font-size: 17px;
		line-height: 1.6;
		color: rgba(255, 255, 255, 0.4);
		margin-bottom: 40px;
		max-width: 560px;
		margin-inline: auto;
	}

	.features {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
		gap: 20px;
		max-width: 900px;
		width: 100%;
	}

	.feature {
		padding: 24px;
		background: rgba(255, 255, 255, 0.02);
		border: 1px solid rgba(255, 255, 255, 0.05);
		border-radius: 16px;
		text-align: center;
		transition: all 0.3s ease;
	}

	.feature:hover {
		background: rgba(255, 255, 255, 0.04);
		border-color: rgba(255, 255, 255, 0.08);
		transform: translateY(-2px);
	}

	.feature-icon {
		font-size: 28px;
		margin-bottom: 12px;
	}

	.feature h3 {
		font-size: 15px;
		font-weight: 600;
		margin-bottom: 8px;
		color: white;
	}

	.feature p {
		font-size: 13px;
		color: rgba(255, 255, 255, 0.35);
		line-height: 1.5;
	}
</style>
