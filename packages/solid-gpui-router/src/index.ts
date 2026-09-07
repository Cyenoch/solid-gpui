import {
  Pressable,
  Text,
  View,
  type PressHandler,
  type PressableProps,
  type SolidElement,
  type Style,
  type StyleProp,
} from "@solid-gpui/core";
import { createComponent } from "@solid-gpui/core/runtime";
import { createMemoryHistory, type RouterHistory } from "@tanstack/history";
import {
  BaseRootRoute,
  BaseRoute,
  RouterCore,
  isNotFound,
  isRedirect,
  notFound,
  redirect,
  replaceEqualDeep,
  type AnyContext,
  type AnyRoute,
  type AnyRouter,
  type ErrorComponentProps,
  type GetStoreConfig,
  type MakeRouteMatch,
  type MakeRouteMatchUnion,
  type NavigateOptions,
  type NotFoundRouteProps,
  type RegisteredRouter,
  type ResolveFullPath,
  type ResolveId,
  type ResolveParams,
  type RootRouteOptions,
  type RouteConstraints,
  type RouteOptions,
  type RouteMask,
  type RouterConstructorOptions,
  type RouterReadableStore,
  type RouterState,
  type RouterWritableStore,
  type StrictOrFrom,
  type ThrowConstraint,
  type ThrowOrOptional,
  type TrailingSlashOption,
} from "@tanstack/router-core";
import {
  batch,
  createContext,
  createEffect,
  createMemo,
  createRoot as createSolidRoot,
  createSignal,
  onCleanup,
  onMount,
  mergeProps,
  splitProps,
  startTransition,
  useContext,
  type Accessor,
} from "solid-js";

export { isNotFound, isRedirect, notFound, redirect };
export type {
  AnyContext,
  AnyRoute,
  AnyRouter,
  ErrorComponentProps,
  NotFoundRouteProps,
  RegisteredRouter,
  RouteOptions,
  RouterState,
  RouteMask,
} from "@tanstack/router-core";

export type RouteComponent = () => SolidElement;
export type ErrorRouteComponent = (props: ErrorComponentProps<unknown>) => SolidElement;
export type NotFoundRouteComponent = (props: NotFoundRouteProps) => SolidElement;

declare module "@tanstack/router-core" {
  interface UpdatableRouteOptionsExtensions {
    component?: RouteComponent;
    errorComponent?: ErrorRouteComponent;
    notFoundComponent?: NotFoundRouteComponent;
    pendingComponent?: RouteComponent;
  }

  interface RouterOptionsExtensions {
    defaultComponent?: RouteComponent;
    defaultErrorComponent?: ErrorRouteComponent;
    defaultNotFoundComponent?: NotFoundRouteComponent;
    defaultPendingComponent?: RouteComponent;
  }
}

const PENDING_STYLE: Style = { padding: 16 };
const ROUTER_ROOT_STYLE: Style = { flexDirection: "column", flexGrow: 1, minWidth: 0, minHeight: 0 };
const MESSAGE_STYLE: Style = { gap: 10, padding: 16 };
const TITLE_STYLE: Style = { fontSize: 16, fontWeight: "semibold" };
const RETRY_STYLE: Style = { padding: 8, borderWidth: 1, borderColor: "#808080", borderRadius: 6 };

export function DefaultPendingComponent(): SolidElement {
  return createComponent(View, {
    style: PENDING_STYLE,
    children: createComponent(Text, { children: "Loading…" }),
  });
}

export function DefaultErrorComponent(props: ErrorComponentProps<unknown>): SolidElement {
  const message = props.error instanceof Error ? props.error.message : String(props.error);
  return createComponent(View, {
    style: MESSAGE_STYLE,
    children: [
      createComponent(Text, { style: TITLE_STYLE, children: "Navigation failed" }),
      createComponent(Text, { children: message }),
      createComponent(Pressable, {
        accessibilityRole: "button",
        accessibilityLabel: "Retry navigation",
        style: RETRY_STYLE,
        onPress: () => props.reset(),
        children: createComponent(Text, { children: "Retry" }),
      }),
    ],
  });
}

export function DefaultNotFoundComponent(props: NotFoundRouteProps): SolidElement {
  return createComponent(View, {
    style: MESSAGE_STYLE,
    children: [
      createComponent(Text, { style: TITLE_STYLE, children: "Route not found" }),
      createComponent(Text, { children: () => String(props.routeId) }),
    ],
  });
}

