/**
 * Worker mínimo: sirve el sitio estático generado por Vite (ASSETS).
 * Compatible con Cloudflare Workers + binding [assets].
 */
export default {
  async fetch(request: Request, env: { ASSETS: { fetch: typeof fetch } }): Promise<Response> {
    return env.ASSETS.fetch(request);
  },
};
