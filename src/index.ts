import { Container, getContainer, loadBalance } from 'cf-containers';

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
	async fetch(request: Request): Promise<Response> {
		return await this.containerFetch(request);
	}
}

export default {
	async fetch(request: Request, env): Promise<Response> {
		const url = new URL(request.url);
		const isLocal = url.protocol === 'http:' ? true : false;
		// const pathname = url.pathname;
		// const jobID = url.searchParams.get('id');

		if (isLocal) return await fetch('http://localhost:8080/convert', request);
		let container = await loadBalance(env.WIFSKI_CONTAINER, 3);
		return await container.fetch(request);
	},
} satisfies ExportedHandler<Env>;
