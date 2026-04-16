<script lang="ts">
	import type { SwarmPhase } from '$lib/types';

	let { currentPhase }: { currentPhase: SwarmPhase } = $props();

	const phases: { key: SwarmPhase; label: string; icon: string }[] = [
		{ key: 'manifest', label: 'Manifest', icon: '📋' },
		{ key: 'populating_agents', label: 'Population', icon: '⚡' },
		{ key: 'simulating', label: 'Simulation', icon: '🔄' },
		{ key: 'report_generation', label: 'Report', icon: '📊' },
		{ key: 'completed', label: 'Complete', icon: '✅' },
	];

	const phaseOrder = phases.map((p) => p.key);

	function getPhaseState(phaseKey: SwarmPhase): 'completed' | 'active' | 'pending' {
		const currentIdx = phaseOrder.indexOf(currentPhase);
		const phaseIdx = phaseOrder.indexOf(phaseKey);

		if (currentPhase === 'failed') return phaseIdx <= phaseOrder.indexOf(currentPhase) ? 'completed' : 'pending';
		if (phaseIdx < currentIdx) return 'completed';
		if (phaseIdx === currentIdx) return 'active';
		return 'pending';
	}
</script>

<div class="workflow-bar">
	{#each phases as phase, i}
		{@const state = getPhaseState(phase.key)}
		<div class="step" class:completed={state === 'completed'} class:active={state === 'active'} class:pending={state === 'pending'}>
			<div class="step-indicator">
				{#if state === 'completed'}
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
						<polyline points="20 6 9 17 4 12"></polyline>
					</svg>
				{:else}
					<span class="step-icon">{phase.icon}</span>
				{/if}
			</div>
			<span class="step-label">{phase.label}</span>
		</div>
		{#if i < phases.length - 1}
			<div class="connector" class:filled={getPhaseState(phases[i + 1].key) !== 'pending'}></div>
		{/if}
	{/each}
</div>

<style>
	.workflow-bar {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 0;
		padding: 16px 24px;
		background: rgba(255, 255, 255, 0.02);
		border: 1px solid rgba(255, 255, 255, 0.06);
		border-radius: 16px;
		backdrop-filter: blur(12px);
	}

	.step {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 12px;
		border-radius: 10px;
		transition: all 0.3s ease;
	}

	.step-indicator {
		width: 28px;
		height: 28px;
		border-radius: 8px;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 14px;
		transition: all 0.3s ease;
	}

	.step.pending .step-indicator {
		background: rgba(255, 255, 255, 0.05);
		color: rgba(255, 255, 255, 0.2);
	}

	.step.active .step-indicator {
		background: linear-gradient(135deg, #ef4444, #f97316);
		color: white;
		box-shadow: 0 0 20px rgba(239, 68, 68, 0.4);
		animation: pulse 2s ease-in-out infinite;
	}

	.step.completed .step-indicator {
		background: rgba(34, 197, 94, 0.2);
		color: #22c55e;
	}

	.step-label {
		font-size: 13px;
		font-weight: 500;
		font-family: 'Inter', sans-serif;
		transition: color 0.3s ease;
	}

	.step.pending .step-label {
		color: rgba(255, 255, 255, 0.2);
	}

	.step.active .step-label {
		color: white;
	}

	.step.completed .step-label {
		color: rgba(255, 255, 255, 0.6);
	}

	.step-icon {
		font-size: 14px;
	}

	.connector {
		width: 32px;
		height: 2px;
		background: rgba(255, 255, 255, 0.06);
		border-radius: 1px;
		transition: background 0.3s ease;
	}

	.connector.filled {
		background: linear-gradient(90deg, rgba(34, 197, 94, 0.4), rgba(239, 68, 68, 0.4));
	}

	@keyframes pulse {
		0%, 100% {
			box-shadow: 0 0 20px rgba(239, 68, 68, 0.4);
		}
		50% {
			box-shadow: 0 0 30px rgba(239, 68, 68, 0.6);
		}
	}
</style>
