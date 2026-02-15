FROM risczero/risc0-groth16-prover:v2025-04-03.1 AS groth16

FROM debian:trixie-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY og-zkp /usr/local/bin/og-zkp
COPY --from=groth16 /usr/local/bin/stark_verify /usr/local/bin/stark_verify
COPY --from=groth16 /usr/local/bin/prover /usr/local/bin/prover
COPY --from=groth16 /app/stark_verify.cs /app/stark_verify.cs
COPY --from=groth16 /app/stark_verify.dat /app/stark_verify.dat
COPY --from=groth16 /app/stark_verify_final.pk.dmp /app/stark_verify_final.pk.dmp
COPY --from=groth16 /app/prover.sh /app/prover.sh
COPY groth16-shim.sh /usr/local/bin/docker
RUN chmod +x /usr/local/bin/docker
ENTRYPOINT ["og-zkp"]
