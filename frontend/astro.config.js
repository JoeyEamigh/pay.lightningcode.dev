import solid from '@astrojs/solid-js';
import tailwind from '@astrojs/tailwind';
import { defineConfig } from 'astro/config';

export default defineConfig({
  integrations: [tailwind({ nesting: true }), solid()],
  server: { port: 3000 },
  prefetch: true,
  prefetch: { prefetchAll: true },
});
