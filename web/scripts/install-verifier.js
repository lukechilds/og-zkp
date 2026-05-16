const fs = require('fs');
const https = require('https');
const path = require('path');

const repo = process.env.OG_ZKP_REPO || 'lukechilds/og-zkp';
const asset = process.env.OG_ZKP_VERIFIER_ASSET || 'og-zkp-verifier-x86_64-unknown-linux-gnu';
const root = path.join(__dirname, '..');
const versionFile = path.join(root, 'VERIFIER_VERSION');
const binDir = path.join(root, 'bin');
const binPath = path.join(binDir, 'og-zkp-verifier');

function verifierVersion() {
  if (process.env.OG_ZKP_VERIFIER_VERSION) {
    return process.env.OG_ZKP_VERIFIER_VERSION;
  }
  return fs.readFileSync(versionFile, 'utf8').trim();
}

function download(url, destination, redirects = 0) {
  if (redirects > 5) {
    throw new Error('Too many redirects while downloading verifier');
  }

  return new Promise((resolve, reject) => {
    const request = https.get(
      url,
      { headers: { 'user-agent': 'og-zkp-web-build' } },
      (response) => {
        if ([301, 302, 303, 307, 308].includes(response.statusCode)) {
          response.resume();
          const location = response.headers.location;
          if (!location) {
            reject(new Error('Verifier download redirect did not include a location'));
            return;
          }
          download(new URL(location, url).toString(), destination, redirects + 1)
            .then(resolve, reject);
          return;
        }

        if (response.statusCode !== 200) {
          response.resume();
          reject(new Error(`Verifier download failed with HTTP ${response.statusCode}`));
          return;
        }

        const file = fs.createWriteStream(destination, { mode: 0o755 });
        response.pipe(file);
        file.on('finish', () => {
          file.close(resolve);
        });
        file.on('error', reject);
      }
    );
    request.on('error', reject);
  });
}

async function main() {
  if (
    process.env.OG_ZKP_FORCE_VERIFIER_DOWNLOAD !== '1'
    && fs.existsSync(binPath)
    && fs.statSync(binPath).size > 0
  ) {
    console.log(`Using existing verifier at ${path.relative(root, binPath)}`);
    return;
  }

  const version = verifierVersion();
  const url = process.env.OG_ZKP_VERIFIER_URL
    || `https://github.com/${repo}/releases/download/${version}/${asset}`;

  fs.mkdirSync(binDir, { recursive: true });
  console.log(`Downloading ${asset} from ${version}`);
  await download(url, binPath);
  fs.chmodSync(binPath, 0o755);
  console.log(`Installed verifier at ${path.relative(root, binPath)}`);
}

main().catch((error) => {
  console.error(error.message);
  process.exit(1);
});
