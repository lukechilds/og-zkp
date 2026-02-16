const nextConfig = {
  outputFileTracingIncludes: {
    '/api/submit': ['./bin/**'],
  },
  serverExternalPackages: ['@libsql/client'],
};
module.exports = nextConfig;
