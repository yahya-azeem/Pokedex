import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	server: {
		proxy: {
			'/api': {
				target: 'http://localhost:5001',
				changeOrigin: true,
			},
			'/ws': {
				target: 'ws://localhost:5001',
				ws: true,
			},
		},
	},
});
