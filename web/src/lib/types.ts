// Types mirroring the Rust backend types for the Pokedex Swarm frontend.

export interface Swarm {
	id: string;
	goal: string;
	phase: SwarmPhase;
	agents: Pokeball[];
	manifest: SwarmManifest | null;
	report: string | null;
	created_at: string;
	completed_at: string | null;
}

export type SwarmPhase =
	| 'manifest'
	| 'populating_agents'
	| 'simulating'
	| 'report_generation'
	| 'completed'
	| 'failed';

export interface Pokeball {
	id: string;
	name: string;
	role: string;
	persona_category: string | null;
	model: string;
	status: PokeballStatus;
	task: string | null;
	output_count: number;
	collaboration_targets: string[];
	created_at: string;
}

export type PokeballStatus = 'idle' | 'working' | 'collaborating' | 'finished' | 'error';

export interface SwarmManifest {
	orchestrator_instructions: string;
	agent_roles: AgentRoleBlueprint[];
	execution_strategy: string;
}

export interface AgentRoleBlueprint {
	name: string;
	role: string;
	task_description: string;
	model_hint: string | null;
	collaborates_with: string[];
}

export type AgentOutputType = 'thinking' | 'deliverable' | 'status_update' | 'question';

// WebSocket event types (matching Rust SwarmEvent enum)
export type SwarmEvent =
	| SwarmCreatedEvent
	| ManifestGeneratedEvent
	| AgentSpawnedEvent
	| AgentStatusChangedEvent
	| AgentOutputEvent
	| AgentMessageEvent
	| PhaseChangedEvent
	| SwarmCompletedEvent
	| SwarmErrorEvent
	| ConnectedEvent;

export interface ConnectedEvent {
	type: 'connected';
	data: { message: string };
}

export interface SwarmCreatedEvent {
	type: 'SwarmCreated';
	data: {
		swarm_id: string;
		goal: string;
		timestamp: string;
	};
}

export interface ManifestGeneratedEvent {
	type: 'ManifestGenerated';
	data: {
		swarm_id: string;
		orchestrator_instructions: string;
		agent_count: number;
		timestamp: string;
	};
}

export interface AgentSpawnedEvent {
	type: 'AgentSpawned';
	data: {
		swarm_id: string;
		agent_id: string;
		name: string;
		role: string;
		persona_category: string | null;
		model: string;
		timestamp: string;
	};
}

export interface AgentStatusChangedEvent {
	type: 'AgentStatusChanged';
	data: {
		swarm_id: string;
		agent_id: string;
		old_status: string;
		new_status: string;
		timestamp: string;
	};
}

export interface AgentOutputEvent {
	type: 'AgentOutput';
	data: {
		swarm_id: string;
		agent_id: string;
		agent_name: string;
		content: string;
		output_type: AgentOutputType;
		timestamp: string;
	};
}

export interface AgentMessageEvent {
	type: 'AgentMessage';
	data: {
		swarm_id: string;
		from_agent_id: string;
		to_agent_id: string;
		from_name: string;
		to_name: string;
		message: string;
		timestamp: string;
	};
}

export interface PhaseChangedEvent {
	type: 'PhaseChanged';
	data: {
		swarm_id: string;
		old_phase: string;
		new_phase: string;
		timestamp: string;
	};
}

export interface SwarmCompletedEvent {
	type: 'SwarmCompleted';
	data: {
		swarm_id: string;
		summary: string;
		total_messages: number;
		duration_secs: number;
		timestamp: string;
	};
}

export interface SwarmErrorEvent {
	type: 'SwarmError';
	data: {
		swarm_id: string;
		error: string;
		timestamp: string;
	};
}

// Graph visualization types
export interface GraphNode {
	id: string;
	name: string;
	role: string;
	status: PokeballStatus;
	category: string | null;
	x?: number;
	y?: number;
	fx?: number | null;
	fy?: number | null;
}

export interface GraphLink {
	source: string | GraphNode;
	target: string | GraphNode;
	active?: boolean;
}

// Chat message for the feed
export interface ChatMessage {
	id: string;
	type: 'output' | 'message' | 'system';
	from_name: string;
	to_name?: string;
	content: string;
	output_type?: AgentOutputType;
	timestamp: string;
}
