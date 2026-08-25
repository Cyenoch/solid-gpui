declare module "react-reconciler" {
  interface ReconcilerRoot {
    readonly current: unknown;
  }

  interface HostConfig<TContainer, TInstance, TProps, TContext> {
    rendererVersion: string;
    rendererPackageName: string;
    isPrimaryRenderer: boolean;
    supportsMutation: boolean;
    supportsPersistence: boolean;
    supportsHydration: boolean;
    supportsMicrotasks: boolean;
    scheduleMicrotask(callback: () => void): void;
    noTimeout: number;
    now(): number;
    getCurrentUpdatePriority(): number;
    setCurrentUpdatePriority(priority: number): void;
    resolveUpdatePriority(): number;
    resolveEventTimeStamp(): number;
    resolveEventType(): unknown;
    trackSchedulerEvent(): void;
    getPublicInstance(instance: TInstance): TInstance;
    getRootHostContext(container: TContainer): TContext;
    getChildHostContext(context: TContext, type: string): TContext;
    prepareForCommit(container: TContainer): null | object;
    resetAfterCommit(container: TContainer): void;
    createInstance(type: string, props: TProps, container: TContainer, context: TContext): TInstance;
    createTextInstance(text: string, container: TContainer, context: TContext): TInstance;
    appendInitialChild(parent: TInstance, child: TInstance): void;
    finalizeInitialChildren(
      instance: TInstance,
      type: string,
      props: TProps,
      container: TContainer,
      context: TContext,
    ): boolean;
    shouldSetTextContent(type: string, props: TProps): boolean;
    commitUpdate(instance: TInstance, type: string, oldProps: TProps, newProps: TProps, internalHandle?: unknown): void;
    commitTextUpdate(instance: TInstance, oldText: string, newText: string): void;
    appendChild(parent: TInstance, child: TInstance): void;
    appendChildToContainer(container: TContainer, child: TInstance): void;
    insertBefore(parent: TInstance, child: TInstance, before: TInstance): void;
    insertInContainerBefore(container: TContainer, child: TInstance, before: TInstance): void;
    removeChild(parent: TInstance, child: TInstance): void;
    removeChildFromContainer(container: TContainer, child: TInstance): void;
    clearContainer(container: TContainer): void;
    detachDeletedInstance(instance: TInstance): void;
    preparePortalMount(container: TContainer): void;
    scheduleTimeout: typeof setTimeout;
    cancelTimeout: typeof clearTimeout;
    hideInstance(instance: TInstance): void;
    hideTextInstance(instance: TInstance): void;
    unhideInstance(instance: TInstance, props: TProps): void;
    unhideTextInstance(instance: TInstance, text: string): void;
  }

  interface ReconcilerRenderer<TContainer> {
    createContainer(
      containerInfo: TContainer,
      tag: number,
      hydrationCallbacks: null,
      isStrictMode: boolean,
      concurrentUpdatesByDefaultOverride: null,
      identifierPrefix: string,
      onUncaughtError: (error: unknown) => void,
      onCaughtError: (error: unknown) => void,
      onRecoverableError: (error: unknown) => void,
      onDefaultTransitionIndicator: null,
    ): ReconcilerRoot;
    updateContainerSync(element: unknown, container: ReconcilerRoot, parentComponent: unknown, callback: null): number;
    batchedUpdates<T>(fn: () => T): T;
    flushSyncFromReconciler<T>(fn: () => T): T;
  }

  const Reconciler: <TContainer, TInstance, TProps, TContext>(
    hostConfig: HostConfig<TContainer, TInstance, TProps, TContext>,
  ) => ReconcilerRenderer<TContainer>;
  export default Reconciler;
}

declare module "react-reconciler/constants" {
  export const DefaultEventPriority: number;
  export const LegacyRoot: number;
}