export type NativeRoute<
  TRegister = unknown,
  TParentRoute extends AnyRoute = AnyRoute,
  TPath extends string = "/",
  TFullPath extends string = ResolveFullPath<TParentRoute, TPath>,
  TCustomId extends string = string,
  TId extends string = ResolveId<TParentRoute, TCustomId, TPath>,
  TSearchValidator = undefined,
  TParams = ResolveParams<TPath>,
  TRouteContextFn = AnyContext,
  TBeforeLoadFn = AnyContext,
  TLoaderDeps extends Record<string, any> = {},
  TLoaderFn = undefined,
  TChildren = unknown,
  TSSR = unknown,
> = BaseRoute<
  TRegister,
  TParentRoute,
  TPath,
  TFullPath,
  TCustomId,
  TId,
  TSearchValidator,
  TParams,
  AnyContext,
  TRouteContextFn,
  TBeforeLoadFn,
  TLoaderDeps,
  TLoaderFn,
  TChildren,
  unknown,
  TSSR
>;

export function createRoute<
  TRegister = unknown,
  TParentRoute extends RouteConstraints["TParentRoute"] = AnyRoute,
  TPath extends RouteConstraints["TPath"] = "/",
  TFullPath extends RouteConstraints["TFullPath"] = ResolveFullPath<TParentRoute, TPath>,
  TCustomId extends RouteConstraints["TCustomId"] = string,
  TId extends RouteConstraints["TId"] = ResolveId<TParentRoute, TCustomId, TPath>,
  TSearchValidator = undefined,
  TParams = ResolveParams<TPath>,
  TRouteContextFn = AnyContext,
  TBeforeLoadFn = AnyContext,
  TLoaderDeps extends Record<string, any> = {},
  TLoaderFn = undefined,
  TChildren = unknown,
  TSSR = unknown,
>(
  options: RouteOptions<
    TRegister,
    TParentRoute,
    TId,
    TCustomId,
    TFullPath,
    TPath,
    TSearchValidator,
    TParams,
    TLoaderDeps,
    TLoaderFn,
    AnyContext,
    TRouteContextFn,
    TBeforeLoadFn,
    TSSR
  >,
): NativeRoute<
  TRegister,
  TParentRoute,
  TPath,
  TFullPath,
  TCustomId,
  TId,
  TSearchValidator,
  TParams,
  TRouteContextFn,
  TBeforeLoadFn,
  TLoaderDeps,
  TLoaderFn,
  TChildren,
  TSSR
> {
  return new BaseRoute(options) as NativeRoute<
    TRegister,
    TParentRoute,
    TPath,
    TFullPath,
    TCustomId,
    TId,
    TSearchValidator,
    TParams,
    TRouteContextFn,
    TBeforeLoadFn,
    TLoaderDeps,
    TLoaderFn,
    TChildren,
    TSSR
  >;
}

export function createRootRoute<
  TRegister = unknown,
  TSearchValidator = undefined,
  TRouterContext = {},
  TRouteContextFn = AnyContext,
  TBeforeLoadFn = AnyContext,
  TLoaderDeps extends Record<string, any> = {},
  TLoaderFn = undefined,
  TSSR = unknown,
>(
  options?: RootRouteOptions<
    TRegister,
    TSearchValidator,
    TRouterContext,
    TRouteContextFn,
    TBeforeLoadFn,
    TLoaderDeps,
    TLoaderFn,
    TSSR
  >,
): BaseRootRoute<
  TRegister,
  TSearchValidator,
  TRouterContext,
  TRouteContextFn,
  TBeforeLoadFn,
  TLoaderDeps,
  TLoaderFn,
  unknown,
  unknown,
  TSSR
> {
  return new BaseRootRoute(options);
}

export function createRootRouteWithContext<TRouterContext extends object>() {
  return <
    TRegister = unknown,
    TSearchValidator = undefined,
    TRouteContextFn = AnyContext,
    TBeforeLoadFn = AnyContext,
    TLoaderDeps extends Record<string, any> = {},
    TLoaderFn = undefined,
    TSSR = unknown,
  >(
    options?: RootRouteOptions<
      TRegister,
      TSearchValidator,
      TRouterContext,
      TRouteContextFn,
      TBeforeLoadFn,
      TLoaderDeps,
      TLoaderFn,
      TSSR
    >,
  ) => new BaseRootRoute(options);
}

