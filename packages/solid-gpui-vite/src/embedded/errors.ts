/**
 * One error type for every embedded-packaging failure.
 *
 * The packager fails closed: a target it cannot qualify, a graph it cannot
 * validate, or a Cargo configuration it cannot merge never produces a product.
 * Callers distinguish user error (`EmbeddedPackagingError`) from a bug, so the
 * message always names the input and the expected value.
 */
export class EmbeddedPackagingError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "EmbeddedPackagingError";
  }
}

export function fail(message: string): never {
  throw new EmbeddedPackagingError(message);
}
