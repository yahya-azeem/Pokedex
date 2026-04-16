<script lang="ts">
	import type { Pokeball } from '$lib/types';

	let { agent }: { agent: Pokeball } = $props();

	const statusColors: Record<string, string> = {
		idle: '#6b7280',
		working: '#06b6d4',
		collaborating: '#f59e0b',
		finished: '#22c55e',
		error: '#ef4444',
	};

	const statusLabels: Record<string, string> = {
		idle: 'Idle',
		working: 'Working',
		collaborating: 'Collaborating',
		finished: 'Finished',
		error: 'Error',
	};

	const categoryIcons: Record<string, string> = {
		engineering: '⚙️',
		design: '🎨',
		marketing: '📣',
		sales: '💰',
		testing: '🧪',
		strategy: '♟️',
		product: '📦',
		support: '🎧',
		academic: '📚',
		'game-development': '🎮',
		specialized: '🔬',
		'project-management': '📋',
	};
</script>

<div class="agent-card" style="--status-color: {statusColors[agent.status] || '#6b7280'}">
	<div class="card-header">
		<div class="avatar">
			{categoryIcons[agent.persona_category || ''] || '🤖'}
		</div>
		<div class="info">
			<h4 class="name">{agent.name}</h4>
			<p class="role">{agent.role}</p>
		</div>
		<div class="status-badge" class:working={agent.status === 'working'} class:collaborating={agent.status === 'collaborating'}>
			<span class="status-dot"></span>
			{statusLabels[agent.status]}
		</div>
	</div>

	<div class="card-body">
		<div class="meta-row">
			<span class="meta-label">Model</span>
			<span class="meta-value">{agent.model.split('-').slice(0, 2).join(' ')}</span>
		</div>
		{#if agent.task}
			<div class="task-preview">
				{agent.task.length > 80 ? agent.task.slice(0, 80) + '...' : agent.task}
			</div>
		{/if}
		<div class="stats">
			<span class="stat">📤 {agent.output_count} outputs</span>
			<span class="stat">🔗 {agent.collaboration_targets.length} links</span>
		</div>
	</div>
</div>

<style>
	.agent-card {
		background: rgba(255, 255, 255, 0.03);
		border: 1px solid rgba(255, 255, 255, 0.06);
		border-left: 3px solid var(--status-color);
		border-radius: 12px;
		padding: 14px;
		transition: all 0.2s ease;
		backdrop-filter: blur(8px);
	}

	.agent-card:hover {
		background: rgba(255, 255, 255, 0.05);
		border-color: rgba(255, 255, 255, 0.1);
		transform: translateY(-1px);
	}

	.card-header {
		display: flex;
		align-items: center;
		gap: 10px;
		margin-bottom: 10px;
	}

	.avatar {
		width: 36px;
		height: 36px;
		border-radius: 10px;
		background: rgba(255, 255, 255, 0.05);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 18px;
		flex-shrink: 0;
	}

	.info {
		flex: 1;
		min-width: 0;
	}

	.name {
		font-size: 13px;
		font-weight: 600;
		color: white;
		margin: 0;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.role {
		font-size: 11px;
		color: rgba(255, 255, 255, 0.4);
		margin: 2px 0 0;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.status-badge {
		display: flex;
		align-items: center;
		gap: 5px;
		font-size: 11px;
		font-weight: 500;
		padding: 4px 8px;
		border-radius: 6px;
		background: rgba(255, 255, 255, 0.05);
		color: var(--status-color);
		flex-shrink: 0;
	}

	.status-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--status-color);
	}

	.status-badge.working .status-dot,
	.status-badge.collaborating .status-dot {
		animation: blink 1s ease-in-out infinite;
	}

	@keyframes blink {
		0%, 100% { opacity: 1; }
		50% { opacity: 0.3; }
	}

	.card-body {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.meta-row {
		display: flex;
		justify-content: space-between;
		font-size: 11px;
	}

	.meta-label {
		color: rgba(255, 255, 255, 0.3);
	}

	.meta-value {
		color: rgba(255, 255, 255, 0.6);
		font-family: 'JetBrains Mono', monospace;
		font-size: 10px;
	}

	.task-preview {
		font-size: 11px;
		color: rgba(255, 255, 255, 0.35);
		line-height: 1.4;
		padding: 8px;
		background: rgba(0, 0, 0, 0.2);
		border-radius: 8px;
	}

	.stats {
		display: flex;
		gap: 12px;
		font-size: 11px;
		color: rgba(255, 255, 255, 0.3);
	}

	.stat {
		display: flex;
		align-items: center;
		gap: 4px;
	}
</style>