export function createRouteMask<TRouteTree extends AnyRoute>(options: RouteMask<TRouteTree>): RouteMask<TRouteTree> {
  return options;
}

function createNativeMutableStore<TValue>(initialValue: TValue): RouterWritableStore<TValue> {
  const [get, set] = createSignal(initialValue);
  return { get, set };
}

function createNativeReadonlyStore<TValue>(read: () => TValue): RouterReadableStore<TValue> {
  return { get: createSolidRoot(() => createMemo(read)) };
}

const nativeStoreFactory: GetStoreConfig = () => ({
  createMutableStore: createNativeMutableStore,
  createReadonlyStore: createNativeReadonlyStore,
  batch,
});

declare const nativeRouterBrand: unique symbol;
export type NativeRouter<TRouter extends AnyRouter = AnyRouter> = TRouter & {
  readonly [nativeRouterBrand]: true;
};

const nativeRouters = new WeakSet<object>();
const mountedRouters = new WeakSet<object>();

type NativeOnlyOption =
  | "defaultHashScrollIntoView"
  | "defaultViewTransition"
  | "getScrollRestorationKey"
  | "history"
  | "isServer"
  | "origin"
  | "scrollRestoration"
  | "scrollRestorationBehavior"
  | "scrollToTopSelectors";

export type NativeRouterOptions<
  TRouteTree extends AnyRoute,
  TTrailingSlashOption extends TrailingSlashOption = "never",
  TDefaultStructuralSharingOption extends boolean = false,
  TDehydrated extends Record<string, any> = Record<string, any>,
> = Omit<
  RouterConstructorOptions<
    TRouteTree,
    TTrailingSlashOption,
    TDefaultStructuralSharingOption,
    RouterHistory,
    TDehydrated
  >,
  NativeOnlyOption | "defaultPreload"
> & {
  readonly initialEntries?: readonly string[];
};

export type NativeRouterInstance<
  TRouteTree extends AnyRoute,
  TTrailingSlashOption extends TrailingSlashOption = "never",
  TDefaultStructuralSharingOption extends boolean = false,
  TDehydrated extends Record<string, any> = Record<string, any>,
> = NativeRouter<
  RouterCore<TRouteTree, TTrailingSlashOption, TDefaultStructuralSharingOption, RouterHistory, TDehydrated>
>;

const disableNativeScrollRestoration = (): false => false;

export function createRouter<
  TRouteTree extends AnyRoute,
  TTrailingSlashOption extends TrailingSlashOption = "never",
  TDefaultStructuralSharingOption extends boolean = false,
  TDehydrated extends Record<string, any> = Record<string, any>,
>(
  options: NativeRouterOptions<TRouteTree, TTrailingSlashOption, TDefaultStructuralSharingOption, TDehydrated>,
): NativeRouterInstance<TRouteTree, TTrailingSlashOption, TDefaultStructuralSharingOption, TDehydrated> {
  const { initialEntries = ["/"], ...routerOptions } = options;
  if (initialEntries.length === 0) throw new TypeError("initialEntries must contain at least one route");

  const history = createMemoryHistory({ initialEntries: [...initialEntries] });
  const router = new RouterCore<
    TRouteTree,
    TTrailingSlashOption,
    TDefaultStructuralSharingOption,
    RouterHistory,
    TDehydrated
  >(
    {
      ...routerOptions,
      history,
      isServer: false,
      origin: "native://solid-gpui",
      scrollRestoration: false,
      defaultHashScrollIntoView: false,
      defaultViewTransition: false,
      defaultPreload: false,
      defaultPendingComponent: routerOptions.defaultPendingComponent ?? DefaultPendingComponent,
      defaultErrorComponent: routerOptions.defaultErrorComponent ?? DefaultErrorComponent,
      defaultNotFoundComponent: routerOptions.defaultNotFoundComponent ?? DefaultNotFoundComponent,
    } as RouterConstructorOptions<
      TRouteTree,
      TTrailingSlashOption,
      TDefaultStructuralSharingOption,
      RouterHistory,
      TDehydrated
    >,
    nativeStoreFactory,
  );
  const nativeOptions = {
    ...router.options,
    scrollRestoration: disableNativeScrollRestoration,
  } as RouterConstructorOptions<
    TRouteTree,
    TTrailingSlashOption,
    TDefaultStructuralSharingOption,
    RouterHistory,
    TDehydrated
  >;
  router.update(nativeOptions);
  nativeRouters.add(router);
  return router as NativeRouterInstance<TRouteTree, TTrailingSlashOption, TDefaultStructuralSharingOption, TDehydrated>;
}

