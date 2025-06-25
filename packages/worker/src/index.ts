import { Hono } from 'hono';
import { serve } from '@hono/node-server';

import type { Env } from '@celebrum-ai/shared';

import { router as apiRouter } from './services/router';

const app = new Hono<{ Bindings: Env }>();

app.route('/api/v1', apiRouter);

app.get('*', (c) => {
  return c.text('Not Found', 404);
});

// Check if running in Node.js environment for container
if (typeof process !== 'undefined' && process.versions && process.versions.node) {
  const port = process.env.PORT ? parseInt(process.env.PORT, 10) : 8787;
  console.log(`Server is running on port ${port}`);
  serve({
    fetch: app.fetch,
    port,
  });
}

export default {
  fetch: app.fetch,
};