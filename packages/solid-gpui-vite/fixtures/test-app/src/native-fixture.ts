// Application-owned descriptor fixture for the Vite type-resolution test.
import { createNativeClient, type NativeClientDescriptor, type NativeInvoker } from "@solid-gpui/core/native";

export type CounterProps = {
  count?: number;
  label?: string | null;
};

export interface CounterClient {
  increment(amount: number): Promise<number>;
}

export const counterDescriptor: NativeClientDescriptor = {
  buildDigest: Array(32).fill(11),
  semanticVersion: "1.0.0",
  moduleId: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
  moduleDigest: Array.from({ length: 32 }, (_, index) => index),
  commands: [{ id: 1, name: "increment" }],
};

export function createCounterClient(invoker: NativeInvoker): CounterClient {
  return createNativeClient<CounterClient>(invoker, counterDescriptor);
}