const RouterContext = createContext<AnyRouter>();
const MatchContext = createContext<Accessor<MakeRouteMatchUnion<AnyRouter> | undefined>>();

type UniversalComponent<Props extends object> = (props: Props) => SolidElement;
const UniversalRouterContextProvider = RouterContext.Provider as unknown as UniversalComponent<{
  readonly value: AnyRouter;
  readonly children: SolidElement;
}>;
const UniversalMatchContextProvider = MatchContext.Provider as unknown as UniversalComponent<{
  readonly value: Accessor<MakeRouteMatchUnion<AnyRouter> | undefined>;
  readonly children: SolidElement;
}>;

export function useRouter<TRouter extends AnyRouter = RegisteredRouter>(): TRouter {
  const router = useContext(RouterContext);
  if (router === undefined) throw new Error("useRouter must be used under RouterProvider");
  return router as TRouter;
}

export interface UseRouterStateOptions<TRouter extends AnyRouter, TSelected> {
  readonly router?: TRouter;
  readonly select?: (state: RouterState<TRouter["routeTree"]>) => TSelected;
}

export type UseRouterStateResult<TRouter extends AnyRouter, TSelected> = unknown extends TSelected
  ? RouterState<TRouter["routeTree"]>
  : TSelected;

export function useRouterState<TRouter extends AnyRouter = RegisteredRouter, TSelected = unknown>(
  options?: UseRouterStateOptions<TRouter, TSelected>,
): Accessor<UseRouterStateResult<TRouter, TSelected>> {
  const router = options?.router ?? useRouter<TRouter>();
  if (options?.select === undefined) {
    return (() => router.stores.__store.get()) as Accessor<UseRouterStateResult<TRouter, TSelected>>;
  }
  const select = options.select;
  return createMemo((previous: TSelected | undefined) => {
    const selected = select(router.stores.__store.get());
    return previous === undefined ? selected : replaceEqualDeep(previous, selected);
  }) as Accessor<UseRouterStateResult<TRouter, TSelected>>;
}

export interface UseLocationOptions<TRouter extends AnyRouter, TSelected> {
  readonly select?: (location: RouterState<TRouter["routeTree"]>["location"]) => TSelected;
}

export type UseLocationResult<TRouter extends AnyRouter, TSelected> = unknown extends TSelected
  ? RouterState<TRouter["routeTree"]>["location"]
  : TSelected;

export function useLocation<TRouter extends AnyRouter = RegisteredRouter, TSelected = unknown>(
  options?: UseLocationOptions<TRouter, TSelected>,
): Accessor<UseLocationResult<TRouter, TSelected>> {
  const router = useRouter<TRouter>();
  return createMemo(() => {
    const location = router.stores.__store.get().location;
    return (options?.select ? options.select(location) : location) as UseLocationResult<TRouter, TSelected>;
  });
}

export interface UseMatchBaseOptions<TRouter extends AnyRouter, TFrom, TStrict extends boolean, TThrow, TSelected> {
  readonly select?: (match: MakeRouteMatch<TRouter["routeTree"], TFrom, TStrict>) => TSelected;
  readonly shouldThrow?: TThrow;
}

export type UseMatchOptions<
  TRouter extends AnyRouter,
  TFrom extends string | undefined,
  TStrict extends boolean,
  TThrow extends boolean,
  TSelected,
> = StrictOrFrom<TRouter, TFrom, TStrict> & UseMatchBaseOptions<TRouter, TFrom, TStrict, TThrow, TSelected>;

export type UseMatchResult<
  TRouter extends AnyRouter,
  TFrom,
  TStrict extends boolean,
  TSelected,
> = unknown extends TSelected
  ? TStrict extends true
    ? MakeRouteMatch<TRouter["routeTree"], TFrom, TStrict>
    : MakeRouteMatchUnion<TRouter>
  : TSelected;

