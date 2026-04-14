# Development & Deployment

## Target Hardware

- **Device:** Raspberry Pi 4
- **OS:** Linux (Raspberry Pi OS / Debian-based)
- **Architecture:** `aarch64-unknown-linux-gnu`

---

## Development Cycle

Three phases with increasing fidelity. Move to the next phase only when the current one passes.

### Phase 1 — Loopback on Windows (fastest)

Run both roles on the same machine using `localhost`. No Pi, no compilation delay. Use this for logic and functional iteration.

### Phase 2 — Local cross-compile + SCP to Pi (hardware iteration)

When you need to test on real hardware without going through CI. Cross-compile in WSL2 and copy the binary directly to the Pi over SSH. Use this for tight iteration on hardware-specific behavior. Copy results (CSV and log file) back to the dev machine for analysis.

### Phase 3 — CI artifact + manual Pi deployment (authoritative)

On every push, GitHub Actions cross-compiles the binary and uploads it as a build artifact. The artifact is then transferred to the Pi by any convenient means and used to run the official test scenarios.

**The CI artifact is the authoritative binary for all official measurements.** Do not use Phase 2 binaries for final results.

CI pipeline:

```
push
 └─ job: build          (GitHub hosted runner, ubuntu-latest)
      checkout → cross-compile → upload artifact
```

After the build completes:
1. Download the artifact from the GitHub Actions run and transfer it to the Pi
2. Run the test scenarios manually on the Pi
3. Transfer the results (CSV and log file) back to the dev machine for analysis

---

## Cross-Compilation (WSL2)

Cross-compilation is done inside WSL2 using the `aarch64-unknown-linux-gnu` target and a GCC cross-linker. The linker is configured in `.cargo/config.toml`.

---

## GitHub Actions

The CI pipeline consists of a single build job running on a GitHub runner:

1. Check out the repository
2. Install the Rust `aarch64-unknown-linux-gnu` target and the GCC cross-linker
3. Cross-compile the binary in release mode
4. Upload the binary as a build artifact
