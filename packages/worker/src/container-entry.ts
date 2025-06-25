import { getContainer } from '@cloudflare/containers';

export default {
  async fetch(request: Request, env: any, ctx: any) {
    const container = getContainer(env.main);
    return container.fetch(request);
  },
};