function routeMatchFrom(options: object): string | undefined {
  return "from" in options && typeof options.from === "string" ? options.from : undefined;
}

export function useMatch<
  TRouter extends AnyRouter = RegisteredRouter,
  const TFrom extends string | undefined = undefined,
  TStrict extends boolean = true,
  TThrow extends boolean = true,
  TSelected = unknown,
>(
  options: UseMatchOptions<TRouter, TFrom, TStrict, ThrowConstraint<TStrict, TThrow>, TSelected>,
): Accessor<ThrowOrOptional<UseMatchResult<TRouter, TFrom, TStrict, TSelected>, TThrow>> {
  const router = useRouter<TRouter>();
  const nearestMatch = useContext(MatchContext);
  const from = routeMatchFrom(options);
  const routeId = from ?? nearestMatch?.()?.routeId;
  const match = (): MakeRouteMatchUnion<TRouter> | undefined =>
    routeId === undefined
      ? undefined
      : (router.stores.getMatchStore(routeId).get() as MakeRouteMatchUnion<TRouter> | undefined);
  createEffect(() => {
    if (match() === undefined && (options.shouldThrow ?? true)) {
      throw new Error(routeId === undefined ? "no active route match" : `no active route match for ${routeId}`);
    }
  });
  return createMemo(() => {
    const current = match();
    if (current === undefined) return undefined;
    return options.select ? options.select(current as never) : current;
  }) as Accessor<ThrowOrOptional<UseMatchResult<TRouter, TFrom, TStrict, TSelected>, TThrow>>;
}

export type NativeNavigateOptions<
  TRouter extends AnyRouter,
  TFrom extends string,
  TTo extends string | undefined,
  TMaskFrom extends string,
  TMaskTo extends string,
> = Omit<
  NavigateOptions<TRouter, TFrom, TTo, TMaskFrom, TMaskTo>,
  "hashScrollIntoView" | "href" | "reloadDocument" | "resetScroll" | "startTransition" | "viewTransition"
>;

export type NativeNavigateResult<TRouter extends AnyRouter = RegisteredRouter, TDefaultFrom extends string = string> = <
  const TTo extends string | undefined = undefined,
  TFrom extends string = TDefaultFrom,
  TMaskFrom extends string = TFrom,
  TMaskTo extends string = "",
>(
  options: NativeNavigateOptions<TRouter, TFrom, TTo, TMaskFrom, TMaskTo>,
) => Promise<void>;

export function useNavigate<
  TRouter extends AnyRouter = RegisteredRouter,
  TDefaultFrom extends string = string,
>(defaults?: { readonly from?: TDefaultFrom }): NativeNavigateResult<TRouter, TDefaultFrom> {
  const router = useRouter<TRouter>();
  return (options) =>
    router.navigate({
      ...options,
      from: options.from ?? defaults?.from,
      resetScroll: false,
      hashScrollIntoView: false,
      viewTransition: false,
      reloadDocument: false,
    } as NavigateOptions<TRouter>);
}

export function useCanGoBack(): Accessor<boolean> {
  const router = useRouter();
  return createMemo(() => router.stores.location.get().state.__TSR_index !== 0);
}

function RouteError(props: {
  readonly component: ErrorRouteComponent;
  readonly error: unknown;
  readonly router: AnyRouter;
}): SolidElement {
  return createComponent(props.component, {
    error: props.error,
    reset: () => {
      void props.router.invalidate();
    },
  });
}

