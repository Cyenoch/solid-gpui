declare module "@babel/core" {
  export interface TransformResult {
    readonly code?: string;
  }
  export function transformAsync(source: string, options: Record<string, unknown>): Promise<TransformResult | null>;
}

declare module "react-refresh/babel" {
  const plugin: unknown;
  export default plugin;
}

declare module "react-refresh/runtime" {
  const runtime: {
    injectIntoGlobalHook(global: typeof globalThis): void;
    register(type: unknown, id: string): void;
    createSignatureFunctionForTransform(...args: unknown[]): (...args: unknown[]) => unknown;
    performReactRefresh(): unknown;
  };
  export default runtime;
}

declare var $RefreshReg$: (type: unknown, id: string) => void;
declare var $RefreshSig$: (...args: unknown[]) => (...args: unknown[]) => unknown;

declare module "@babel/plugin-transform-react-jsx" {
  const plugin: unknown;
  export default plugin;
}


