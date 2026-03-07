# Roadmap

## Collector

- [ ] Improve accuracy of total power usage on several devices by adding more sensors and refining estimation algorithms
- [ ] Add tests
- [ ] Run as a service

### Security

- [ ] Remove WinRing0 driver dependency on Windows (see [Security](https://github.com/daminoup88/wattseal/blob/main/SECURITY.md#winring0-kernel-driver-windows) section for details)

## UI / UX

- [ ] Top process in tooltip for each component and in the total chart
- [ ] Select each component in the total chart
- [ ] Notification thresholds — total and per process ([#12](https://github.com/Daminoup88/WattSeal/issues/12))
- [ ] Differentiate apps and background processes

## Network & emissions

- [ ] Indirect network power usage and emissions calculation
- [ ] Indirect network power usage by domain
- [ ] Power usage breakdown by browser tab
- [ ] Auto-update electricity prices and carbon emissions on build (open data)
