import { Container, loadBalance } from '@cloudflare/containers';

export class WifskiContainer extends Container {
	defaultPort = 8080;
	sleepAfter = '5m';

	override onStart() {
		console.log('Container successfully started');
	}
	override onStop() {
		console.log('Container successfully shut down');
	}
	override onError(error: unknown) {
		console.log('Container error:', error);
	}
}

export default {
	async fetch(request: Request, env): Promise<Response> {
		let container = await loadBalance(env.WIFSKI_CONTAINER, 3);
		return await container.fetch(request);
	},
} satisfies ExportedHandler<Env>;
