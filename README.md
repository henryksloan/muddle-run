# muddle-run
A home for experiments for muddle.run

## Building the client

```bash
cd bins/web_client
yarn install
yarn build # or `yarn dev:start` for hot-reload
```

Building wasm module separately:
```bash
RUSTFLAGS=--cfg=web_sys_unstable_apis cargo build -p mr_web_client --target wasm32-unknown-unknown
```
