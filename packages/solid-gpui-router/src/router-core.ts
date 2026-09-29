import {
  RouterCore,
  type AnyRoute,
  type GetStoreConfig,
  type RouterConstructorOptions,
  type TrailingSlashOption,
} from "@tanstack/router-core";
import { NativeHistory } from "./history";

/** Adapt the asynchronous native navigation boundary before RouterCore starts loaders. */
export class NativeRouterCore<
  TRouteTree extends AnyRoute,
  TTrailingSlashOption extends TrailingSlashOption,
  TDefaultStructuralSharingOption extends boolean,
  TDehydrated extends Record<string, any>,
> extends RouterCore<TRouteTree, TTrailingSlashOption, TDefaultStructuralSharingOption, NativeHistory, TDehydrated> {
  constructor(
    options: RouterConstructorOptions<
      TRouteTree,
      TTrailingSlashOption,
      TDefaultStructuralSharingOption,
      NativeHistory,
      TDehydrated
    >,
    storeFactory: GetStoreConfig,
  ) {
    super(options, storeFactory);
    // Core exposes commitLocation as an instance function, not a prototype method.
    const commitLocation = this.commitLocation;
    this.commitLocation = (next) => {
      const destination = next.maskedLocation ?? next;
      return this.history.commitRouterLocation(
        destination.publicHref,
        destination.state,
        next.replace ?? false,
        { ignoreBlocker: next.ignoreBlocker },
        () => commitLocation({ ...next, ignoreBlocker: true }),
      );
    };
  }
}
