<script lang="ts">
	let { onSubmit }: { onSubmit: (goal: string) => void } = $props();
	let goal = $state('');
	let isFocused = $state(false);

	function handleSubmit() {
		if (goal.trim()) {
			onSubmit(goal.trim());
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			handleSubmit();
		}
	}

	const examples = [
		'Build a SaaS landing page with pricing and auth',
		'Design a mobile app for habit tracking',
		'Create a REST API for a social media platform',
		'Develop a marketing strategy for a crypto project',
	];
</script>

<div class="goal-input-wrapper">
	<div class="input-container" class:focused={isFocused}>
		<div class="glow-border"></div>
		<div class="input-inner">
			<div class="icon">
				<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
					<circle cx="12" cy="12" r="10"></circle>
					<path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20"></path>
					<path d="M2 12h20"></path>
				</svg>
			</div>
			<textarea
				id="goal-input"
				bind:value={goal}
				onfocus={() => (isFocused = true)}
				onblur={() => (isFocused = false)}
				onkeydown={handleKeydown}
				placeholder="What do you want your swarm to achieve?"
				rows={1}
			></textarea>
			<button
				id="submit-goal"
				class="submit-btn"
				class:active={goal.trim().length > 0}
				onclick={handleSubmit}
				disabled={!goal.trim()}
				aria-label="Submit goal"
			>
				<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
					<path d="M5 12h14"></path>
					<path d="m12 5 7 7-7 7"></path>
				</svg>
			</button>
		</div>
	</div>

	<div class="examples">
		<span class="examples-label">Try:</span>
		{#each examples as example}
			<button class="example-chip" onclick={() => (goal = example)}>
				{example}
			</button>
		{/each}
	</div>
</div>

<style>
	.goal-input-wrapper {
		width: 100%;
		max-width: 720px;
		margin: 0 auto;
	}

	.input-container {
		position: relative;
		border-radius: 16px;
		transition: all 0.3s ease;
	}

	.glow-border {
		position: absolute;
		inset: -2px;
		border-radius: 18px;
		background: conic-gradient(
			from 0deg,
			#ef4444,
			#f97316,
			#ef4444,
			#dc2626,
			#ef4444
		);
		opacity: 0.4;
		filter: blur(4px);
		transition: opacity 0.3s ease;
		animation: spin 4s linear infinite;
	}

	.focused .glow-border {
		opacity: 0.8;
		filter: blur(6px);
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.input-inner {
		position: relative;
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 16px 20px;
		background: rgba(15, 15, 25, 0.95);
		border: 1px solid rgba(255, 255, 255, 0.08);
		border-radius: 16px;
		backdrop-filter: blur(20px);
	}

	.icon {
		color: rgba(255, 255, 255, 0.3);
		flex-shrink: 0;
	}

	textarea {
		flex: 1;
		background: none;
		border: none;
		outline: none;
		color: white;
		font-family: 'Inter', sans-serif;
		font-size: 16px;
		font-weight: 400;
		resize: none;
		line-height: 1.5;
	}

	textarea::placeholder {
		color: rgba(255, 255, 255, 0.3);
	}

	.submit-btn {
		flex-shrink: 0;
		width: 40px;
		height: 40px;
		border-radius: 12px;
		border: none;
		background: rgba(255, 255, 255, 0.05);
		color: rgba(255, 255, 255, 0.2);
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: all 0.2s ease;
	}

	.submit-btn.active {
		background: linear-gradient(135deg, #ef4444, #dc2626);
		color: white;
		box-shadow: 0 0 20px rgba(239, 68, 68, 0.3);
	}

	.submit-btn.active:hover {
		transform: scale(1.05);
		box-shadow: 0 0 30px rgba(239, 68, 68, 0.5);
	}

	.submit-btn:disabled {
		cursor: not-allowed;
	}

	.examples {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		margin-top: 16px;
		align-items: center;
	}

	.examples-label {
		font-size: 13px;
		color: rgba(255, 255, 255, 0.3);
		font-weight: 500;
	}

	.example-chip {
		font-size: 12px;
		padding: 6px 12px;
		border-radius: 20px;
		border: 1px solid rgba(255, 255, 255, 0.08);
		background: rgba(255, 255, 255, 0.03);
		color: rgba(255, 255, 255, 0.5);
		cursor: pointer;
		font-family: 'Inter', sans-serif;
		transition: all 0.2s ease;
	}

	.example-chip:hover {
		border-color: rgba(239, 68, 68, 0.3);
		color: rgba(255, 255, 255, 0.8);
		background: rgba(239, 68, 68, 0.08);
	}
</style>
