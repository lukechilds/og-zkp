const crypto = require("crypto");

function proofId(proof) {
  return crypto.createHash("sha256").update(proof).digest("hex").slice(0, 32);
}

module.exports = { proofId };
