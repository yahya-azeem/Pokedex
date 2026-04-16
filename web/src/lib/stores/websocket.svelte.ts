import type { SwarmEvent } from '$lib/types';

type EventHandler = (event: SwarmEvent) => void;

export class WebSocketManager {
	private ws: WebSocket | null = null;
	private handlers: EventHandler[] = [];
	private reconnectAttempts = 0;
	private maxReconnectAttempts = 10;
	private reconnectDelay = 1000;
	private _connected = $state(false);

	get connected() {
		return this._connected;
	}

	connect(url?: string) {
		const wsUrl = url || `ws://${window.location.host}/ws`;

		try {
			this.ws = new WebSocket(wsUrl);

			this.ws.onopen = () => {
				this._connected = true;
				this.reconnectAttempts = 0;
				console.log('[WS] Connected to Pokedex Swarm');
			};

			this.ws.onmessage = (event) => {
				try {
					const data: SwarmEvent = JSON.parse(event.data);
					this.handlers.forEach((handler) => handler(data));
				} catch (e) {
					console.warn('[WS] Failed to parse message:', e);
				}
			};

			this.ws.onclose = () => {
				this._connected = false;
				console.log('[WS] Disconnected');
				this.attemptReconnect(wsUrl);
			};

			this.ws.onerror = (error) => {
				console.error('[WS] Error:', error);
			};
		} catch (e) {
			console.error('[WS] Connection failed:', e);
			this.attemptReconnect(wsUrl);
		}
	}

	private attemptReconnect(url: string) {
		if (this.reconnectAttempts < this.maxReconnectAttempts) {
			this.reconnectAttempts++;
			const delay = this.reconnectDelay * Math.min(this.reconnectAttempts, 5);
			console.log(`[WS] Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts})`);
			setTimeout(() => this.connect(url), delay);
		}
	}

	onEvent(handler: EventHandler) {
		this.handlers.push(handler);
		return () => {
			this.handlers = this.handlers.filter((h) => h !== handler);
		};
	}

	disconnect() {
		if (this.ws) {
			this.ws.close();
			this.ws = null;
		}
		this._connected = false;
	}

	send(data: unknown) {
		if (this.ws && this.ws.readyState === WebSocket.OPEN) {
			this.ws.send(JSON.stringify(data));
		}
	}
}
