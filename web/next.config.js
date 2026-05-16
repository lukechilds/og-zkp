const VERCEL_CDN_CACHE = 's-maxage=1, stale-while-revalidate=604800';

const nextConfig = {
  async headers() {
    const cacheHeader = {
      key: 'Vercel-CDN-Cache-Control',
      value: VERCEL_CDN_CACHE,
    };

    return [
      {
        source: '/',
        headers: [cacheHeader],
      },
      {
        source: '/page/:path*',
        headers: [cacheHeader],
      },
      {
        source: '/search/:path*',
        headers: [cacheHeader],
      },
      {
        source: '/proof/:id',
        headers: [cacheHeader],
      },
      {
        source: '/api/proofs',
        headers: [cacheHeader],
      },
      {
        source: '/api/proof/:id',
        headers: [cacheHeader],
      },
    ];
  },
  outputFileTracingIncludes: {
    '/api/submit': ['./bin/**'],
  },
  serverExternalPackages: ['@libsql/client', 'ws'],
};
module.exports = nextConfig;