function NativeMatch(props: { readonly routeId: string; readonly router: AnyRouter }): SolidElement {
  const match = (): MakeRouteMatchUnion<AnyRouter> | undefined =>
    props.router.stores.getMatchStore(props.routeId).get() as MakeRouteMatchUnion<AnyRouter> | undefined;
  const route = props.router.routesById[props.routeId];
  // Match objects change on every navigation. Only a rendering-state change
  // may replace this route component; loader data stays reactive via context.
  const status = createMemo(() => match()?.status);
  const render = (): SolidElement => {
    const currentStatus = status();
    if (currentStatus === undefined || currentStatus === "pending") {
      const Pending = route.options.pendingComponent ?? props.router.options.defaultPendingComponent;
      return createComponent((Pending ?? DefaultPendingComponent) as RouteComponent, {});
    }
    if (currentStatus === "error") {
      const ErrorView =
        route.options.errorComponent ?? props.router.options.defaultErrorComponent ?? DefaultErrorComponent;
      return createComponent(RouteError, {
        component: ErrorView as ErrorRouteComponent,
        get error() {
          return match()?.error;
        },
        router: props.router,
      });
    }
    if (currentStatus === "notFound") {
      const NotFoundView =
        route.options.notFoundComponent ?? props.router.options.defaultNotFoundComponent ?? DefaultNotFoundComponent;
      return createComponent(NotFoundView as NotFoundRouteComponent, {
        get data() {
          const error = match()?.error;
          return isNotFound(error) ? error.data : undefined;
        },
        isNotFound: true,
        routeId: props.routeId,
      });
    }
    const Component = route.options.component ?? props.router.options.defaultComponent ?? Outlet;
    return createComponent(Component as RouteComponent, {});
  };
  return createComponent(UniversalMatchContextProvider, {
    value: match,
    get children() {
      const rendered = createMemo(render);
      return rendered;
    },
  }) as SolidElement;
}

export function Outlet(): SolidElement {
  const router = useRouter();
  const nearestMatch = useContext(MatchContext);
  const routeId = nearestMatch?.()?.routeId;
  if (routeId === undefined) throw new Error("Outlet must be rendered by a route component");
  const childRouteId = createMemo(() => {
    const routeIds = router.stores.ids.get();
    return routeIds[routeIds.indexOf(routeId) + 1];
  });
  return (() => {
    const id = childRouteId();
    return id === undefined ? null : createComponent(NativeMatch, { routeId: id, router });
  }) as SolidElement;
}

function RouterLifecycle(props: { readonly router: AnyRouter }): null {
  const [failure, setFailure] = createSignal<unknown>();
  let settleCurrent: ((rendered: boolean) => void) | undefined;

  props.router.startTransition = (publish) => {
    settleCurrent?.(false);
    return new Promise<boolean>((resolve, reject) => {
      const settle = (rendered: boolean): void => {
        if (settleCurrent !== settle) return;
        settleCurrent = undefined;
        resolve(rendered);
      };
      const fail = (error: unknown): void => {
        if (settleCurrent !== settle) return;
        settleCurrent = undefined;
        reject(error);
      };
      settleCurrent = settle;
      void startTransition(() => publish()).then(() => settle(true), fail);
    });
  };

  const load = (): void => {
    void props.router.load().catch((error: unknown) => setFailure(() => error));
  };

  createEffect(() => {
    const error = failure();
    if (error !== undefined) throw error;
  });

  onMount(() => {
    if (mountedRouters.has(props.router)) {
      throw new Error("a native router instance can only be mounted in one Solid GPUI root");
    }
    mountedRouters.add(props.router);
    const unsubscribe = props.router.history.subscribe(load);
    load();
    onCleanup(() => {
      settleCurrent?.(false);
      mountedRouters.delete(props.router);
      unsubscribe();
      props.router.history.destroy();
      props.router.clearCache();
    });
  });
  return null;
}

export interface RouterProviderProps<TRouter extends AnyRouter> {
  readonly router: NativeRouter<TRouter>;
}

export function RouterProvider<TRouter extends AnyRouter>(props: RouterProviderProps<TRouter>): SolidElement {
  if (!nativeRouters.has(props.router)) {
    throw new TypeError("RouterProvider requires a router created by @solid-gpui/router");
  }
  return createComponent(UniversalRouterContextProvider, {
    value: props.router,
    get children() {
      createComponent(RouterLifecycle, { router: props.router });
      const rootRouteId = createMemo(() => props.router.stores.__store.get().matches[0]?.routeId);
      return createComponent(View, {
        style: ROUTER_ROOT_STYLE,
        children: () => {
          const routeId = rootRouteId();
          return routeId === undefined
            ? createComponent(DefaultPendingComponent, {})
            : createComponent(NativeMatch, { routeId, router: props.router });
        },
      });
    },
  }) as SolidElement;
}

export interface NativeActiveOptions {
  readonly exact?: boolean;
  readonly includeHash?: boolean;
  readonly includeSearch?: boolean;
}

export type LinkProps<
  TRouter extends AnyRouter = RegisteredRouter,
  TFrom extends string = string,
  TTo extends string | undefined = ".",
  TMaskFrom extends string = TFrom,
  TMaskTo extends string = ".",
