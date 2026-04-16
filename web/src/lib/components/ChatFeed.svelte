<script lang="ts">
	import type { ChatMessage } from '$lib/types';
	import { tick } from 'svelte';

	let { messages }: { messages: ChatMessage[] } = $props();
	let feedEl: HTMLDivElement;

	$effect(() => {
		if (messages.length > 0) {
			tick().then(() => {
				if (feedEl) {
					feedEl.scrollTop = feedEl.scrollHeight;
				}
			});
		}
	});

	function formatTime(timestamp: string): string {
		return new Date(timestamp).toLocaleTimeString('en-US', {
			hour: '2-digit',
			minute: '2-digit',
			second: '2-digit',
		});
	}

	const outputTypeStyles: Record<string, { color: string; label: string }> = {
		thinking: { color: '#a78bfa', label: '💭 Thinking' },
		deliverable: { color: '#22c55e', label: '📄 Deliverable' },
		status_update: { color: '#06b6d4', label: '📡 Update' },
		question: { color: '#f59e0b', label: '❓ Question' },
	};
</script>

<div class="chat-feed" bind:this={feedEl}>
	{#each messages as msg (msg.id)}
		<div class="message" class:system={msg.type === 'system'} class:agent-output={msg.type === 'output'} class:agent-message={msg.type === 'message'}>
			{#if msg.type === 'system'}
				<div class="system-msg">
					<span class="time">{formatTime(msg.timestamp)}</span>
					<span class="content">{msg.content}</span>
				</div>
			{:else if msg.type === 'output'}
				<div class="output-msg" class:thinking={msg.output_type === 'thinking'}>
					<div class="msg-header">
						<span class="sender">{msg.from_name}</span>
						{#if msg.output_type}
							{@const style = outputTypeStyles[msg.output_type]}
							<span class="output-badge" style="color: {style?.color}; border-color: {style?.color}33">
								{style?.label}
							</span>
						{/if}
						<span class="time">{formatTime(msg.timestamp)}</span>
					</div>
					<div class="msg-content">
						{#if msg.output_type === 'thinking'}
							<div class="thought-stream">
								<span class="pulse"></span>
								{msg.content}
							</div>
						{:else}
							{msg.content}
						{/if}
					</div>
				</div>
			{:else}
				<div class="direct-msg">
					<div class="msg-header">
						<span class="sender">{msg.from_name}</span>
						<span class="arrow">→</span>
						<span class="receiver">{msg.to_name}</span>
						<span class="time">{formatTime(msg.timestamp)}</span>
					</div>
					<div class="msg-content">{msg.content}</div>
				</div>
			{/if}
		</div>
	{/each}

	{#if messages.length > 0 && messages[messages.length - 1].type === 'output' && messages[messages.length - 1].output_type === 'thinking'}
		<div class="typing-indicator">
			<span></span><span></span><span></span>
		</div>
	{/if}

	{#if messages.length === 0}
		<div class="empty">
			<span class="empty-icon">💬</span>
			<p>Waiting for swarm activity...</p>
		</div>
	{/if}
</div>

<style>
	.chat-feed {
		display: flex;
		flex-direction: column;
		gap: 4px;
		overflow-y: auto;
		max-height: 100%;
		padding: 12px;
		scroll-behavior: smooth;
	}

	.chat-feed::-webkit-scrollbar {
		width: 4px;
	}

	.chat-feed::-webkit-scrollbar-track {
		background: transparent;
	}

	.chat-feed::-webkit-scrollbar-thumb {
		background: rgba(255, 255, 255, 0.1);
		border-radius: 2px;
	}

	.message {
		animation: fadeIn 0.3s ease;
	}

	@keyframes fadeIn {
		from {
			opacity: 0;
			transform: translateY(4px);
		}
		to {
			opacity: 1;
			transform: none;
		}
	}

	.system-msg {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 10px;
		font-size: 12px;
		color: rgba(255, 255, 255, 0.4);
	}

	.system-msg .content {
		flex: 1;
	}

	.output-msg,
	.direct-msg {
		padding: 10px 12px;
		background: rgba(255, 255, 255, 0.02);
		border: 1px solid rgba(255, 255, 255, 0.04);
		border-radius: 10px;
		transition: background 0.2s ease;
	}

	.output-msg:hover,
	.direct-msg:hover {
		background: rgba(255, 255, 255, 0.04);
	}

	.msg-header {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 6px;
		flex-wrap: wrap;
	}

	.sender {
		font-size: 12px;
		font-weight: 600;
		color: #ef4444;
	}

	.receiver {
		font-size: 12px;
		font-weight: 600;
		color: #06b6d4;
	}

	.arrow {
		font-size: 11px;
		color: rgba(255, 255, 255, 0.2);
	}

	.output-badge {
		font-size: 10px;
		font-weight: 500;
		padding: 2px 6px;
		border-radius: 4px;
		background: rgba(255, 255, 255, 0.05);
	}

	.time {
		font-size: 10px;
		color: rgba(255, 255, 255, 0.15);
		font-family: 'JetBrains Mono', monospace;
		margin-left: auto;
	}

	.msg-content {
		font-size: 12px;
		color: rgba(255, 255, 255, 0.65);
		line-height: 1.5;
		white-space: pre-wrap;
		word-break: break-word;
	}

	.empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 8px;
		padding: 40px;
		color: rgba(255, 255, 255, 0.15);
	}

	.empty-icon {
		font-size: 32px;
		opacity: 0.5;
	}

	.empty p {
		font-size: 13px;
		margin: 0;
	}
</style>
