# Power Monitor Collector

The backend component responsible for gathering hardware power consumption metrics.

## Data Architecture

The collector gathers the following events:

- **Power**:
  - Intel RAPL (PKG, PP0, PP1, DRAM)
  - AMD RAPL
  - NVSMI (NVIDIA GPUs)
  - RAM (Estimation)
  - Disks/Peripherals (Estimation)
- **Usage**:
  - CPU
  - GPU
  - RAM

## Requirements

- **Windows**: Requires Administrator privileges to access hardware counters via the WinRing0 driver.

## Usage

To run only the collector:

```bash
cargo run -p collector
```

## Troubleshooting

### Windows Driver Issues

If the WinRing0 driver fails to stop correctly, you can manually stop the service:

```cmd
sc stop WinRing0_1_2_0
```