> = NativeNavigateOptions<TRouter, TFrom, TTo, TMaskFrom, TMaskTo> &
  Omit<PressableProps, "children" | "onPress" | "style"> & {
    readonly activeOptions?: NativeActiveOptions;
    readonly activeStyle?: StyleProp;
    readonly children?: SolidElement | ((state: { readonly isActive: boolean }) => SolidElement);
    readonly inactiveStyle?: StyleProp;
    readonly onPress?: PressHandler;
    readonly style?: StyleProp;
  };

const NAVIGATION_KEYS = [
  "activeOptions",
  "activeStyle",
  "children",
  "from",
  "hash",
  "ignoreBlocker",
  "inactiveStyle",
  "mask",
  "onPress",
  "params",
  "replace",
  "search",
  "state",
  "style",
  "to",
  "unsafeRelative",
] as const;

export function Link<
  TRouter extends AnyRouter = RegisteredRouter,
  const TFrom extends string = string,
  const TTo extends string | undefined = ".",
  const TMaskFrom extends string = TFrom,
  const TMaskTo extends string = ".",
>(props: LinkProps<TRouter, TFrom, TTo, TMaskFrom, TMaskTo>): SolidElement {
  const router = useRouter<TRouter>();
  // Solid props are live getters; splitting and merging must preserve them.
  const [, baseProps] = splitProps(
    props as Omit<PressableProps, "children" | "onPress" | "style"> & Record<(typeof NAVIGATION_KEYS)[number], unknown>,
    NAVIGATION_KEYS,
  );
  const navigation = createMemo(
    () =>
      ({
        to: props.to,
        from: props.from,
        params: props.params,
        search: props.search,
        hash: props.hash,
        state: props.state,
        mask: props.mask,
        replace: props.replace,
        ignoreBlocker: props.ignoreBlocker,
        unsafeRelative: props.unsafeRelative,
        resetScroll: false,
        hashScrollIntoView: false,
        viewTransition: false,
        reloadDocument: false,
      }) as NavigateOptions<TRouter, TFrom, TTo, TMaskFrom, TMaskTo>,
  );
  const isActive = createMemo(() => {
    const current = router.stores.location.get();
    router.stores.resolvedLocation.get();
    router.stores.status.get();
    const matched = router.matchRoute(navigation() as never, {
      fuzzy: !(props.activeOptions?.exact ?? false),
      includeSearch: props.activeOptions?.includeSearch ?? true,
    });
    if (!matched) return false;
    if (!props.activeOptions?.includeHash) return true;
    return current.hash === router.buildLocation(navigation() as never).hash;
  });
  const child = createMemo(() => {
    const children = props.children;
    return typeof children === "function" ? children({ isActive: isActive() }) : children;
  });

  return createComponent(
    Pressable,
    mergeProps(baseProps, {
      get accessibilityRole() {
        return props.accessibilityRole ?? "link";
      },
      get accessibilitySelected() {
        return props.accessibilitySelected ?? isActive();
      },
      get style() {
        const stateStyle = isActive() ? props.activeStyle : props.inactiveStyle;
        if (stateStyle == null) return props.style;
        return { ...(props.style ?? {}), ...stateStyle };
      },
      onPress: (event) => {
        props.onPress?.(event);
        if (!props.disabled) void router.navigate(navigation());
      },
      get children() {
        return child();
      },
    } satisfies PressableProps),
  );
}

export type BackButtonProps = Omit<PressableProps, "children" | "onPress"> & {
  readonly children?: SolidElement;
  readonly onPress?: PressHandler;
};

export function BackButton(props: BackButtonProps): SolidElement {
  const router = useRouter();
  const canGoBack = useCanGoBack();
  return createComponent(
    Pressable,
    mergeProps(props, {
      get accessibilityRole() {
        return props.accessibilityRole ?? "button";
      },
      get accessibilityLabel() {
        return props.accessibilityLabel ?? "Back";
      },
      get disabled() {
        return props.disabled || !canGoBack();
      },
      onPress: (event) => {
        props.onPress?.(event);
        if (!props.disabled && canGoBack()) router.history.back();
      },
      get children() {
        return props.children ?? createComponent(Text, { children: "Back" });
      },
    } satisfies PressableProps),
  );
}
