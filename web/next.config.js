const nextConfig = {
  outputFileTracingIncludes: {
    '/api/submit': ['./bin/**'],
  },
  serverExternalPackages: ['@libsql/client', 'ws'],
};
module.exports = nextConfig;
