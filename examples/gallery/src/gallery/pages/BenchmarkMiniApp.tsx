import { Text, View } from "@solid-gpui/core";
import { batch, createSignal, onCleanup } from "@solid-gpui/core/runtime";
import type { SolidChild } from "../types";
import { contrastingText } from "../theme";
import { useGallery } from "../context";
import { Button, Card, DenseRow, ResponsiveRow, SectionHeader } from "../components/ui";

type BenchmarkTimer = ReturnType<typeof setInterval>;

export function BenchmarkMiniApp(): SolidChild {
  const { theme, showStatus } = useGallery();

  const CELL_COUNT = 100;
  // Initialize signals for each cell
  const cellSignals = Array.from({ length: CELL_COUNT }, (_, idx) => {
    const [val, setVal] = createSignal(0);
    return { id: idx, get: val, set: setVal };
  });

  const [isRunning, setIsRunning] = createSignal(false);
  const [totalTicks, setTotalTicks] = createSignal(0);
  const [useBatching, setUseBatching] = createSignal(true);
  const [intervalMs, setIntervalMs] = createSignal(50);
  const [lastBatchDurationMs, setLastBatchDurationMs] = createSignal(0);

  let timer: BenchmarkTimer | null = null;

  const stopBenchmark = () => {
    if (timer !== null) {
      clearInterval(timer);
      timer = null;
    }
    setIsRunning(false);
  };

  const runTick = () => {
    const start = performance.now();
    const batchMode = useBatching();

    if (batchMode) {
      batch(() => {
        for (let i = 0; i < CELL_COUNT; i++) {
          cellSignals[i]!.set((prev) => (prev + 1) % 100);
        }
      });
    } else {
      for (let i = 0; i < CELL_COUNT; i++) {
        cellSignals[i]!.set((prev) => (prev + 1) % 100);
      }
    }

    const elapsed = performance.now() - start;
    setLastBatchDurationMs(Number(elapsed.toFixed(2)));
    setTotalTicks((t) => t + 1);
  };

  const startBenchmark = () => {
    stopBenchmark();
    setIsRunning(true);
    timer = setInterval(runTick, intervalMs());
    showStatus(`Benchmark started (${CELL_COUNT} cells @ ${intervalMs()}ms)`);
  };

  const stepOnce = () => {
    runTick();
    showStatus("Step tick completed");
  };

  const resetAll = () => {
    stopBenchmark();
    batch(() => {
      for (let i = 0; i < CELL_COUNT; i++) {
        cellSignals[i]!.set(0);
      }
    });
    setTotalTicks(0);
    setLastBatchDurationMs(0);
    showStatus("Reset all benchmark cells");
  };

  onCleanup(() => {
    stopBenchmark();
  });

  return (
    <View style={{ gap: 24, minWidth: 0, flexShrink: 0 }}>
      <SectionHeader
        title="Reactive Performance Benchmark"
        description="Measure Solid GPUI signal updates and incremental protocol serialization across 100 cells."
        tag="Benchmark"
      />

      {/* Control Panel & Stats */}
      <Card
        title="Benchmark Controller"
        description="Compare batched updates vs unbatched dispatch across 100 reactive signals."
      >
        <View style={{ gap: 16 }}>
          {/* Controls Bar */}
          <View
            style={{
              backgroundColor: theme().bgActive,
              padding: 12,
              borderRadius: 8,
              gap: 12,
            }}
          >
            <DenseRow gap={12} style={{ justifyContent: "flex-start", alignItems: "center" }}>
              {/* Action Buttons */}
              <DenseRow gap={10} style={{ alignItems: "center" }}>
                <Button
                  variant={isRunning() ? "danger" : "primary"}
                  icon={isRunning() ? "lucide:pause" : "lucide:play"}
                  onPress={isRunning() ? stopBenchmark : startBenchmark}
                >
                  {isRunning() ? "Stop Benchmark" : "Start Benchmark"}
                </Button>
                <Button variant="secondary" disabled={isRunning()} onPress={stepOnce}>
                  Step Single Tick
                </Button>
                <Button variant="outline" onPress={resetAll}>
                  Reset All
                </Button>
              </DenseRow>

              {/* Batching Toggle */}
              <View style={{ flexDirection: "row", gap: 8, alignItems: "center" }}>
                <Text style={{ color: theme().textSecondary, fontSize: 13, fontWeight: "medium" }}>Batch Mode:</Text>
                <Button
                  variant={useBatching() ? "success" : "ghost"}
                  size="sm"
                  icon={useBatching() ? "lucide:check" : undefined}
                  onPress={() => setUseBatching(!useBatching())}
                >
                  {useBatching() ? "batch() Enabled" : "Unbatched"}
                </Button>
              </View>
            </DenseRow>

            {/* Frequency Interval */}
            <DenseRow gap={12} style={{ alignItems: "center" }}>
              <Text style={{ color: theme().textSecondary, fontSize: 13, fontWeight: "medium" }}>
                Update Frequency:
              </Text>
              {([16, 33, 50, 100, 250] as const).map((ms) => (
                <Button
                  variant={intervalMs() === ms ? "primary" : "ghost"}
                  size="sm"
                  onPress={() => {
                    setIntervalMs(ms);
                    if (isRunning()) startBenchmark();
                  }}
                >
                  {`${ms}ms (~${Math.round(1000 / ms)}Hz)`}
                </Button>
              ))}
            </DenseRow>
          </View>

          {/* Live Metrics Bar */}
          <ResponsiveRow gap={12}>
            {[
              { label: "Total Ticks", value: () => totalTicks().toLocaleString() },
              { label: "Total Cell Updates", value: () => (totalTicks() * CELL_COUNT).toLocaleString() },
              { label: "Dispatch Latency", value: () => `${lastBatchDurationMs()} ms` },
              { label: "Status", value: () => (isRunning() ? "Running" : "Idle") },
            ].map((m) => (
              <View
                style={{
                  flexGrow: 1,
                  minWidth: 0,
                  flexShrink: 1,
                  backgroundColor: theme().bgMuted,
                  borderRadius: 6,
                  padding: 12,
                  borderWidth: 1,
                  borderColor: theme().border,
                  gap: 4,
                }}
              >
                <Text style={{ color: theme().textMuted, fontSize: 11, fontWeight: "semibold", minWidth: 0 }}>
                  {m.label.toUpperCase()}
                </Text>
                <Text style={{ color: theme().textPrimary, fontSize: 18, fontWeight: "bold", minWidth: 0 }}>
                  {m.value}
                </Text>
              </View>
            ))}
          </ResponsiveRow>

          {/* 100-Cell Reactive Visual Grid */}
          <DenseRow
            gap={6}
            style={{
              backgroundColor: theme().bgMuted,
              borderRadius: 8,
              borderWidth: 1,
              borderColor: theme().border,
              padding: 16,
              justifyContent: "flex-start" as const,
            }}
          >
            {cellSignals.map((cell) => {
              const color = () => {
                const val = cell.get();
                if (val > 75) return "#3B82F6";
                if (val > 50) return "#10B981";
                if (val > 25) return "#8B5CF6";
                if (val > 0) return "#F59E0B";
                return theme().bgActive;
              };

              return (
                <View
                  style={{
                    width: 60,
                    height: 38,
                    flexShrink: 0,
                    backgroundColor: color(),
                    borderRadius: 4,
                    alignItems: "center" as const,
                    justifyContent: "center" as const,
                  }}
                >
                  <Text style={{ color: contrastingText(color()), fontSize: 12, fontWeight: "bold" }}>
                    {() => String(cell.get())}
                  </Text>
                </View>
              );
            })}
          </DenseRow>
        </View>
      </Card>
    </View>
  );
}
