<script lang="ts">
	import './layout.css';
	let { children } = $props();
</script>

<div class="gradient-bg"></div>

<div class="app-shell">
	<header class="app-header">
		<a href="/" class="logo-link">
			<div class="logo">
				<div class="logo-icon">
					<div class="pokeball-mini">
						<div class="pb-top"></div>
						<div class="pb-band"></div>
						<div class="pb-dot"></div>
					</div>
				</div>
				<div class="logo-text">
					<span class="logo-name">Pokedex</span>
					<span class="logo-sub">Swarm</span>
				</div>
			</div>
		</a>
		<nav class="header-nav">
			<span class="status-pill">
				<span class="dot"></span>
				Simulation Mode
			</span>
		</nav>
	</header>

	<main class="app-main">
		{@render children()}
	</main>
</div>

<style>
	.app-shell {
		min-height: 100vh;
		display: flex;
		flex-direction: column;
	}

	.app-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px 24px;
		border-bottom: 1px solid rgba(255, 255, 255, 0.04);
		backdrop-filter: blur(12px);
		position: sticky;
		top: 0;
		z-index: 50;
		background: rgba(10, 10, 15, 0.8);
	}

	.logo-link {
		text-decoration: none;
		color: inherit;
	}

	.logo {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.logo-icon {
		width: 32px;
		height: 32px;
		border-radius: 8px;
		background: linear-gradient(135deg, #ef4444, #dc2626);
		display: flex;
		align-items: center;
		justify-content: center;
		position: relative;
	}

	.pokeball-mini {
		width: 20px;
		height: 20px;
		border-radius: 50%;
		border: 2px solid rgba(255, 255, 255, 0.9);
		position: relative;
		overflow: hidden;
	}

	.pb-top {
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		height: 50%;
		background: rgba(255, 255, 255, 0.3);
	}

	.pb-band {
		position: absolute;
		top: 50%;
		left: 0;
		right: 0;
		height: 2px;
		background: rgba(255, 255, 255, 0.9);
		transform: translateY(-50%);
		z-index: 1;
	}

	.pb-dot {
		position: absolute;
		top: 50%;
		left: 50%;
		width: 6px;
		height: 6px;
		border-radius: 50%;
		border: 1.5px solid rgba(255, 255, 255, 0.9);
		background: rgba(0, 0, 0, 0.5);
		transform: translate(-50%, -50%);
		z-index: 2;
	}

	.logo-text {
		display: flex;
		align-items: baseline;
		gap: 6px;
	}

	.logo-name {
		font-size: 18px;
		font-weight: 700;
		letter-spacing: -0.02em;
	}

	.logo-sub {
		font-size: 14px;
		font-weight: 500;
		color: rgba(255, 255, 255, 0.4);
	}

	.header-nav {
		display: flex;
		align-items: center;
		gap: 16px;
	}

	.status-pill {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 12px;
		font-weight: 500;
		padding: 6px 12px;
		border-radius: 20px;
		background: rgba(239, 68, 68, 0.1);
		border: 1px solid rgba(239, 68, 68, 0.2);
		color: #ef4444;
	}

	.dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: #ef4444;
		animation: pulse 2s ease-in-out infinite;
	}

	@keyframes pulse {
		0%, 100% { opacity: 1; }
		50% { opacity: 0.4; }
	}

	.app-main {
		flex: 1;
		display: flex;
		flex-direction: column;
	}
</style>
