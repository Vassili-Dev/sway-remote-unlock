# Sway Remote Unlock Design Doc

## Goals

- Allow waking the screen from a remote signal
- Allow unlocking Swaylock from a remote signal
- Verify authenticity of remote signal (Fingerprint-generated Cryptographic key)
- Run a deamonized server as a systemd service to accept remote signals
- (Optional) Modularize unlock to allow portability to other systems

## Non-goals

- Writing a remote fingerprint driver

## Assumptions

- The daemon will run on startup as a non-root user
- The remote agent will be connected to the same network as the daemon
