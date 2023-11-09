# WASI Sidecar for AUTHZ<OpenID> and BucketRateLimits<Redis>

## Notes
* `integrations` dir contains the Apollo-Router plugin/binary that talks to the WASM sidecar.
* `wasi_crates` WIP conversions for wasi-sidecar

## Steps
Build Dockerfiles for router and sidecar:
* `cd integrations/apollo-router-plugin && docker build -t upwork-auth:router .`
* `cd wasm-sidecar && docker build --target server-standalone -t upwork-auth:sidecar .`

Run deployment:
* `kubectl apply -f Deployment.yaml`
