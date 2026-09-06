import {openSync, writeSync} from 'node:fs';
import {StdioTransport} from '../../packages/solid-gpui/dist/index.js';
const file = openSync('/tmp/solid-native-ui-commits.bin', 'w');
const submit = StdioTransport.prototype.submit;
StdioTransport.prototype.submit = function (frame: Uint8Array) {
  writeSync(file, frame);
  return submit.call(this, frame);
};
await import('../../examples/gallery/src/main.tsx');
