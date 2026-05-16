const { execFile } = require("child_process");
const path = require("path");

function verify(proof) {
  return new Promise((resolve, reject) => {
    const bin = path.join(process.cwd(), "bin", "og-zkp-verifier");
    execFile(bin, [proof, "--json"], { timeout: 30000 }, (error, stdout) => {
      if (error) {
        try {
          const result = JSON.parse(stdout);
          reject(new Error(result.error || "Verification failed"));
        } catch {
          reject(new Error("Verification failed"));
        }
        return;
      }
      try {
        resolve(JSON.parse(stdout));
      } catch {
        reject(new Error("Failed to parse verifier output"));
      }
    });
  });
}

module.exports = { verify };
