declare module "core-js-pure/actual/url" {
  const URLConstructor: typeof URL;
  export default URLConstructor;
}

declare module "core-js-pure/actual/url-search-params" {
  const URLSearchParamsConstructor: typeof URLSearchParams;
  export default URLSearchParamsConstructor;
}

declare module "core-js-pure/actual/dom-exception" {
  const DOMExceptionConstructor: typeof DOMException;
  export default DOMExceptionConstructor;
}

// The package omits a types condition for its ESM export. These are the standard
// interfaces used here, plus its two event-attribute helpers.
declare module "event-target-shim" {
  const Event: typeof globalThis.Event;
  type Event = globalThis.Event;
  const EventTarget: typeof globalThis.EventTarget;
  function getEventAttributeValue(target: globalThis.EventTarget, type: string): ((event: Event) => void) | null;
  function setEventAttributeValue(
    target: globalThis.EventTarget,
    type: string,
    value: ((event: Event) => void) | null,
  ): void;
  export { Event, EventTarget, getEventAttributeValue, setEventAttributeValue };
}
