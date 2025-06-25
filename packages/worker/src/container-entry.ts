import { Container, getContainer } from '@cloudflare/containers';

export class MainContainer extends Container {
  defaultPort = 3000; // Port the container is listening on
  sleepAfter = "10m"; // Stop the instance if requests not sent for 10 minutes
}

export default {
  async fetch(request: Request, env: any, ctx: any) {
    const container = getContainer(env.main);
    return container.fetch(request);
  },
};