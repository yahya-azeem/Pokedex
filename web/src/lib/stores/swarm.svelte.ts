import type {
	Pokeball,
	SwarmPhase,
	SwarmEvent,
	ChatMessage,
	GraphNode,
	GraphLink
} from '$lib/types';
import { WebSocketManager } from './websocket.svelte';

// Singleton WebSocket manager
export const wsManager = new WebSocketManager();

// Reactive swarm state using Svelte 5 runes
let _swarmId = $state<string | null>(null);
let _goal = $state('');
let _phase = $state<SwarmPhase>('manifest');
let _agents = $state<Pokeball[]>([]);
let _chatMessages = $state<ChatMessage[]>([]);
let _graphNodes = $state<GraphNode[]>([]);
let _graphLinks = $state<GraphLink[]>([]);
let _report = $state<string | null>(null);
let _isRunning = $state(false);
let _error = $state<string | null>(null);
let _duration = $state(0);

// Exported reactive getters
export function getSwarmState() {
	return {
		get swarmId() { return _swarmId; },
		get goal() { return _goal; },
		get phase() { return _phase; },
		get agents() { return _agents; },
		get chatMessages() { return _chatMessages; },
		get graphNodes() { return _graphNodes; },
		get graphLinks() { return _graphLinks; },
		get report() { return _report; },
		get isRunning() { return _isRunning; },
		get error() { return _error; },
		get duration() { return _duration; },
	};
}

// Initialize WebSocket and event handling
export function initSwarmConnection() {
	wsManager.connect();
	wsManager.onEvent(handleSwarmEvent);
}

// Create a new swarm via the API
export async function createSwarm(goal: string): Promise<string | null> {
	try {
		_error = null;
		_isRunning = true;
		_agents = [];
		_chatMessages = [];
		_graphNodes = [];
		_graphLinks = [];
		_report = null;
		_goal = goal;
		_phase = 'manifest';

		const response = await fetch('/api/swarm', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ goal }),
		});

		const data = await response.json();
		if (data.status === 'success') {
			_swarmId = data.swarm_id;
			return data.swarm_id;
		} else {
			_error = data.message;
			_isRunning = false;
			return null;
		}
	} catch (e) {
		_error = `Failed to create swarm: ${e}`;
		_isRunning = false;
		return null;
	}
}

// Handle incoming WebSocket events
function handleSwarmEvent(event: SwarmEvent) {
	switch (event.type) {
		case 'connected':
			addSystemMessage('Connected to Pokedex Swarm server');
			break;

		case 'SwarmCreated':
			addSystemMessage(`🔴 Swarm created for goal: "${event.data.goal}"`);
			break;

		case 'ManifestGenerated':
			addSystemMessage(
				`📋 Manifest generated — ${event.data.agent_count} agents planned`
			);
			break;

		case 'AgentSpawned': {
			const { agent_id, name, role, persona_category, model } = event.data;
			const newAgent: Pokeball = {
				id: agent_id,
				name,
				role,
				persona_category: persona_category,
				model,
				status: 'idle',
				task: null,
				output_count: 0,
				collaboration_targets: [],
				created_at: event.data.timestamp,
			};
			_agents = [..._agents, newAgent];
			_graphNodes = [
				..._graphNodes,
				{
					id: agent_id,
					name,
					role,
					status: 'idle',
					category: persona_category,
				},
			];
			addSystemMessage(`⚡ Agent spawned: **${name}** (${role})`);
			break;
		}

		case 'AgentStatusChanged': {
			const { agent_id, new_status } = event.data;
			_agents = _agents.map((a) =>
				a.id === agent_id ? { ...a, status: new_status as Pokeball['status'] } : a
			);
			_graphNodes = _graphNodes.map((n) =>
				n.id === agent_id ? { ...n, status: new_status as Pokeball['status'] } : n
			);
			break;
		}

		case 'AgentOutput': {
			const { agent_name, content, output_type, timestamp } = event.data;
			const msg: ChatMessage = {
				id: crypto.randomUUID(),
				type: 'output',
				from_name: agent_name,
				content,
				output_type,
				timestamp,
			};
			_chatMessages = [..._chatMessages, msg];

			// Update output count
			_agents = _agents.map((a) =>
				a.id === event.data.agent_id
					? { ...a, output_count: a.output_count + 1 }
					: a
			);
			break;
		}

		case 'AgentMessage': {
			const { from_name, to_name, message, from_agent_id, to_agent_id, timestamp } = event.data;
			const msg: ChatMessage = {
				id: crypto.randomUUID(),
				type: 'message',
				from_name,
				to_name,
				content: message,
				timestamp,
			};
			_chatMessages = [..._chatMessages, msg];

			// Add or activate graph link
			const existingLink = _graphLinks.find(
				(l) =>
					(typeof l.source === 'string' ? l.source : l.source.id) === from_agent_id &&
					(typeof l.target === 'string' ? l.target : l.target.id) === to_agent_id
			);
			if (!existingLink) {
				_graphLinks = [
					..._graphLinks,
					{ source: from_agent_id, target: to_agent_id, active: true },
				];
			} else {
				_graphLinks = _graphLinks.map((l) =>
					l === existingLink ? { ...l, active: true } : l
				);
			}
			break;
		}

		case 'PhaseChanged':
			_phase = event.data.new_phase as SwarmPhase;
			addSystemMessage(`🔄 Phase: ${formatPhase(event.data.new_phase)}`);
			break;

		case 'SwarmCompleted':
			_report = event.data.summary;
			_isRunning = false;
			_duration = event.data.duration_secs;
			addSystemMessage(
				`✅ Swarm completed in ${event.data.duration_secs.toFixed(1)}s — ${event.data.total_messages} total outputs`
			);
			break;

		case 'SwarmError':
			_error = event.data.error;
			_isRunning = false;
			addSystemMessage(`❌ Error: ${event.data.error}`);
			break;
	}
}

function addSystemMessage(content: string) {
	const msg: ChatMessage = {
		id: crypto.randomUUID(),
		type: 'system',
		from_name: 'System',
		content,
		timestamp: new Date().toISOString(),
	};
	_chatMessages = [..._chatMessages, msg];
}

function formatPhase(phase: string): string {
	const labels: Record<string, string> = {
		manifest: 'Generating Manifest',
		populating_agents: 'Populating Agents',
		simulating: 'Simulating Collaboration',
		report_generation: 'Generating Report',
		completed: 'Completed',
		failed: 'Failed',
	};
	return labels[phase] || phase;
}

// Cleanup
export function destroySwarmConnection() {
	wsManager.disconnect();
}
