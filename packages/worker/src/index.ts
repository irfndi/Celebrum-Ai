import { Hono } from 'hono';

import type { Env } from '@celebrum-ai/shared';

import { router as apiRouter } from './services/router';

const app = new Hono<{ Bindings: Env }>();

app.route('/api/v1', apiRouter);

app.get('*', (c) => {
  return c.text('Not Found', 404);
});

export default {
  fetch: app.fetch,
};