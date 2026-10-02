import { createRequire } from "node:module";
import { closeSync, openSync, writeSync } from "node:fs";
//#region \0rolldown/runtime.js
var __require$1 = /* #__PURE__ */ (() => createRequire(import.meta.url))();
//#endregion
//#region packages/solid-gpui/src/protocol/generated/schema-facts.ts
var BODY_TAGS = {
	Snapshot: 1,
	Event: 2,
	Patch: 3,
	Command: 4
};
var EVENT_PAYLOAD_TAGS = {
	TextInputEventData: 1,
	CommandResult: 2,
	VisibleRangeEvent: 3,
	AnimationCompleteEvent: 4,
	KeyEvent: 5,
	PointerEvent: 6,
	PointerMoveEvent: 7,
	ScrollEvent: 8,
	SubmitEvent: 9,
	WindowResizeEvent: 10,
	WindowActivationEvent: 11,
	ActionEvent: 12,
	WindowAppearanceEvent: 13,
	LayoutEvent: 14,
	DragOverEvent: 15,
	DragDropEvent: 16,
	ExternalFileDropEvent: 17,
	NotificationResponseEvent: 18,
	PointerDownOutsideEvent: 19,
	CloseRequestedEvent: 20,
	ExtensionEvent: 21,
	ApplicationActivationEvent: 22
};
var EVENT_KIND_CODES = {
	Unknown: 0,
	Press: 1,
	Change: 2,
	Selection: 3,
	Focus: 4,
	Blur: 5,
	CommandResult: 6,
	VisibleRange: 7,
	AnimationComplete: 8,
	Key: 9,
	Pointer: 10,
	Hover: 11,
	Scroll: 12,
	Submit: 13,
	WindowResize: 14,
	WindowActivation: 15,
	SurfaceClosed: 16,
	Action: 17,
	WindowAppearance: 18,
	Layout: 19,
	Drag: 20,
	NotificationResponse: 21,
	PointerDownOutside: 22,
	CloseRequested: 23,
	Extension: 24,
	ApplicationActivation: 25
};
var COMMAND_KIND_CODES = {
	Unknown: 0,
	Focus: 1,
	Blur: 2,
	SetSelection: 3,
	ScrollToIndex: 4,
	ScrollToEnd: 5,
	SetTitle: 6,
	ResizeWindow: 7,
	ZoomWindow: 8,
	ToggleFullscreen: 9,
	OpenUrl: 10,
	FocusNext: 11,
	FocusPrev: 12,
	GetWindowSize: 13,
	GetFocus: 14,
	ClipboardWrite: 15,
	ClipboardRead: 16,
	OpenSurface: 17,
	FileDialogOpen: 18,
	FileDialogSave: 19,
	ShowNotification: 20,
	SetMenus: 21,
	SetKeybindings: 22,
	SetClosePolicy: 23,
	ResolveCloseRequest: 24,
	ReadTextFile: 25,
	WriteTextFile: 26,
	ClipboardWriteImage: 27,
	ClipboardReadImage: 28,
	LoadFont: 29,
	MinimizeWindow: 30,
	GetWindowBounds: 31,
	GetWindowState: 32,
	ActivateWindow: 33,
	GetScrollOffset: 34,
	ScrollToOffset: 35,
	InvokeNative: 36,
	CancelNative: 37,
	ConfigureApplication: 38,
	OpenPopup: 39,
	ClosePopup: 40
};
var COMMAND_KINDS$1 = [
	1,
	2,
	3,
	4,
	5,
	6,
	7,
	8,
	9,
	10,
	11,
	12,
	13,
	14,
	15,
	16,
	17,
	18,
	19,
	20,
	21,
	22,
	23,
	24,
	25,
	26,
	27,
	28,
	29,
	30,
	31,
	32,
	33,
	34,
	35,
	36,
	37,
	38,
	39,
	40
];
BODY_TAGS.Snapshot;
BODY_TAGS.Event;
BODY_TAGS.Patch;
BODY_TAGS.Command;
var MAX_FRAME_SIZE = 16777216;
var MAX_CLIPBOARD_TEXT_BYTES = 1 << 20;
var MAX_CLIPBOARD_IMAGE_BYTES = 16776192;
var MAX_FILE_WRITE_BYTES = 16776192;
var MAX_FILE_READ_BYTES = 16776192;
var MAX_NATIVE_CALL_BYTES = 1048576;
var MAX_IMAGE_SOURCE_BYTES = 1 << 20;
var EVENT_EXTENSION = EVENT_KIND_CODES.Extension;
var MAX_EXTENSION_TEXT_BYTES = 1 << 20;
var EVENT_PRESS = EVENT_KIND_CODES.Press;
var EVENT_CHANGE = EVENT_KIND_CODES.Change;
var EVENT_SELECTION = EVENT_KIND_CODES.Selection;
var EVENT_FOCUS = EVENT_KIND_CODES.Focus;
var EVENT_BLUR = EVENT_KIND_CODES.Blur;
var EVENT_COMMAND_RESULT = EVENT_KIND_CODES.CommandResult;
var EVENT_VISIBLE_RANGE = EVENT_KIND_CODES.VisibleRange;
var EVENT_ANIMATION_COMPLETE = EVENT_KIND_CODES.AnimationComplete;
var EVENT_KEY = EVENT_KIND_CODES.Key;
var EVENT_POINTER = EVENT_KIND_CODES.Pointer;
var EVENT_HOVER = EVENT_KIND_CODES.Hover;
var EVENT_SCROLL = EVENT_KIND_CODES.Scroll;
var EVENT_SUBMIT = EVENT_KIND_CODES.Submit;
var EVENT_WINDOW_RESIZE = EVENT_KIND_CODES.WindowResize;
var EVENT_WINDOW_ACTIVATION = EVENT_KIND_CODES.WindowActivation;
var EVENT_SURFACE_CLOSED = EVENT_KIND_CODES.SurfaceClosed;
var EVENT_ACTION = EVENT_KIND_CODES.Action;
var EVENT_WINDOW_APPEARANCE = EVENT_KIND_CODES.WindowAppearance;
var EVENT_LAYOUT = EVENT_KIND_CODES.Layout;
var EVENT_DRAG = EVENT_KIND_CODES.Drag;
var EVENT_NOTIFICATION_RESPONSE = EVENT_KIND_CODES.NotificationResponse;
var EVENT_POINTER_DOWN_OUTSIDE = EVENT_KIND_CODES.PointerDownOutside;
var EVENT_CLOSE_REQUESTED = EVENT_KIND_CODES.CloseRequested;
EVENT_PAYLOAD_TAGS.DragOverEvent;
EVENT_PAYLOAD_TAGS.DragDropEvent;
EVENT_PAYLOAD_TAGS.ExternalFileDropEvent;
var COMMAND_FOCUS = COMMAND_KIND_CODES.Focus;
var COMMAND_BLUR = COMMAND_KIND_CODES.Blur;
var COMMAND_SET_SELECTION = COMMAND_KIND_CODES.SetSelection;
var COMMAND_SCROLL_TO_INDEX = COMMAND_KIND_CODES.ScrollToIndex;
var COMMAND_SCROLL_TO_END = COMMAND_KIND_CODES.ScrollToEnd;
var COMMAND_GET_SCROLL_OFFSET = COMMAND_KIND_CODES.GetScrollOffset;
var COMMAND_SCROLL_TO_OFFSET = COMMAND_KIND_CODES.ScrollToOffset;
var COMMAND_SET_TITLE = COMMAND_KIND_CODES.SetTitle;
var COMMAND_RESIZE_WINDOW = COMMAND_KIND_CODES.ResizeWindow;
var COMMAND_ZOOM_WINDOW = COMMAND_KIND_CODES.ZoomWindow;
var COMMAND_TOGGLE_FULLSCREEN = COMMAND_KIND_CODES.ToggleFullscreen;
var COMMAND_OPEN_URL = COMMAND_KIND_CODES.OpenUrl;
var COMMAND_FOCUS_NEXT = COMMAND_KIND_CODES.FocusNext;
var COMMAND_FOCUS_PREV = COMMAND_KIND_CODES.FocusPrev;
var COMMAND_GET_WINDOW_SIZE = COMMAND_KIND_CODES.GetWindowSize;
var COMMAND_GET_FOCUS = COMMAND_KIND_CODES.GetFocus;
var COMMAND_CLIPBOARD_WRITE = COMMAND_KIND_CODES.ClipboardWrite;
var COMMAND_CLIPBOARD_READ = COMMAND_KIND_CODES.ClipboardRead;
var COMMAND_OPEN_SURFACE = COMMAND_KIND_CODES.OpenSurface;
var COMMAND_FILE_DIALOG_OPEN = COMMAND_KIND_CODES.FileDialogOpen;
var COMMAND_FILE_DIALOG_SAVE = COMMAND_KIND_CODES.FileDialogSave;
var COMMAND_SHOW_NOTIFICATION = COMMAND_KIND_CODES.ShowNotification;
var COMMAND_SET_MENUS = COMMAND_KIND_CODES.SetMenus;
var COMMAND_SET_KEYBINDINGS = COMMAND_KIND_CODES.SetKeybindings;
var COMMAND_SET_CLOSE_POLICY = COMMAND_KIND_CODES.SetClosePolicy;
var COMMAND_RESOLVE_CLOSE_REQUEST = COMMAND_KIND_CODES.ResolveCloseRequest;
var COMMAND_READ_TEXT_FILE = COMMAND_KIND_CODES.ReadTextFile;
var COMMAND_WRITE_TEXT_FILE = COMMAND_KIND_CODES.WriteTextFile;
var COMMAND_CLIPBOARD_WRITE_IMAGE = COMMAND_KIND_CODES.ClipboardWriteImage;
var COMMAND_CLIPBOARD_READ_IMAGE = COMMAND_KIND_CODES.ClipboardReadImage;
var COMMAND_LOAD_FONT = COMMAND_KIND_CODES.LoadFont;
var COMMAND_MINIMIZE_WINDOW = COMMAND_KIND_CODES.MinimizeWindow;
var COMMAND_GET_WINDOW_BOUNDS = COMMAND_KIND_CODES.GetWindowBounds;
var COMMAND_GET_WINDOW_STATE = COMMAND_KIND_CODES.GetWindowState;
var COMMAND_ACTIVATE_WINDOW = COMMAND_KIND_CODES.ActivateWindow;
var COMMAND_INVOKE_NATIVE = COMMAND_KIND_CODES.InvokeNative;
var COMMAND_CANCEL_NATIVE = COMMAND_KIND_CODES.CancelNative;
var COMMAND_KINDS = COMMAND_KINDS$1;
var KEY_MODIFIER_NAMES = {
	cmd: true,
	ctrl: true,
	alt: true,
	shift: true,
	function: true
};
var EVENT_APPLICATION_ACTIVATION = EVENT_KIND_CODES.ApplicationActivation;
var COMMAND_CONFIGURE_APPLICATION = COMMAND_KIND_CODES.ConfigureApplication;
var COMMAND_OPEN_POPUP = COMMAND_KIND_CODES.OpenPopup;
var COMMAND_CLOSE_POPUP = COMMAND_KIND_CODES.ClosePopup;
//#endregion
//#region packages/solid-gpui/src/protocol/types.ts
var ICON_NAMES = [
	"lucide:arrow-left",
	"lucide:bell",
	"lucide:book-open",
	"lucide:box",
	"lucide:brush",
	"lucide:check",
	"lucide:check-square",
	"lucide:chevron-down",
	"lucide:chevron-right",
	"lucide:clipboard-list",
	"lucide:clock",
	"lucide:copy",
	"lucide:cpu",
	"lucide:download",
	"lucide:ellipsis",
	"lucide:external-link",
	"lucide:file",
	"lucide:folder",
	"lucide:folder-open",
	"lucide:folder-plus",
	"lucide:gauge",
	"lucide:globe",
	"lucide:hard-drive",
	"lucide:heart",
	"lucide:home",
	"lucide:house",
	"lucide:image",
	"lucide:info",
	"lucide:layers",
	"lucide:layout",
	"lucide:list",
	"lucide:minus",
	"lucide:moon",
	"lucide:more-horizontal",
	"lucide:monitor",
	"lucide:mouse-pointer-click",
	"lucide:move",
	"lucide:package",
	"lucide:palette",
	"lucide:pause",
	"lucide:play",
	"lucide:plus",
	"lucide:power",
	"lucide:puzzle",
	"lucide:refresh-cw",
	"lucide:rocket",
	"lucide:search",
	"lucide:settings",
	"lucide:sliders-horizontal",
	"lucide:square",
	"lucide:star",
	"lucide:sun",
	"lucide:text-cursor-input",
	"lucide:trash-2",
	"lucide:triangle-alert",
	"lucide:type",
	"lucide:upload",
	"lucide:user",
	"lucide:users",
	"lucide:x",
	"lucide:zap"
];
var applicationIcons = /* @__PURE__ */ new Set();
function isIconName(name) {
	return typeof name === "string" && (ICON_NAMES.includes(name) || applicationIcons.has(name));
}
var EXTENSION_ENCODER$1 = new TextEncoder();
function isU32$2(value) {
	return typeof value === "number" && Number.isInteger(value) && value >= 0 && value <= 4294967295;
}
/** Validate one provider-neutral extension value without throwing on malformed input. */
function validateExtensionValue(value) {
	try {
		if (value === null || typeof value !== "object" || Array.isArray(value)) return false;
		const candidate = value;
		switch (candidate.type) {
			case "bool": return typeof candidate.value === "boolean";
			case "i32": return typeof candidate.value === "number" && Number.isInteger(candidate.value) && candidate.value >= -2147483648 && candidate.value <= 2147483647;
			case "u32": return isU32$2(candidate.value);
			case "f32": return typeof candidate.value === "number" && Number.isFinite(candidate.value) && Number.isFinite(Math.fround(candidate.value));
			case "text": return typeof candidate.value === "string" && EXTENSION_ENCODER$1.encode(candidate.value).byteLength <= 1048576;
			case "bytes": return candidate.value instanceof Uint8Array && candidate.value.byteLength <= 1048576;
			default: return false;
		}
	} catch {
		return false;
	}
}
/** Validate sorted extension fields and their independent aggregate payload caps. */
function validateExtensionFields(fields) {
	try {
		if (!Array.isArray(fields) || fields.length > 256) return false;
		let previous = 0;
		let textBytes = 0;
		let bytes = 0;
		for (const field of fields) {
			if (field === null || typeof field !== "object" || Array.isArray(field)) return false;
			const candidate = field;
			if (!isU32$2(candidate.id) || candidate.id <= previous || !validateExtensionValue(candidate.value)) return false;
			previous = candidate.id;
			const value = candidate.value;
			if (value.type === "text") textBytes += EXTENSION_ENCODER$1.encode(value.value).byteLength;
			if (value.type === "bytes") bytes += value.value.byteLength;
			if (textBytes > 1048576 || bytes > 1048576) return false;
		}
		return true;
	} catch {
		return false;
	}
}
/** Validate complete host extension properties without throwing on malformed input. */
function validateExtensionProperties(value) {
	try {
		if (value === null || typeof value !== "object" || Array.isArray(value)) return false;
		const candidate = value;
		if (!(candidate.providerId instanceof Uint8Array) || candidate.providerId.byteLength !== 16) return false;
		if (!(candidate.catalogDigest instanceof Uint8Array) || candidate.catalogDigest.byteLength !== 32) return false;
		if (candidate.entryId === 0 || !isU32$2(candidate.entryId) || !isU32$2(candidate.entryVersion) || candidate.entryVersion === 0) return false;
		if (!validateExtensionFields(candidate.fields)) return false;
		if (!Array.isArray(candidate.eventIds) || candidate.eventIds.length > 256) return false;
		let previous = 0;
		for (const id of candidate.eventIds) {
			if (!isU32$2(id) || id <= previous) return false;
			previous = id;
		}
		return true;
	} catch {
		return false;
	}
}
//#endregion
//#region packages/solid-gpui/src/protocol/frame.ts
function bytesFrom(value) {
	return value instanceof Uint8Array ? value : new Uint8Array(value);
}
/**
* Incremental decoder for the little-endian u32 length-prefixed frame stream.
* Complete frames borrow the caller's chunk when possible; consume them before
* mutating a retained input chunk. Only an incomplete frame is buffered.
*/
var FrameDecoder = class {
	maxFrameSize;
	buffer = /* @__PURE__ */ new Uint8Array(4);
	bufferedBytes = 0;
	constructor(maxFrameSize = MAX_FRAME_SIZE) {
		this.maxFrameSize = maxFrameSize;
		if (!Number.isInteger(maxFrameSize) || maxFrameSize < 1) throw new RangeError("maxFrameSize must be positive");
	}
	push(chunk) {
		const incoming = bytesFrom(chunk);
		const payloads = [];
		const view = new DataView(incoming.buffer, incoming.byteOffset, incoming.byteLength);
		let offset = 0;
		while (offset < incoming.byteLength) {
			if (this.bufferedBytes > 0) {
				if (this.bufferedBytes < 4) {
					const count = Math.min(4 - this.bufferedBytes, incoming.byteLength - offset);
					this.buffer.set(incoming.subarray(offset, offset + count), this.bufferedBytes);
					this.bufferedBytes += count;
					offset += count;
					if (this.bufferedBytes < 4) break;
				}
				const size = new DataView(this.buffer.buffer).getUint32(0, true);
				this.checkSize(size);
				const count = Math.min(4 + size - this.bufferedBytes, incoming.byteLength - offset);
				this.ensureCapacity(this.bufferedBytes + count);
				this.buffer.set(incoming.subarray(offset, offset + count), this.bufferedBytes);
				this.bufferedBytes += count;
				offset += count;
				if (this.bufferedBytes < 4 + size) break;
				payloads.push(this.buffer.slice(4, 4 + size));
				this.bufferedBytes = 0;
				continue;
			}
			if (incoming.byteLength - offset >= 4) {
				const size = view.getUint32(offset, true);
				this.checkSize(size);
				if (incoming.byteLength - offset - 4 >= size) {
					payloads.push(incoming.subarray(offset + 4, offset + 4 + size));
					offset += 4 + size;
					continue;
				}
			}
			const remainder = incoming.subarray(offset);
			this.ensureCapacity(remainder.byteLength);
			this.buffer.set(remainder);
			this.bufferedBytes = remainder.byteLength;
			break;
		}
		return payloads;
	}
	checkSize(size) {
		if (size <= this.maxFrameSize) return;
		this.bufferedBytes = 0;
		throw new RangeError(`frame length ${size} exceeds maximum ${this.maxFrameSize}`);
	}
	ensureCapacity(required) {
		if (required <= this.buffer.byteLength) return;
		const capacity = Math.min(Math.max(this.buffer.byteLength * 2, required), this.maxFrameSize + 4);
		const grown = new Uint8Array(capacity);
		grown.set(this.buffer.subarray(0, this.bufferedBytes));
		this.buffer = grown;
	}
};
//#endregion
//#region node_modules/.bun/bebop@3.2.3/node_modules/bebop/dist/index.mjs
var __require = /* @__PURE__ */ ((x) => typeof __require$1 !== "undefined" ? __require$1 : typeof Proxy !== "undefined" ? new Proxy(x, { get: (a, b) => (typeof __require$1 !== "undefined" ? __require$1 : a)[b] }) : x)(function(x) {
	if (typeof __require$1 !== "undefined") return __require$1.apply(this, arguments);
	throw new Error("Dynamic require of \"" + x + "\" is not supported");
});
new TextDecoder();
var hexDigits = "0123456789abcdef";
var asciiToHex = [
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	1,
	2,
	3,
	4,
	5,
	6,
	7,
	8,
	9,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	10,
	11,
	12,
	13,
	14,
	15,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	10,
	11,
	12,
	13,
	14,
	15,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0,
	0
];
var guidDelimiter = "-";
var ticksBetweenEpochs = 621355968000000000n;
var dateMask = 4611686018427387903n;
var emptyByteArray = /* @__PURE__ */ new Uint8Array(0);
var emptyString = "";
var byteToHex = [];
for (const x of hexDigits) for (const y of hexDigits) byteToHex.push(x + y);
var BebopRuntimeError = class extends Error {
	constructor(message) {
		super(message);
		this.name = "BebopRuntimeError";
	}
};
var readGuid = (buffer, start) => {
	let s = byteToHex[buffer[start + 3]];
	s += byteToHex[buffer[start + 2]];
	s += byteToHex[buffer[start + 1]];
	s += byteToHex[buffer[start]];
	s += guidDelimiter;
	s += byteToHex[buffer[start + 5]];
	s += byteToHex[buffer[start + 4]];
	s += guidDelimiter;
	s += byteToHex[buffer[start + 7]];
	s += byteToHex[buffer[start + 6]];
	s += guidDelimiter;
	s += byteToHex[buffer[start + 8]];
	s += byteToHex[buffer[start + 9]];
	s += guidDelimiter;
	s += byteToHex[buffer[start + 10]];
	s += byteToHex[buffer[start + 11]];
	s += byteToHex[buffer[start + 12]];
	s += byteToHex[buffer[start + 13]];
	s += byteToHex[buffer[start + 14]];
	s += byteToHex[buffer[start + 15]];
	return s;
};
var BebopView = class _BebopView {
	static textDecoder;
	static writeBuffer = /* @__PURE__ */ new Uint8Array(256);
	static writeBufferView = new DataView(_BebopView.writeBuffer.buffer);
	static instance;
	static getInstance() {
		if (!_BebopView.instance) _BebopView.instance = new _BebopView();
		return _BebopView.instance;
	}
	minimumTextDecoderLength = 300;
	buffer;
	view;
	index;
	length;
	constructor() {
		this.buffer = _BebopView.writeBuffer;
		this.view = _BebopView.writeBufferView;
		this.index = 0;
		this.length = 0;
	}
	startReading(buffer) {
		this.buffer = buffer;
		this.view = new DataView(this.buffer.buffer, this.buffer.byteOffset, this.buffer.byteLength);
		this.index = 0;
		this.length = buffer.length;
	}
	startWriting() {
		this.buffer = _BebopView.writeBuffer;
		this.view = _BebopView.writeBufferView;
		this.index = 0;
		this.length = 0;
	}
	guaranteeBufferLength(length) {
		if (length > this.buffer.length) {
			const data = new Uint8Array(length << 1);
			data.set(this.buffer);
			this.buffer = data;
			this.view = new DataView(data.buffer);
		}
	}
	growBy(amount) {
		this.length += amount;
		this.guaranteeBufferLength(this.length);
	}
	skip(amount) {
		this.index += amount;
	}
	toArray() {
		return this.buffer.subarray(0, this.length);
	}
	readByte() {
		return this.buffer[this.index++];
	}
	readUint16() {
		const result = this.view.getUint16(this.index, true);
		this.index += 2;
		return result;
	}
	readInt16() {
		const result = this.view.getInt16(this.index, true);
		this.index += 2;
		return result;
	}
	readUint32() {
		const result = this.view.getUint32(this.index, true);
		this.index += 4;
		return result;
	}
	readInt32() {
		const result = this.view.getInt32(this.index, true);
		this.index += 4;
		return result;
	}
	readUint64() {
		const result = this.view.getBigUint64(this.index, true);
		this.index += 8;
		return result;
	}
	readInt64() {
		const result = this.view.getBigInt64(this.index, true);
		this.index += 8;
		return result;
	}
	readFloat32() {
		const result = this.view.getFloat32(this.index, true);
		this.index += 4;
		return result;
	}
	readFloat64() {
		const result = this.view.getFloat64(this.index, true);
		this.index += 8;
		return result;
	}
	writeByte(value) {
		const index = this.length;
		this.growBy(1);
		this.buffer[index] = value;
	}
	writeUint16(value) {
		const index = this.length;
		this.growBy(2);
		this.view.setUint16(index, value, true);
	}
	writeInt16(value) {
		const index = this.length;
		this.growBy(2);
		this.view.setInt16(index, value, true);
	}
	writeUint32(value) {
		const index = this.length;
		this.growBy(4);
		this.view.setUint32(index, value, true);
	}
	writeInt32(value) {
		const index = this.length;
		this.growBy(4);
		this.view.setInt32(index, value, true);
	}
	writeUint64(value) {
		const index = this.length;
		this.growBy(8);
		this.view.setBigUint64(index, value, true);
	}
	writeInt64(value) {
		const index = this.length;
		this.growBy(8);
		this.view.setBigInt64(index, value, true);
	}
	writeFloat32(value) {
		const index = this.length;
		this.growBy(4);
		this.view.setFloat32(index, value, true);
	}
	writeFloat64(value) {
		const index = this.length;
		this.growBy(8);
		this.view.setFloat64(index, value, true);
	}
	readBytes() {
		const length = this.readUint32();
		if (length === 0) return emptyByteArray;
		const start = this.index;
		const end = start + length;
		this.index = end;
		return this.buffer.subarray(start, end);
	}
	writeBytes(value) {
		const byteCount = value.length;
		this.writeUint32(byteCount);
		if (byteCount === 0) return;
		const index = this.length;
		this.growBy(byteCount);
		this.buffer.set(value, index);
	}
	/**
	* Reads a length-prefixed UTF-8-encoded string.
	*/
	readString() {
		const lengthBytes = this.readUint32();
		if (lengthBytes === 0) return emptyString;
		if (lengthBytes >= this.minimumTextDecoderLength) {
			if (typeof __require !== "undefined") {
				if (typeof TextDecoder === "undefined") throw new BebopRuntimeError("TextDecoder is not defined on 'global'. Please include a polyfill.");
			}
			if (_BebopView.textDecoder === void 0) _BebopView.textDecoder = new TextDecoder();
			return _BebopView.textDecoder.decode(this.buffer.subarray(this.index, this.index += lengthBytes));
		}
		const end = this.index + lengthBytes;
		let result = "";
		let codePoint;
		while (this.index < end) {
			const a = this.buffer[this.index++];
			if (a < 192) codePoint = a;
			else {
				const b = this.buffer[this.index++];
				if (a < 224) codePoint = (a & 31) << 6 | b & 63;
				else {
					const c = this.buffer[this.index++];
					if (a < 240) codePoint = (a & 15) << 12 | (b & 63) << 6 | c & 63;
					else {
						const d = this.buffer[this.index++];
						codePoint = (a & 7) << 18 | (b & 63) << 12 | (c & 63) << 6 | d & 63;
					}
				}
			}
			if (codePoint < 65536) result += String.fromCharCode(codePoint);
			else {
				codePoint -= 65536;
				result += String.fromCharCode((codePoint >> 10) + 55296, (codePoint & 1023) + 56320);
			}
		}
		this.index = end;
		return result;
	}
	/**
	* Writes a length-prefixed UTF-8-encoded string.
	*/
	writeString(value) {
		const stringLength = value.length;
		if (stringLength === 0) {
			this.writeUint32(0);
			return;
		}
		const maxBytes = 4 + stringLength * 3;
		this.guaranteeBufferLength(this.length + maxBytes);
		let w = this.length + 4;
		const start = w;
		let codePoint;
		for (let i = 0; i < stringLength; i++) {
			const a = value.charCodeAt(i);
			if (i + 1 === stringLength || a < 55296 || a >= 56320) codePoint = a;
			else {
				const b = value.charCodeAt(++i);
				codePoint = (a << 10) + b + -56613888;
			}
			if (codePoint < 128) this.buffer[w++] = codePoint;
			else {
				if (codePoint < 2048) this.buffer[w++] = codePoint >> 6 & 31 | 192;
				else {
					if (codePoint < 65536) this.buffer[w++] = codePoint >> 12 & 15 | 224;
					else {
						this.buffer[w++] = codePoint >> 18 & 7 | 240;
						this.buffer[w++] = codePoint >> 12 & 63 | 128;
					}
					this.buffer[w++] = codePoint >> 6 & 63 | 128;
				}
				this.buffer[w++] = codePoint & 63 | 128;
			}
		}
		const written = w - start;
		this.view.setUint32(this.length, written, true);
		this.length += 4 + written;
	}
	readGuid() {
		const start = this.index;
		this.index += 16;
		return readGuid(this.buffer, start);
	}
	writeGuid(value) {
		let p = 0;
		let a = 0;
		const index = this.length;
		this.growBy(16);
		a = a << 4 | asciiToHex[value.charCodeAt(p++)];
		a = a << 4 | asciiToHex[value.charCodeAt(p++)];
		a = a << 4 | asciiToHex[value.charCodeAt(p++)];
		a = a << 4 | asciiToHex[value.charCodeAt(p++)];
		a = a << 4 | asciiToHex[value.charCodeAt(p++)];
		a = a << 4 | asciiToHex[value.charCodeAt(p++)];
		a = a << 4 | asciiToHex[value.charCodeAt(p++)];
		a = a << 4 | asciiToHex[value.charCodeAt(p++)];
		p += value.charCodeAt(p) === 45;
		this.view.setUint32(index, a, true);
		a = a << 4 | asciiToHex[value.charCodeAt(p++)];
		a = a << 4 | asciiToHex[value.charCodeAt(p++)];
		a = a << 4 | asciiToHex[value.charCodeAt(p++)];
		a = a << 4 | asciiToHex[value.charCodeAt(p++)];
		p += value.charCodeAt(p) === 45;
		this.view.setUint16(index + 4, a, true);
		a = a << 4 | asciiToHex[value.charCodeAt(p++)];
		a = a << 4 | asciiToHex[value.charCodeAt(p++)];
		a = a << 4 | asciiToHex[value.charCodeAt(p++)];
		a = a << 4 | asciiToHex[value.charCodeAt(p++)];
		p += value.charCodeAt(p) === 45;
		this.view.setUint16(index + 6, a, true);
		a = a << 4 | asciiToHex[value.charCodeAt(p++)];
		a = a << 4 | asciiToHex[value.charCodeAt(p++)];
		a = a << 4 | asciiToHex[value.charCodeAt(p++)];
		a = a << 4 | asciiToHex[value.charCodeAt(p++)];
		p += value.charCodeAt(p) === 45;
		a = a << 4 | asciiToHex[value.charCodeAt(p++)];
		a = a << 4 | asciiToHex[value.charCodeAt(p++)];
		a = a << 4 | asciiToHex[value.charCodeAt(p++)];
		a = a << 4 | asciiToHex[value.charCodeAt(p++)];
		this.view.setUint32(index + 8, a, false);
		a = a << 4 | asciiToHex[value.charCodeAt(p++)];
		a = a << 4 | asciiToHex[value.charCodeAt(p++)];
		a = a << 4 | asciiToHex[value.charCodeAt(p++)];
		a = a << 4 | asciiToHex[value.charCodeAt(p++)];
		a = a << 4 | asciiToHex[value.charCodeAt(p++)];
		a = a << 4 | asciiToHex[value.charCodeAt(p++)];
		a = a << 4 | asciiToHex[value.charCodeAt(p++)];
		a = a << 4 | asciiToHex[value.charCodeAt(p++)];
		this.view.setUint32(index + 12, a, false);
	}
	readDate() {
		const ms = ((this.readUint64() & dateMask) - ticksBetweenEpochs) / 10000n;
		return new Date(Number(ms));
	}
	writeDate(date) {
		const ticks = BigInt(date.getTime()) * 10000n + ticksBetweenEpochs;
		this.writeUint64(ticks & dateMask);
	}
	/**
	* Reserve some space to write a message's length prefix, and return its index.
	* The length is stored as a little-endian fixed-width unsigned 32-bit integer, so 4 bytes are reserved.
	*/
	reserveMessageLength() {
		const i = this.length;
		this.growBy(4);
		return i;
	}
	/**
	* Fill in a message's length prefix.
	*/
	fillMessageLength(position, messageLength) {
		this.view.setUint32(position, messageLength, true);
	}
	/**
	* Read out a message's length prefix.
	*/
	readMessageLength() {
		const result = this.view.getUint32(this.index, true);
		this.index += 4;
		return result;
	}
};
//#endregion
//#region packages/solid-gpui/src/protocol/generated/schema-meta.ts
var PROTOCOL_SCHEMA = {
	schema: "protocol.bop",
	root: "Envelope",
	digest: "67cb7354b185f9ff16e28ea4c321c57ee53d610c0eaa0a3ea96012f18463a47d",
	definitions: {
		NodeKind: {
			kind: "enum",
			base: "uint8",
			values: [
				0,
				1,
				2,
				3,
				4,
				5,
				6,
				7,
				8,
				9
			],
			names: [
				"Unspecified",
				"View",
				"Text",
				"Pressable",
				"RawText",
				"TextInput",
				"VirtualList",
				"Image",
				"Extension",
				"Icon"
			]
		},
		WindowAppearance: {
			kind: "enum",
			base: "uint8",
			values: [
				0,
				1,
				2
			],
			names: [
				"Unspecified",
				"Light",
				"Dark"
			]
		},
		EventKind: {
			kind: "enum",
			base: "uint8",
			values: [
				0,
				1,
				2,
				3,
				4,
				5,
				6,
				7,
				8,
				9,
				10,
				11,
				12,
				13,
				14,
				15,
				16,
				17,
				18,
				19,
				20,
				21,
				22,
				23,
				24,
				25
			],
			names: [
				"Unknown",
				"Press",
				"Change",
				"Selection",
				"Focus",
				"Blur",
				"CommandResult",
				"VisibleRange",
				"AnimationComplete",
				"Key",
				"Pointer",
				"Hover",
				"Scroll",
				"Submit",
				"WindowResize",
				"WindowActivation",
				"SurfaceClosed",
				"Action",
				"WindowAppearance",
				"Layout",
				"Drag",
				"NotificationResponse",
				"PointerDownOutside",
				"CloseRequested",
				"Extension",
				"ApplicationActivation"
			]
		},
		CommandKind: {
			kind: "enum",
			base: "uint8",
			values: [
				0,
				1,
				2,
				3,
				4,
				5,
				6,
				7,
				8,
				9,
				10,
				11,
				12,
				13,
				14,
				15,
				16,
				17,
				18,
				19,
				20,
				21,
				22,
				23,
				24,
				25,
				26,
				27,
				28,
				29,
				30,
				31,
				32,
				33,
				34,
				35,
				36,
				37,
				38,
				39,
				40
			],
			names: [
				"Unknown",
				"Focus",
				"Blur",
				"SetSelection",
				"ScrollToIndex",
				"ScrollToEnd",
				"SetTitle",
				"ResizeWindow",
				"ZoomWindow",
				"ToggleFullscreen",
				"OpenUrl",
				"FocusNext",
				"FocusPrev",
				"GetWindowSize",
				"GetFocus",
				"ClipboardWrite",
				"ClipboardRead",
				"OpenSurface",
				"FileDialogOpen",
				"FileDialogSave",
				"ShowNotification",
				"SetMenus",
				"SetKeybindings",
				"SetClosePolicy",
				"ResolveCloseRequest",
				"ReadTextFile",
				"WriteTextFile",
				"ClipboardWriteImage",
				"ClipboardReadImage",
				"LoadFont",
				"MinimizeWindow",
				"GetWindowBounds",
				"GetWindowState",
				"ActivateWindow",
				"GetScrollOffset",
				"ScrollToOffset",
				"InvokeNative",
				"CancelNative",
				"ConfigureApplication",
				"OpenPopup",
				"ClosePopup"
			]
		},
		Body: {
			kind: "union",
			branches: [
				{
					id: 1,
					type: "Snapshot"
				},
				{
					id: 2,
					type: "Event"
				},
				{
					id: 3,
					type: "Patch"
				},
				{
					id: 4,
					type: "Command"
				}
			]
		},
		Snapshot: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "surfaceId",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 2,
					name: "epoch",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 3,
					name: "baseRevision",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 4,
					name: "revision",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 5,
					name: "nodes",
					type: {
						kind: "array",
						element: {
							kind: "def",
							name: "Node"
						}
					}
				}
			]
		},
		Event: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "surfaceId",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 2,
					name: "epoch",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 3,
					name: "revision",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 4,
					name: "sequence",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 5,
					name: "nodeId",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 6,
					name: "listenerId",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 7,
					name: "eventType",
					type: {
						kind: "def",
						name: "EventKind"
					}
				},
				{
					id: 8,
					name: "payload",
					type: {
						kind: "def",
						name: "EventPayload"
					}
				}
			]
		},
		Patch: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "surfaceId",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 2,
					name: "epoch",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 3,
					name: "baseRevision",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 4,
					name: "revision",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 5,
					name: "operations",
					type: {
						kind: "array",
						element: {
							kind: "def",
							name: "PatchOperation"
						}
					}
				}
			]
		},
		Command: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "surfaceId",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 2,
					name: "epoch",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 3,
					name: "afterRevision",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 4,
					name: "requestId",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 5,
					name: "nodeId",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 6,
					name: "kind",
					type: {
						kind: "def",
						name: "CommandKind"
					}
				},
				{
					id: 7,
					name: "payload",
					type: {
						kind: "def",
						name: "CommandPayload"
					}
				}
			]
		},
		ExtensionValue: {
			kind: "union",
			branches: [
				{
					id: 1,
					type: "ExtensionBoolValue"
				},
				{
					id: 2,
					type: "ExtensionInt32Value"
				},
				{
					id: 3,
					type: "ExtensionU32Value"
				},
				{
					id: 4,
					type: "ExtensionF32Value"
				},
				{
					id: 5,
					type: "ExtensionTextValue"
				},
				{
					id: 6,
					type: "ExtensionBytesValue"
				}
			]
		},
		ExtensionBoolValue: {
			kind: "message",
			fields: [{
				id: 1,
				name: "value",
				type: {
					kind: "scalar",
					name: "bool"
				}
			}]
		},
		ExtensionInt32Value: {
			kind: "message",
			fields: [{
				id: 1,
				name: "value",
				type: {
					kind: "scalar",
					name: "int32"
				}
			}]
		},
		ExtensionU32Value: {
			kind: "message",
			fields: [{
				id: 1,
				name: "value",
				type: {
					kind: "scalar",
					name: "uint32"
				}
			}]
		},
		ExtensionF32Value: {
			kind: "message",
			fields: [{
				id: 1,
				name: "value",
				type: {
					kind: "scalar",
					name: "float32"
				}
			}]
		},
		ExtensionTextValue: {
			kind: "message",
			fields: [{
				id: 1,
				name: "value",
				type: {
					kind: "scalar",
					name: "string"
				}
			}]
		},
		ExtensionBytesValue: {
			kind: "message",
			fields: [{
				id: 1,
				name: "value",
				type: {
					kind: "array",
					element: {
						kind: "scalar",
						name: "byte"
					}
				}
			}]
		},
		ExtensionField: {
			kind: "message",
			fields: [{
				id: 1,
				name: "id",
				type: {
					kind: "scalar",
					name: "uint32"
				}
			}, {
				id: 2,
				name: "value",
				type: {
					kind: "def",
					name: "ExtensionValue"
				}
			}]
		},
		HostProperties: {
			kind: "union",
			branches: [
				{
					id: 1,
					type: "TextInputProperties"
				},
				{
					id: 2,
					type: "VirtualListProperties"
				},
				{
					id: 3,
					type: "ImageProperties"
				},
				{
					id: 4,
					type: "DragProperties"
				},
				{
					id: 5,
					type: "ExtensionProperties"
				},
				{
					id: 6,
					type: "IconProperties"
				}
			]
		},
		TextInputProperties: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "value",
					type: {
						kind: "scalar",
						name: "string"
					}
				},
				{
					id: 2,
					name: "placeholder",
					type: {
						kind: "scalar",
						name: "string"
					}
				},
				{
					id: 3,
					name: "multiline",
					type: {
						kind: "scalar",
						name: "bool"
					}
				},
				{
					id: 4,
					name: "disabled",
					type: {
						kind: "scalar",
						name: "bool"
					}
				},
				{
					id: 5,
					name: "controlled",
					type: {
						kind: "scalar",
						name: "bool"
					}
				},
				{
					id: 6,
					name: "ackEditSeq",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 7,
					name: "selectionStart",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 8,
					name: "selectionEnd",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 9,
					name: "markedStart",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 10,
					name: "markedEnd",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 11,
					name: "maxLength",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 12,
					name: "selectionReversed",
					type: {
						kind: "scalar",
						name: "bool"
					}
				}
			]
		},
		VirtualListProperties: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "itemCount",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 2,
					name: "rangeStart",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 3,
					name: "rangeEnd",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 4,
					name: "estimatedItemSize",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 5,
					name: "overscan",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 6,
					name: "dataRevision",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 7,
					name: "dataEdit",
					type: {
						kind: "def",
						name: "VirtualListDataEdit"
					}
				}
			]
		},
		ImageProperties: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "source",
					type: {
						kind: "scalar",
						name: "string"
					}
				},
				{
					id: 2,
					name: "objectFit",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 3,
					name: "fallbackSource",
					type: {
						kind: "scalar",
						name: "string"
					}
				},
				{
					id: 4,
					name: "sources",
					type: {
						kind: "array",
						element: {
							kind: "def",
							name: "ImageCandidate"
						}
					}
				}
			]
		},
		DragProperties: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "dragType",
					type: {
						kind: "scalar",
						name: "string"
					}
				},
				{
					id: 2,
					name: "exportFiles",
					type: {
						kind: "array",
						element: {
							kind: "scalar",
							name: "string"
						}
					}
				},
				{
					id: 3,
					name: "acceptsDragOver",
					type: {
						kind: "scalar",
						name: "bool"
					}
				},
				{
					id: 4,
					name: "acceptsDrop",
					type: {
						kind: "scalar",
						name: "bool"
					}
				}
			]
		},
		ExtensionProperties: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "providerId",
					type: {
						kind: "array",
						element: {
							kind: "scalar",
							name: "byte"
						}
					}
				},
				{
					id: 2,
					name: "catalogDigest",
					type: {
						kind: "array",
						element: {
							kind: "scalar",
							name: "byte"
						}
					}
				},
				{
					id: 3,
					name: "entryId",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 4,
					name: "entryVersion",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 5,
					name: "fields",
					type: {
						kind: "array",
						element: {
							kind: "def",
							name: "ExtensionField"
						}
					}
				},
				{
					id: 6,
					name: "eventIds",
					type: {
						kind: "array",
						element: {
							kind: "scalar",
							name: "uint32"
						}
					}
				}
			]
		},
		IconProperties: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "name",
					type: {
						kind: "scalar",
						name: "string"
					}
				},
				{
					id: 2,
					name: "size",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 3,
					name: "color",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				}
			]
		},
		ImageCandidate: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "source",
					type: {
						kind: "scalar",
						name: "string"
					}
				},
				{
					id: 2,
					name: "width",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 3,
					name: "height",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				}
			]
		},
		VirtualListDataEdit: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "baseRevision",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 2,
					name: "start",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 3,
					name: "oldCount",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 4,
					name: "newCount",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				}
			]
		},
		AccessibilityProperties: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "role",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 2,
					name: "label",
					type: {
						kind: "scalar",
						name: "string"
					}
				},
				{
					id: 3,
					name: "description",
					type: {
						kind: "scalar",
						name: "string"
					}
				},
				{
					id: 4,
					name: "disabled",
					type: {
						kind: "scalar",
						name: "bool"
					}
				},
				{
					id: 5,
					name: "checked",
					type: {
						kind: "scalar",
						name: "bool"
					}
				},
				{
					id: 6,
					name: "selected",
					type: {
						kind: "scalar",
						name: "bool"
					}
				},
				{
					id: 7,
					name: "value",
					type: {
						kind: "scalar",
						name: "string"
					}
				},
				{
					id: 8,
					name: "expanded",
					type: {
						kind: "scalar",
						name: "bool"
					}
				},
				{
					id: 9,
					name: "level",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 10,
					name: "live",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				}
			]
		},
		Transition: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "durationMs",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 2,
					name: "delayMs",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 3,
					name: "easing",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 4,
					name: "propertyMask",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				}
			]
		},
		BoxShadowValue: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "offsetX",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 2,
					name: "offsetY",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 3,
					name: "blurRadius",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 4,
					name: "spreadRadius",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 5,
					name: "color",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 6,
					name: "inset",
					type: {
						kind: "scalar",
						name: "bool"
					}
				}
			]
		},
		BoxShadowSet: {
			kind: "message",
			fields: [{
				id: 1,
				name: "values",
				type: {
					kind: "array",
					element: {
						kind: "def",
						name: "BoxShadowValue"
					}
				}
			}]
		},
		LinearGradient: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "angle",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 2,
					name: "startColor",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 3,
					name: "startPosition",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 4,
					name: "endColor",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 5,
					name: "endPosition",
					type: {
						kind: "scalar",
						name: "float32"
					}
				}
			]
		},
		Style: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "width",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 2,
					name: "height",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 3,
					name: "flexDirection",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 4,
					name: "flexGrow",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 5,
					name: "padding",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 6,
					name: "gap",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 7,
					name: "backgroundColor",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 8,
					name: "color",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 9,
					name: "opacity",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 10,
					name: "transition",
					type: {
						kind: "def",
						name: "Transition"
					}
				},
				{
					id: 11,
					name: "justifyContent",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 12,
					name: "alignItems",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 13,
					name: "borderRadius",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 14,
					name: "borderWidth",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 15,
					name: "borderColor",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 16,
					name: "fontSize",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 17,
					name: "fontWeight",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 18,
					name: "overflow",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 19,
					name: "lineClamp",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 20,
					name: "textOverflow",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 21,
					name: "marginTop",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 22,
					name: "marginRight",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 23,
					name: "marginBottom",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 24,
					name: "marginLeft",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 25,
					name: "fontStyle",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 26,
					name: "textDecoration",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 27,
					name: "lineHeight",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 28,
					name: "minWidth",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 29,
					name: "maxWidth",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 30,
					name: "minHeight",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 31,
					name: "maxHeight",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 32,
					name: "flexShrink",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 33,
					name: "alignSelf",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 34,
					name: "position",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 35,
					name: "left",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 36,
					name: "top",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 37,
					name: "right",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 38,
					name: "bottom",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 39,
					name: "cursor",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 40,
					name: "textAlign",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 41,
					name: "boxShadow",
					type: {
						kind: "def",
						name: "BoxShadowSet"
					}
				},
				{
					id: 42,
					name: "fontFamily",
					type: {
						kind: "scalar",
						name: "string"
					}
				},
				{
					id: 43,
					name: "paddingTop",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 44,
					name: "paddingRight",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 45,
					name: "paddingBottom",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 46,
					name: "paddingLeft",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 47,
					name: "borderTopWidth",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 48,
					name: "borderRightWidth",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 49,
					name: "borderBottomWidth",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 50,
					name: "borderLeftWidth",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 51,
					name: "borderTopLeftRadius",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 52,
					name: "borderTopRightRadius",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 53,
					name: "borderBottomRightRadius",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 54,
					name: "borderBottomLeftRadius",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 55,
					name: "widthPercent",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 56,
					name: "heightPercent",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 57,
					name: "flexWrap",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 58,
					name: "linearGradient",
					type: {
						kind: "def",
						name: "LinearGradient"
					}
				},
				{
					id: 59,
					name: "borderTopColor",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 60,
					name: "borderRightColor",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 61,
					name: "borderBottomColor",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 62,
					name: "borderLeftColor",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 63,
					name: "gridColumns",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 64,
					name: "gridRows",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 65,
					name: "gridColumnSpan",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 66,
					name: "gridRowSpan",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				}
			]
		},
		Node: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "id",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 2,
					name: "parentId",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 3,
					name: "index",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 4,
					name: "kind",
					type: {
						kind: "def",
						name: "NodeKind"
					}
				},
				{
					id: 5,
					name: "style",
					type: {
						kind: "def",
						name: "Style"
					}
				},
				{
					id: 6,
					name: "text",
					type: {
						kind: "scalar",
						name: "string"
					}
				},
				{
					id: 7,
					name: "listenerId",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 8,
					name: "hostProperties",
					type: {
						kind: "def",
						name: "HostProperties"
					}
				},
				{
					id: 9,
					name: "accessibility",
					type: {
						kind: "def",
						name: "AccessibilityProperties"
					}
				},
				{
					id: 10,
					name: "focusable",
					type: {
						kind: "scalar",
						name: "bool"
					}
				},
				{
					id: 11,
					name: "selectable",
					type: {
						kind: "scalar",
						name: "bool"
					}
				},
				{
					id: 12,
					name: "tooltip",
					type: {
						kind: "scalar",
						name: "string"
					}
				},
				{
					id: 13,
					name: "acceptsPointerMove",
					type: {
						kind: "scalar",
						name: "bool"
					}
				},
				{
					id: 14,
					name: "observesLayout",
					type: {
						kind: "scalar",
						name: "bool"
					}
				}
			]
		},
		ClearStyle: {
			kind: "message",
			fields: []
		},
		PatchOperationValue: {
			kind: "union",
			branches: [
				{
					id: 1,
					type: "PatchCreate"
				},
				{
					id: 2,
					type: "PatchUpdate"
				},
				{
					id: 3,
					type: "PatchMove"
				},
				{
					id: 4,
					type: "PatchDelete"
				}
			]
		},
		PatchCreate: {
			kind: "message",
			fields: [{
				id: 1,
				name: "node",
				type: {
					kind: "def",
					name: "Node"
				}
			}]
		},
		PatchUpdate: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "id",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 2,
					name: "mask",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 3,
					name: "style",
					type: {
						kind: "def",
						name: "Style"
					}
				},
				{
					id: 4,
					name: "clearStyle",
					type: {
						kind: "def",
						name: "ClearStyle"
					}
				},
				{
					id: 5,
					name: "text",
					type: {
						kind: "scalar",
						name: "string"
					}
				},
				{
					id: 6,
					name: "listenerId",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 7,
					name: "hostProperties",
					type: {
						kind: "def",
						name: "HostProperties"
					}
				},
				{
					id: 8,
					name: "accessibility",
					type: {
						kind: "def",
						name: "AccessibilityProperties"
					}
				},
				{
					id: 9,
					name: "focusable",
					type: {
						kind: "scalar",
						name: "bool"
					}
				},
				{
					id: 10,
					name: "selectable",
					type: {
						kind: "scalar",
						name: "bool"
					}
				},
				{
					id: 11,
					name: "tooltip",
					type: {
						kind: "scalar",
						name: "string"
					}
				},
				{
					id: 12,
					name: "acceptsPointerMove",
					type: {
						kind: "scalar",
						name: "bool"
					}
				},
				{
					id: 13,
					name: "observesLayout",
					type: {
						kind: "scalar",
						name: "bool"
					}
				}
			]
		},
		PatchMove: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "id",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 2,
					name: "parentId",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 3,
					name: "index",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				}
			]
		},
		PatchDelete: {
			kind: "message",
			fields: [{
				id: 1,
				name: "id",
				type: {
					kind: "scalar",
					name: "uint32"
				}
			}]
		},
		PatchOperation: {
			kind: "message",
			fields: [{
				id: 1,
				name: "operation",
				type: {
					kind: "def",
					name: "PatchOperationValue"
				}
			}]
		},
		WindowOpenOptions: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "kind",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 2,
					name: "resizable",
					type: {
						kind: "scalar",
						name: "bool"
					}
				},
				{
					id: 3,
					name: "minWidth",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 4,
					name: "minHeight",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				}
			]
		},
		NotificationActionDefinition: {
			kind: "message",
			fields: [{
				id: 1,
				name: "id",
				type: {
					kind: "scalar",
					name: "string"
				}
			}, {
				id: 2,
				name: "label",
				type: {
					kind: "scalar",
					name: "string"
				}
			}]
		},
		MenuDefinition: {
			kind: "message",
			fields: [{
				id: 1,
				name: "title",
				type: {
					kind: "scalar",
					name: "string"
				}
			}, {
				id: 2,
				name: "items",
				type: {
					kind: "array",
					element: {
						kind: "def",
						name: "MenuItem"
					}
				}
			}]
		},
		MenuItemValue: {
			kind: "union",
			branches: [
				{
					id: 1,
					type: "MenuSeparator"
				},
				{
					id: 2,
					type: "MenuAction"
				},
				{
					id: 3,
					type: "MenuSubmenu"
				}
			]
		},
		MenuSeparator: {
			kind: "message",
			fields: []
		},
		MenuAction: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "name",
					type: {
						kind: "scalar",
						name: "string"
					}
				},
				{
					id: 2,
					name: "disabled",
					type: {
						kind: "scalar",
						name: "bool"
					}
				},
				{
					id: 3,
					name: "checked",
					type: {
						kind: "scalar",
						name: "bool"
					}
				}
			]
		},
		MenuSubmenu: {
			kind: "message",
			fields: [{
				id: 1,
				name: "menu",
				type: {
					kind: "def",
					name: "MenuDefinition"
				}
			}]
		},
		MenuItem: {
			kind: "message",
			fields: [{
				id: 1,
				name: "value",
				type: {
					kind: "def",
					name: "MenuItemValue"
				}
			}]
		},
		KeybindingDefinition: {
			kind: "message",
			fields: [{
				id: 1,
				name: "keystrokes",
				type: {
					kind: "scalar",
					name: "string"
				}
			}, {
				id: 2,
				name: "actionName",
				type: {
					kind: "scalar",
					name: "string"
				}
			}]
		},
		CommandPayload: {
			kind: "union",
			branches: [
				{
					id: 1,
					type: "U32PairCommand"
				},
				{
					id: 2,
					type: "FloatCommand"
				},
				{
					id: 3,
					type: "TextCommand"
				},
				{
					id: 4,
					type: "StringPairCommand"
				},
				{
					id: 5,
					type: "OpenSurfaceCommand"
				},
				{
					id: 6,
					type: "FileDialogOpenCommand"
				},
				{
					id: 7,
					type: "NotificationCommand"
				},
				{
					id: 8,
					type: "MenusCommand"
				},
				{
					id: 9,
					type: "KeybindingsCommand"
				},
				{
					id: 10,
					type: "ClipboardImageCommand"
				},
				{
					id: 11,
					type: "CloseResolutionCommand"
				},
				{
					id: 12,
					type: "InvokeNativeCommand"
				},
				{
					id: 13,
					type: "CancelNativeCommand"
				},
				{
					id: 14,
					type: "ConfigureApplicationCommand"
				},
				{
					id: 15,
					type: "OpenPopupCommand"
				},
				{
					id: 16,
					type: "ClosePopupCommand"
				}
			]
		},
		U32PairCommand: {
			kind: "message",
			fields: [{
				id: 1,
				name: "first",
				type: {
					kind: "scalar",
					name: "uint32"
				}
			}, {
				id: 2,
				name: "second",
				type: {
					kind: "scalar",
					name: "uint32"
				}
			}]
		},
		FloatCommand: {
			kind: "message",
			fields: [{
				id: 1,
				name: "value",
				type: {
					kind: "scalar",
					name: "float32"
				}
			}]
		},
		TextCommand: {
			kind: "message",
			fields: [{
				id: 1,
				name: "value",
				type: {
					kind: "scalar",
					name: "string"
				}
			}]
		},
		StringPairCommand: {
			kind: "message",
			fields: [{
				id: 1,
				name: "path",
				type: {
					kind: "scalar",
					name: "string"
				}
			}, {
				id: 2,
				name: "content",
				type: {
					kind: "scalar",
					name: "string"
				}
			}]
		},
		OpenSurfaceCommand: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "title",
					type: {
						kind: "scalar",
						name: "string"
					}
				},
				{
					id: 2,
					name: "width",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 3,
					name: "height",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 4,
					name: "options",
					type: {
						kind: "def",
						name: "WindowOpenOptions"
					}
				}
			]
		},
		FileDialogOpenCommand: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "title",
					type: {
						kind: "scalar",
						name: "string"
					}
				},
				{
					id: 2,
					name: "directories",
					type: {
						kind: "scalar",
						name: "bool"
					}
				},
				{
					id: 3,
					name: "multiple",
					type: {
						kind: "scalar",
						name: "bool"
					}
				}
			]
		},
		NotificationCommand: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "title",
					type: {
						kind: "scalar",
						name: "string"
					}
				},
				{
					id: 2,
					name: "body",
					type: {
						kind: "scalar",
						name: "string"
					}
				},
				{
					id: 3,
					name: "actions",
					type: {
						kind: "array",
						element: {
							kind: "def",
							name: "NotificationActionDefinition"
						}
					}
				}
			]
		},
		MenusCommand: {
			kind: "message",
			fields: [{
				id: 1,
				name: "menus",
				type: {
					kind: "array",
					element: {
						kind: "def",
						name: "MenuDefinition"
					}
				}
			}]
		},
		KeybindingsCommand: {
			kind: "message",
			fields: [{
				id: 1,
				name: "bindings",
				type: {
					kind: "array",
					element: {
						kind: "def",
						name: "KeybindingDefinition"
					}
				}
			}]
		},
		ClipboardImageCommand: {
			kind: "message",
			fields: [{
				id: 1,
				name: "format",
				type: {
					kind: "scalar",
					name: "uint32"
				}
			}, {
				id: 2,
				name: "bytes",
				type: {
					kind: "array",
					element: {
						kind: "scalar",
						name: "byte"
					}
				}
			}]
		},
		CloseResolutionCommand: {
			kind: "message",
			fields: [{
				id: 1,
				name: "requestId",
				type: {
					kind: "scalar",
					name: "uint32"
				}
			}, {
				id: 2,
				name: "allow",
				type: {
					kind: "scalar",
					name: "bool"
				}
			}]
		},
		InvokeNativeCommand: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "moduleId",
					type: {
						kind: "array",
						element: {
							kind: "scalar",
							name: "byte"
						}
					}
				},
				{
					id: 2,
					name: "moduleDigest",
					type: {
						kind: "array",
						element: {
							kind: "scalar",
							name: "byte"
						}
					}
				},
				{
					id: 3,
					name: "functionId",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 4,
					name: "args",
					type: {
						kind: "array",
						element: {
							kind: "scalar",
							name: "byte"
						}
					}
				}
			]
		},
		CancelNativeCommand: {
			kind: "message",
			fields: [{
				id: 1,
				name: "requestId",
				type: {
					kind: "scalar",
					name: "uint32"
				}
			}]
		},
		ConfigureApplicationCommand: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "keepAlive",
					type: {
						kind: "scalar",
						name: "bool"
					}
				},
				{
					id: 2,
					name: "quit",
					type: {
						kind: "scalar",
						name: "bool"
					}
				},
				{
					id: 3,
					name: "acknowledgedSequence",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				}
			]
		},
		OpenPopupCommand: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "anchorNodeId",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 2,
					name: "width",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 3,
					name: "height",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 4,
					name: "placement",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 5,
					name: "gap",
					type: {
						kind: "scalar",
						name: "float32"
					}
				}
			]
		},
		ClosePopupCommand: {
			kind: "message",
			fields: [{
				id: 1,
				name: "requestId",
				type: {
					kind: "scalar",
					name: "uint32"
				}
			}]
		},
		CommandValue: {
			kind: "union",
			branches: [
				{
					id: 1,
					type: "NumberValue"
				},
				{
					id: 2,
					type: "PairValue"
				},
				{
					id: 3,
					type: "BoolValue"
				},
				{
					id: 4,
					type: "TextValue"
				},
				{
					id: 5,
					type: "PathsValue"
				},
				{
					id: 6,
					type: "FileTextValue"
				},
				{
					id: 7,
					type: "ImageValue"
				},
				{
					id: 8,
					type: "BoundsValue"
				},
				{
					id: 9,
					type: "WindowStateValue"
				},
				{
					id: 10,
					type: "ScrollOffsetValue"
				},
				{
					id: 11,
					type: "BytesValue"
				}
			]
		},
		NumberValue: {
			kind: "message",
			fields: [{
				id: 1,
				name: "value",
				type: {
					kind: "scalar",
					name: "uint32"
				}
			}]
		},
		PairValue: {
			kind: "message",
			fields: [{
				id: 1,
				name: "width",
				type: {
					kind: "scalar",
					name: "float32"
				}
			}, {
				id: 2,
				name: "height",
				type: {
					kind: "scalar",
					name: "float32"
				}
			}]
		},
		BoolValue: {
			kind: "message",
			fields: [{
				id: 1,
				name: "value",
				type: {
					kind: "scalar",
					name: "bool"
				}
			}]
		},
		TextValue: {
			kind: "message",
			fields: [{
				id: 1,
				name: "value",
				type: {
					kind: "scalar",
					name: "string"
				}
			}]
		},
		PathsValue: {
			kind: "message",
			fields: [{
				id: 1,
				name: "paths",
				type: {
					kind: "array",
					element: {
						kind: "scalar",
						name: "string"
					}
				}
			}]
		},
		FileTextValue: {
			kind: "message",
			fields: [{
				id: 1,
				name: "value",
				type: {
					kind: "scalar",
					name: "string"
				}
			}]
		},
		ImageValue: {
			kind: "message",
			fields: [{
				id: 1,
				name: "format",
				type: {
					kind: "scalar",
					name: "uint32"
				}
			}, {
				id: 2,
				name: "bytes",
				type: {
					kind: "array",
					element: {
						kind: "scalar",
						name: "byte"
					}
				}
			}]
		},
		BoundsValue: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "x",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 2,
					name: "y",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 3,
					name: "width",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 4,
					name: "height",
					type: {
						kind: "scalar",
						name: "float32"
					}
				}
			]
		},
		WindowStateValue: {
			kind: "message",
			fields: [{
				id: 1,
				name: "fullscreen",
				type: {
					kind: "scalar",
					name: "bool"
				}
			}, {
				id: 2,
				name: "maximized",
				type: {
					kind: "scalar",
					name: "bool"
				}
			}]
		},
		ScrollOffsetValue: {
			kind: "message",
			fields: [{
				id: 1,
				name: "value",
				type: {
					kind: "scalar",
					name: "float32"
				}
			}]
		},
		BytesValue: {
			kind: "message",
			fields: [{
				id: 1,
				name: "value",
				type: {
					kind: "array",
					element: {
						kind: "scalar",
						name: "byte"
					}
				}
			}]
		},
		EventPayload: {
			kind: "union",
			branches: [
				{
					id: 1,
					type: "TextInputEventData"
				},
				{
					id: 2,
					type: "CommandResult"
				},
				{
					id: 3,
					type: "VisibleRangeEvent"
				},
				{
					id: 4,
					type: "AnimationCompleteEvent"
				},
				{
					id: 5,
					type: "KeyEvent"
				},
				{
					id: 6,
					type: "PointerEvent"
				},
				{
					id: 7,
					type: "PointerMoveEvent"
				},
				{
					id: 8,
					type: "ScrollEvent"
				},
				{
					id: 9,
					type: "SubmitEvent"
				},
				{
					id: 10,
					type: "WindowResizeEvent"
				},
				{
					id: 11,
					type: "WindowActivationEvent"
				},
				{
					id: 12,
					type: "ActionEvent"
				},
				{
					id: 13,
					type: "WindowAppearanceEvent"
				},
				{
					id: 14,
					type: "LayoutEvent"
				},
				{
					id: 15,
					type: "DragOverEvent"
				},
				{
					id: 16,
					type: "DragDropEvent"
				},
				{
					id: 17,
					type: "ExternalFileDropEvent"
				},
				{
					id: 18,
					type: "NotificationResponseEvent"
				},
				{
					id: 19,
					type: "PointerDownOutsideEvent"
				},
				{
					id: 20,
					type: "CloseRequestedEvent"
				},
				{
					id: 21,
					type: "ExtensionEvent"
				},
				{
					id: 22,
					type: "ApplicationActivationEvent"
				}
			]
		},
		TextInputEventData: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "text",
					type: {
						kind: "scalar",
						name: "string"
					}
				},
				{
					id: 2,
					name: "selectionStart",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 3,
					name: "selectionEnd",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 4,
					name: "markedStart",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 5,
					name: "markedEnd",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 6,
					name: "editSeq",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 7,
					name: "reversed",
					type: {
						kind: "scalar",
						name: "bool"
					}
				}
			]
		},
		CommandResult: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "requestId",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 2,
					name: "command",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 3,
					name: "nodeId",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 4,
					name: "success",
					type: {
						kind: "scalar",
						name: "bool"
					}
				},
				{
					id: 5,
					name: "error",
					type: {
						kind: "scalar",
						name: "string"
					}
				},
				{
					id: 6,
					name: "value",
					type: {
						kind: "def",
						name: "CommandValue"
					}
				}
			]
		},
		VisibleRangeEvent: {
			kind: "message",
			fields: [{
				id: 1,
				name: "start",
				type: {
					kind: "scalar",
					name: "uint32"
				}
			}, {
				id: 2,
				name: "end",
				type: {
					kind: "scalar",
					name: "uint32"
				}
			}]
		},
		AnimationCompleteEvent: {
			kind: "message",
			fields: [{
				id: 1,
				name: "generation",
				type: {
					kind: "scalar",
					name: "uint32"
				}
			}]
		},
		KeyEvent: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "key",
					type: {
						kind: "scalar",
						name: "string"
					}
				},
				{
					id: 2,
					name: "modifiers",
					type: {
						kind: "array",
						element: {
							kind: "scalar",
							name: "string"
						}
					}
				},
				{
					id: 3,
					name: "action",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				}
			]
		},
		PointerEvent: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "button",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 2,
					name: "modifiers",
					type: {
						kind: "array",
						element: {
							kind: "scalar",
							name: "string"
						}
					}
				},
				{
					id: 3,
					name: "action",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 4,
					name: "clickCount",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 5,
					name: "x",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 6,
					name: "y",
					type: {
						kind: "scalar",
						name: "float32"
					}
				}
			]
		},
		PointerMoveEvent: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "modifiers",
					type: {
						kind: "array",
						element: {
							kind: "scalar",
							name: "string"
						}
					}
				},
				{
					id: 2,
					name: "x",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 3,
					name: "y",
					type: {
						kind: "scalar",
						name: "float32"
					}
				}
			]
		},
		ScrollEvent: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "deltaKind",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 2,
					name: "dx",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 3,
					name: "dy",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 4,
					name: "x",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 5,
					name: "y",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 6,
					name: "modifiers",
					type: {
						kind: "array",
						element: {
							kind: "scalar",
							name: "string"
						}
					}
				}
			]
		},
		SubmitEvent: {
			kind: "message",
			fields: [{
				id: 1,
				name: "text",
				type: {
					kind: "scalar",
					name: "string"
				}
			}]
		},
		WindowResizeEvent: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "width",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 2,
					name: "height",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 3,
					name: "scaleFactor",
					type: {
						kind: "scalar",
						name: "float32"
					}
				}
			]
		},
		WindowActivationEvent: {
			kind: "message",
			fields: [{
				id: 1,
				name: "active",
				type: {
					kind: "scalar",
					name: "bool"
				}
			}]
		},
		ActionEvent: {
			kind: "message",
			fields: [{
				id: 1,
				name: "action",
				type: {
					kind: "scalar",
					name: "string"
				}
			}]
		},
		WindowAppearanceEvent: {
			kind: "message",
			fields: [{
				id: 1,
				name: "appearance",
				type: {
					kind: "def",
					name: "WindowAppearance"
				}
			}]
		},
		LayoutEvent: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "x",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 2,
					name: "y",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 3,
					name: "width",
					type: {
						kind: "scalar",
						name: "float32"
					}
				},
				{
					id: 4,
					name: "height",
					type: {
						kind: "scalar",
						name: "float32"
					}
				}
			]
		},
		DragOverEvent: {
			kind: "message",
			fields: [{
				id: 1,
				name: "dragType",
				type: {
					kind: "scalar",
					name: "string"
				}
			}]
		},
		DragDropEvent: {
			kind: "message",
			fields: [{
				id: 1,
				name: "dragType",
				type: {
					kind: "scalar",
					name: "string"
				}
			}]
		},
		ExternalFileDropEvent: {
			kind: "message",
			fields: [{
				id: 1,
				name: "paths",
				type: {
					kind: "array",
					element: {
						kind: "scalar",
						name: "string"
					}
				}
			}]
		},
		NotificationResponseEvent: {
			kind: "message",
			fields: [{
				id: 1,
				name: "tag",
				type: {
					kind: "scalar",
					name: "string"
				}
			}, {
				id: 2,
				name: "actionId",
				type: {
					kind: "scalar",
					name: "string"
				}
			}]
		},
		PointerDownOutsideEvent: {
			kind: "message",
			fields: [{
				id: 1,
				name: "x",
				type: {
					kind: "scalar",
					name: "float32"
				}
			}, {
				id: 2,
				name: "y",
				type: {
					kind: "scalar",
					name: "float32"
				}
			}]
		},
		CloseRequestedEvent: {
			kind: "message",
			fields: [{
				id: 1,
				name: "requestId",
				type: {
					kind: "scalar",
					name: "uint32"
				}
			}]
		},
		ExtensionEvent: {
			kind: "message",
			fields: [{
				id: 1,
				name: "eventId",
				type: {
					kind: "scalar",
					name: "uint32"
				}
			}, {
				id: 2,
				name: "fields",
				type: {
					kind: "array",
					element: {
						kind: "def",
						name: "ExtensionField"
					}
				}
			}]
		},
		ApplicationActivationEvent: {
			kind: "message",
			fields: [
				{
					id: 1,
					name: "targetSurfaceId",
					type: {
						kind: "scalar",
						name: "uint32"
					}
				},
				{
					id: 2,
					name: "reason",
					type: {
						kind: "scalar",
						name: "string"
					}
				},
				{
					id: 3,
					name: "urls",
					type: {
						kind: "array",
						element: {
							kind: "scalar",
							name: "string"
						}
					}
				}
			]
		},
		Envelope: {
			kind: "message",
			fields: [{
				id: 1,
				name: "protocolVersion",
				type: {
					kind: "scalar",
					name: "uint32"
				}
			}, {
				id: 2,
				name: "body",
				type: {
					kind: "def",
					name: "Body"
				}
			}]
		}
	}
};
//#endregion
//#region packages/solid-gpui/src/protocol/bebop-guard.ts
var DEFAULT_BEBOP_LIMITS = {
	maxFrameBytes: MAX_FRAME_SIZE,
	maxMessageBytes: MAX_FRAME_SIZE,
	maxArrayItems: 1e5,
	maxStringBytes: MAX_CLIPBOARD_TEXT_BYTES,
	maxBytes: MAX_CLIPBOARD_IMAGE_BYTES,
	maxNesting: 80,
	maxTotalItems: 1e6
};
var BoundedDecodeError = class extends Error {
	reason;
	offset;
	constructor(reason, offset) {
		super(`${reason} at byte ${offset}`);
		this.reason = reason;
		this.offset = offset;
		this.name = "BoundedDecodeError";
	}
};
var SCALAR_STRING = 0;
var SCALAR_BOOL = 1;
var SCALAR_BYTE = 2;
var SCALAR_UINT8 = 3;
var SCALAR_UINT16 = 4;
var SCALAR_INT16 = 5;
var SCALAR_UINT32 = 6;
var SCALAR_INT32 = 7;
var SCALAR_FLOAT32 = 8;
var SCALAR_UINT64 = 9;
var SCALAR_INT64 = 10;
var SCALAR_FLOAT64 = 11;
var SCALAR_GUID = 12;
var SCALAR_DATE = 13;
var SCALAR_SIZES = [
	0,
	1,
	1,
	1,
	2,
	2,
	4,
	4,
	4,
	8,
	8,
	8,
	16,
	8
];
var STRING_DEFAULT = 0;
var STRING_CONTENT = 1;
var STRING_FONT_FAMILY = 2;
var STRING_PATH = 3;
var STRING_DRAG_TYPE = 4;
var STRING_TAG = 5;
var STRING_NAMED = 6;
var STRING_FILE_TEXT = 7;
var STRING_IMAGE_SOURCE = 8;
var ARRAY_DEFAULT = 0;
var ARRAY_MODIFIERS = 1;
var ARRAY_ACTIONS = 2;
var ARRAY_VALUES = 3;
var ARRAY_ITEMS = 4;
var ARRAY_MENUS = 5;
var ARRAY_PATHS = 6;
var ARRAY_NODES = 7;
var ARRAY_OPERATIONS = 8;
var ARRAY_EXTENSION_FIELDS = 9;
var ARRAY_EXTENSION_EVENTS = 10;
function definitionMap(schema) {
	return schema.definitions;
}
var SCHEMA_DEFINITIONS = definitionMap(PROTOCOL_SCHEMA);
var DEFINITION_NAMES = Object.keys(SCHEMA_DEFINITIONS);
var DEFINITION_IDS = {};
for (let index = 0; index < DEFINITION_NAMES.length; index += 1) DEFINITION_IDS[DEFINITION_NAMES[index]] = index;
function scalarCode(name) {
	switch (name) {
		case "string": return SCALAR_STRING;
		case "bool": return SCALAR_BOOL;
		case "byte": return SCALAR_BYTE;
		case "uint8": return SCALAR_UINT8;
		case "uint16": return SCALAR_UINT16;
		case "int16": return SCALAR_INT16;
		case "uint32": return SCALAR_UINT32;
		case "int32": return SCALAR_INT32;
		case "float32": return SCALAR_FLOAT32;
		case "uint64": return SCALAR_UINT64;
		case "int64": return SCALAR_INT64;
		case "float64": return SCALAR_FLOAT64;
		case "guid": return SCALAR_GUID;
		case "date": return SCALAR_DATE;
		default: throw new Error(`unknown schema scalar ${name}`);
	}
}
function stringLimitKind(parent, field) {
	if (field === "content") return STRING_CONTENT;
	if (field === "fontFamily") return STRING_FONT_FAMILY;
	if (field === "path") return STRING_PATH;
	if (field === "source" || field === "fallbackSource") return STRING_IMAGE_SOURCE;
	if (field === "dragType") return STRING_DRAG_TYPE;
	if (field === "tag") return STRING_TAG;
	if (field === "title" || field === "action" || field === "name" || field === "label" || field === "key") return STRING_NAMED;
	if (parent === "FileTextValue") return STRING_FILE_TEXT;
	return STRING_DEFAULT;
}
function arrayLimitKind(field) {
	if (field === "modifiers") return ARRAY_MODIFIERS;
	if (field === "actions" || field === "bindings") return ARRAY_ACTIONS;
	if (field === "values") return ARRAY_VALUES;
	if (field === "items") return ARRAY_ITEMS;
	if (field === "menus") return ARRAY_MENUS;
	if (field === "paths" || field === "exportFiles") return ARRAY_PATHS;
	if (field === "nodes") return ARRAY_NODES;
	if (field === "operations") return ARRAY_OPERATIONS;
	if (field === "fields") return ARRAY_EXTENSION_FIELDS;
	if (field === "eventIds") return ARRAY_EXTENSION_EVENTS;
	return ARRAY_DEFAULT;
}
function compileType(type, parent, field) {
	if (type.kind === "scalar") return {
		kind: "scalar",
		code: scalarCode(type.name),
		stringLimit: stringLimitKind(parent, field)
	};
	if (type.kind === "array") {
		const element = compileType(type.element, parent, field);
		return {
			kind: "array",
			element,
			arrayLimit: arrayLimitKind(field),
			byteArray: element.kind === "scalar" && element.code === SCALAR_BYTE,
			minBytes: parent === "InvokeNativeCommand" && field === "moduleId" ? 16 : parent === "InvokeNativeCommand" && field === "moduleDigest" ? 32 : 0,
			maxBytes: parent === "InvokeNativeCommand" ? field === "moduleId" ? 16 : field === "moduleDigest" ? 32 : MAX_NATIVE_CALL_BYTES : parent === "BytesValue" ? MAX_NATIVE_CALL_BYTES : MAX_FRAME_SIZE
		};
	}
	const index = DEFINITION_IDS[type.name];
	if (index === void 0) throw new Error(`unknown schema definition ${type.name}`);
	return {
		kind: "definition",
		index
	};
}
function compileDefinition(name) {
	const definition = SCHEMA_DEFINITIONS[name];
	if (definition.kind === "enum") {
		const base = scalarCode(definition.base);
		return {
			kind: "enum",
			base,
			size: SCALAR_SIZES[base],
			values: definition.values ?? []
		};
	}
	if (definition.kind === "union") {
		const branches = [];
		for (const branch of definition.branches ?? []) {
			const index = DEFINITION_IDS[branch.type];
			if (index === void 0) throw new Error(`unknown union schema definition ${branch.type}`);
			branches[branch.id] = index;
		}
		return {
			kind: "union",
			branches
		};
	}
	const fields = [];
	for (const field of definition.fields ?? []) fields[field.id] = { type: compileType(field.type, name, field.name) };
	return {
		kind: "message",
		fields
	};
}
var COMPILED_DEFINITIONS = DEFINITION_NAMES.map(compileDefinition);
var ROOT_DEFINITION = DEFINITION_IDS[PROTOCOL_SCHEMA.root];
function validUtf8(data, offset, length) {
	const end = offset + length;
	while (offset < end) {
		const first = data[offset++];
		if (first <= 127) continue;
		if (first >= 194 && first <= 223) {
			if (offset >= end || (data[offset++] & 192) !== 128) return false;
			continue;
		}
		if (first >= 224 && first <= 239) {
			if (offset + 1 >= end) return false;
			const second = data[offset++];
			if ((second & 192) !== 128 || first === 224 && second < 160 || first === 237 && second >= 160) return false;
			if ((data[offset++] & 192) !== 128) return false;
			continue;
		}
		if (first >= 240 && first <= 244) {
			if (offset + 2 >= end) return false;
			const second = data[offset++];
			if ((second & 192) !== 128 || first === 240 && second < 144 || first === 244 && second >= 144) return false;
			if ((data[offset++] & 192) !== 128 || (data[offset++] & 192) !== 128) return false;
			continue;
		}
		return false;
	}
	return true;
}
var Guard = class {
	data;
	limits;
	offset = 0;
	depth = 0;
	totalItems = 0;
	bounds = [];
	view;
	constructor(data, limits) {
		this.data = data;
		this.limits = limits;
		this.view = new DataView(data.buffer, data.byteOffset, data.byteLength);
	}
	end() {
		return this.bounds[this.bounds.length - 1] ?? this.data.length;
	}
	fail(reason) {
		throw new BoundedDecodeError(reason, this.offset);
	}
	ensure(size) {
		if (!Number.isSafeInteger(size) || size < 0 || this.offset + size > this.end()) this.fail("remaining-byte budget exceeded");
	}
	u8() {
		this.ensure(1);
		return this.data[this.offset++];
	}
	u32() {
		this.ensure(4);
		const value = this.view.getUint32(this.offset, true);
		this.offset += 4;
		return value;
	}
	blob(max) {
		const lengthOffset = this.offset;
		const length = this.u32();
		if (length > max) throw new BoundedDecodeError(`string budget exceeded (${length} > ${max})`, lengthOffset);
		this.ensure(length);
		if (!validUtf8(this.data, this.offset, length)) throw new BoundedDecodeError("invalid UTF-8 string", this.offset);
		this.offset += length;
	}
	stringLimit(kind) {
		switch (kind) {
			case STRING_CONTENT: return MAX_FILE_WRITE_BYTES;
			case STRING_FONT_FAMILY: return 256;
			case STRING_PATH: return 1024;
			case STRING_IMAGE_SOURCE: return MAX_IMAGE_SOURCE_BYTES;
			case STRING_DRAG_TYPE: return 512;
			case STRING_TAG: return 1024;
			case STRING_NAMED: return 1024;
			case STRING_FILE_TEXT: return MAX_FILE_WRITE_BYTES;
			default: return this.limits.maxStringBytes;
		}
	}
	arrayLimit(kind) {
		switch (kind) {
			case ARRAY_MODIFIERS: return 5;
			case ARRAY_ACTIONS: return 64;
			case ARRAY_VALUES: return 2;
			case ARRAY_ITEMS: return 1024;
			case ARRAY_MENUS: return 64;
			case ARRAY_PATHS:
			case ARRAY_EXTENSION_FIELDS: return 256;
			case ARRAY_EXTENSION_EVENTS: return 256;
			case ARRAY_NODES:
			case ARRAY_OPERATIONS:
			case ARRAY_DEFAULT:
			default: return this.limits.maxArrayItems;
		}
	}
	scalar(code, stringLimit) {
		if (code === SCALAR_STRING) return this.blob(this.stringLimit(stringLimit));
		if (code === SCALAR_BOOL) {
			if (this.u8() > 1) throw new BoundedDecodeError("boolean must be 0 or 1", this.offset - 1);
			return;
		}
		const size = SCALAR_SIZES[code] ?? 0;
		if (size === 0) this.fail(`unknown scalar ${code}`);
		this.ensure(size);
		this.offset += size;
	}
	type(type) {
		if (type.kind === "scalar") return this.scalar(type.code, type.stringLimit);
		if (type.kind === "array") {
			const lengthOffset = this.offset;
			const length = this.u32();
			if (type.byteArray) {
				const limit = Math.min(this.limits.maxBytes, type.maxBytes);
				if (length < type.minBytes) throw new BoundedDecodeError(`byte length below minimum (${length} < ${type.minBytes})`, lengthOffset);
				if (length > limit) throw new BoundedDecodeError(`bytes budget exceeded (${length} > ${limit})`, lengthOffset);
				this.ensure(length);
				this.offset += length;
				return;
			}
			const limit = this.arrayLimit(type.arrayLimit);
			if (length > limit) throw new BoundedDecodeError(`array item budget exceeded (${length} > ${limit})`, lengthOffset);
			this.totalItems += length;
			if (this.totalItems > this.limits.maxTotalItems) throw new BoundedDecodeError("total array item budget exceeded", lengthOffset);
			for (let index = 0; index < length; index += 1) this.type(type.element);
			return;
		}
		this.definition(type.index);
	}
	enumValue(definition) {
		const valueOffset = this.offset;
		this.ensure(definition.size);
		const value = definition.size === 1 ? this.data[this.offset] : definition.base === SCALAR_UINT16 ? this.view.getUint16(this.offset, true) : definition.base === SCALAR_INT16 ? this.view.getInt16(this.offset, true) : definition.base === SCALAR_INT32 ? this.view.getInt32(this.offset, true) : this.view.getUint32(this.offset, true);
		this.offset += definition.size;
		if (!definition.values.includes(value)) throw new BoundedDecodeError(`unknown enum value ${value}`, valueOffset);
	}
	definition(index) {
		this.depth += 1;
		if (this.depth > this.limits.maxNesting) this.fail(`nesting budget exceeded (${this.limits.maxNesting})`);
		const definition = COMPILED_DEFINITIONS[index];
		if (definition === void 0) this.fail(`unknown definition ${index}`);
		if (definition.kind === "enum") {
			this.enumValue(definition);
			this.depth -= 1;
			return;
		}
		const lengthOffset = this.offset;
		const length = this.u32();
		if (length > this.limits.maxMessageBytes) throw new BoundedDecodeError(`message budget exceeded (${length} > ${this.limits.maxMessageBytes})`, lengthOffset);
		const union = definition.kind === "union";
		const contentLength = union ? length + 1 : length;
		if (!Number.isSafeInteger(contentLength)) this.fail("message length overflow");
		const end = this.offset + contentLength;
		this.ensure(contentLength);
		this.bounds.push(end);
		if (union) {
			const tagOffset = this.offset;
			const tag = this.u8();
			const branch = definition.branches[tag];
			if (branch === void 0) throw new BoundedDecodeError(`unknown union discriminator ${tag}`, tagOffset);
			this.definition(branch);
			if (this.offset !== end) this.fail("union payload has trailing bytes");
			this.bounds.pop();
			this.depth -= 1;
			return;
		}
		let last = 0;
		let terminated = false;
		while (this.offset < end) {
			const idOffset = this.offset;
			const id = this.u8();
			if (id === 0) {
				terminated = true;
				if (this.offset !== end) throw new BoundedDecodeError("message bytes after terminator", this.offset);
				break;
			}
			if (id <= last) throw new BoundedDecodeError(id === last ? "duplicate message field" : "message fields out of order", idOffset);
			last = id;
			const field = definition.fields[id];
			if (field === void 0) throw new BoundedDecodeError(`unknown message field ${id}`, idOffset);
			this.type(field.type);
		}
		if (!terminated || this.offset !== end) this.fail("message terminator or consumed length missing");
		this.bounds.pop();
		this.depth -= 1;
	}
	run() {
		if (this.data.byteLength > this.limits.maxFrameBytes) throw new BoundedDecodeError(`frame budget exceeded (${this.data.byteLength} > ${this.limits.maxFrameBytes})`, 0);
		this.definition(ROOT_DEFINITION);
		if (this.offset !== this.data.byteLength) this.fail("trailing bytes");
	}
};
function boundedBebopDecode(payload, limits = DEFAULT_BEBOP_LIMITS) {
	new Guard(payload, limits).run();
}
new Uint8Array([
	3,
	102,
	0,
	0,
	0,
	78,
	111,
	100,
	101,
	75,
	105,
	110,
	100,
	0,
	4,
	0,
	254,
	255,
	255,
	255,
	0,
	1,
	0,
	0,
	0,
	10,
	85,
	110,
	115,
	112,
	101,
	99,
	105,
	102,
	105,
	101,
	100,
	0,
	0,
	0,
	86,
	105,
	101,
	119,
	0,
	0,
	1,
	84,
	101,
	120,
	116,
	0,
	0,
	2,
	80,
	114,
	101,
	115,
	115,
	97,
	98,
	108,
	101,
	0,
	0,
	3,
	82,
	97,
	119,
	84,
	101,
	120,
	116,
	0,
	0,
	4,
	84,
	101,
	120,
	116,
	73,
	110,
	112,
	117,
	116,
	0,
	0,
	5,
	86,
	105,
	114,
	116,
	117,
	97,
	108,
	76,
	105,
	115,
	116,
	0,
	0,
	6,
	73,
	109,
	97,
	103,
	101,
	0,
	0,
	7,
	69,
	120,
	116,
	101,
	110,
	115,
	105,
	111,
	110,
	0,
	0,
	8,
	73,
	99,
	111,
	110,
	0,
	0,
	9,
	87,
	105,
	110,
	100,
	111,
	119,
	65,
	112,
	112,
	101,
	97,
	114,
	97,
	110,
	99,
	101,
	0,
	4,
	0,
	254,
	255,
	255,
	255,
	0,
	1,
	0,
	0,
	0,
	3,
	85,
	110,
	115,
	112,
	101,
	99,
	105,
	102,
	105,
	101,
	100,
	0,
	0,
	0,
	76,
	105,
	103,
	104,
	116,
	0,
	0,
	1,
	68,
	97,
	114,
	107,
	0,
	0,
	2,
	69,
	118,
	101,
	110,
	116,
	75,
	105,
	110,
	100,
	0,
	4,
	0,
	254,
	255,
	255,
	255,
	0,
	1,
	0,
	0,
	0,
	26,
	85,
	110,
	107,
	110,
	111,
	119,
	110,
	0,
	0,
	0,
	80,
	114,
	101,
	115,
	115,
	0,
	0,
	1,
	67,
	104,
	97,
	110,
	103,
	101,
	0,
	0,
	2,
	83,
	101,
	108,
	101,
	99,
	116,
	105,
	111,
	110,
	0,
	0,
	3,
	70,
	111,
	99,
	117,
	115,
	0,
	0,
	4,
	66,
	108,
	117,
	114,
	0,
	0,
	5,
	67,
	111,
	109,
	109,
	97,
	110,
	100,
	82,
	101,
	115,
	117,
	108,
	116,
	0,
	0,
	6,
	86,
	105,
	115,
	105,
	98,
	108,
	101,
	82,
	97,
	110,
	103,
	101,
	0,
	0,
	7,
	65,
	110,
	105,
	109,
	97,
	116,
	105,
	111,
	110,
	67,
	111,
	109,
	112,
	108,
	101,
	116,
	101,
	0,
	0,
	8,
	75,
	101,
	121,
	0,
	0,
	9,
	80,
	111,
	105,
	110,
	116,
	101,
	114,
	0,
	0,
	10,
	72,
	111,
	118,
	101,
	114,
	0,
	0,
	11,
	83,
	99,
	114,
	111,
	108,
	108,
	0,
	0,
	12,
	83,
	117,
	98,
	109,
	105,
	116,
	0,
	0,
	13,
	87,
	105,
	110,
	100,
	111,
	119,
	82,
	101,
	115,
	105,
	122,
	101,
	0,
	0,
	14,
	87,
	105,
	110,
	100,
	111,
	119,
	65,
	99,
	116,
	105,
	118,
	97,
	116,
	105,
	111,
	110,
	0,
	0,
	15,
	83,
	117,
	114,
	102,
	97,
	99,
	101,
	67,
	108,
	111,
	115,
	101,
	100,
	0,
	0,
	16,
	65,
	99,
	116,
	105,
	111,
	110,
	0,
	0,
	17,
	87,
	105,
	110,
	100,
	111,
	119,
	65,
	112,
	112,
	101,
	97,
	114,
	97,
	110,
	99,
	101,
	0,
	0,
	18,
	76,
	97,
	121,
	111,
	117,
	116,
	0,
	0,
	19,
	68,
	114,
	97,
	103,
	0,
	0,
	20,
	78,
	111,
	116,
	105,
	102,
	105,
	99,
	97,
	116,
	105,
	111,
	110,
	82,
	101,
	115,
	112,
	111,
	110,
	115,
	101,
	0,
	0,
	21,
	80,
	111,
	105,
	110,
	116,
	101,
	114,
	68,
	111,
	119,
	110,
	79,
	117,
	116,
	115,
	105,
	100,
	101,
	0,
	0,
	22,
	67,
	108,
	111,
	115,
	101,
	82,
	101,
	113,
	117,
	101,
	115,
	116,
	101,
	100,
	0,
	0,
	23,
	69,
	120,
	116,
	101,
	110,
	115,
	105,
	111,
	110,
	0,
	0,
	24,
	65,
	112,
	112,
	108,
	105,
	99,
	97,
	116,
	105,
	111,
	110,
	65,
	99,
	116,
	105,
	118,
	97,
	116,
	105,
	111,
	110,
	0,
	0,
	25,
	67,
	111,
	109,
	109,
	97,
	110,
	100,
	75,
	105,
	110,
	100,
	0,
	4,
	0,
	254,
	255,
	255,
	255,
	0,
	1,
	0,
	0,
	0,
	41,
	85,
	110,
	107,
	110,
	111,
	119,
	110,
	0,
	0,
	0,
	70,
	111,
	99,
	117,
	115,
	0,
	0,
	1,
	66,
	108,
	117,
	114,
	0,
	0,
	2,
	83,
	101,
	116,
	83,
	101,
	108,
	101,
	99,
	116,
	105,
	111,
	110,
	0,
	0,
	3,
	83,
	99,
	114,
	111,
	108,
	108,
	84,
	111,
	73,
	110,
	100,
	101,
	120,
	0,
	0,
	4,
	83,
	99,
	114,
	111,
	108,
	108,
	84,
	111,
	69,
	110,
	100,
	0,
	0,
	5,
	83,
	101,
	116,
	84,
	105,
	116,
	108,
	101,
	0,
	0,
	6,
	82,
	101,
	115,
	105,
	122,
	101,
	87,
	105,
	110,
	100,
	111,
	119,
	0,
	0,
	7,
	90,
	111,
	111,
	109,
	87,
	105,
	110,
	100,
	111,
	119,
	0,
	0,
	8,
	84,
	111,
	103,
	103,
	108,
	101,
	70,
	117,
	108,
	108,
	115,
	99,
	114,
	101,
	101,
	110,
	0,
	0,
	9,
	79,
	112,
	101,
	110,
	85,
	114,
	108,
	0,
	0,
	10,
	70,
	111,
	99,
	117,
	115,
	78,
	101,
	120,
	116,
	0,
	0,
	11,
	70,
	111,
	99,
	117,
	115,
	80,
	114,
	101,
	118,
	0,
	0,
	12,
	71,
	101,
	116,
	87,
	105,
	110,
	100,
	111,
	119,
	83,
	105,
	122,
	101,
	0,
	0,
	13,
	71,
	101,
	116,
	70,
	111,
	99,
	117,
	115,
	0,
	0,
	14,
	67,
	108,
	105,
	112,
	98,
	111,
	97,
	114,
	100,
	87,
	114,
	105,
	116,
	101,
	0,
	0,
	15,
	67,
	108,
	105,
	112,
	98,
	111,
	97,
	114,
	100,
	82,
	101,
	97,
	100,
	0,
	0,
	16,
	79,
	112,
	101,
	110,
	83,
	117,
	114,
	102,
	97,
	99,
	101,
	0,
	0,
	17,
	70,
	105,
	108,
	101,
	68,
	105,
	97,
	108,
	111,
	103,
	79,
	112,
	101,
	110,
	0,
	0,
	18,
	70,
	105,
	108,
	101,
	68,
	105,
	97,
	108,
	111,
	103,
	83,
	97,
	118,
	101,
	0,
	0,
	19,
	83,
	104,
	111,
	119,
	78,
	111,
	116,
	105,
	102,
	105,
	99,
	97,
	116,
	105,
	111,
	110,
	0,
	0,
	20,
	83,
	101,
	116,
	77,
	101,
	110,
	117,
	115,
	0,
	0,
	21,
	83,
	101,
	116,
	75,
	101,
	121,
	98,
	105,
	110,
	100,
	105,
	110,
	103,
	115,
	0,
	0,
	22,
	83,
	101,
	116,
	67,
	108,
	111,
	115,
	101,
	80,
	111,
	108,
	105,
	99,
	121,
	0,
	0,
	23,
	82,
	101,
	115,
	111,
	108,
	118,
	101,
	67,
	108,
	111,
	115,
	101,
	82,
	101,
	113,
	117,
	101,
	115,
	116,
	0,
	0,
	24,
	82,
	101,
	97,
	100,
	84,
	101,
	120,
	116,
	70,
	105,
	108,
	101,
	0,
	0,
	25,
	87,
	114,
	105,
	116,
	101,
	84,
	101,
	120,
	116,
	70,
	105,
	108,
	101,
	0,
	0,
	26,
	67,
	108,
	105,
	112,
	98,
	111,
	97,
	114,
	100,
	87,
	114,
	105,
	116,
	101,
	73,
	109,
	97,
	103,
	101,
	0,
	0,
	27,
	67,
	108,
	105,
	112,
	98,
	111,
	97,
	114,
	100,
	82,
	101,
	97,
	100,
	73,
	109,
	97,
	103,
	101,
	0,
	0,
	28,
	76,
	111,
	97,
	100,
	70,
	111,
	110,
	116,
	0,
	0,
	29,
	77,
	105,
	110,
	105,
	109,
	105,
	122,
	101,
	87,
	105,
	110,
	100,
	111,
	119,
	0,
	0,
	30,
	71,
	101,
	116,
	87,
	105,
	110,
	100,
	111,
	119,
	66,
	111,
	117,
	110,
	100,
	115,
	0,
	0,
	31,
	71,
	101,
	116,
	87,
	105,
	110,
	100,
	111,
	119,
	83,
	116,
	97,
	116,
	101,
	0,
	0,
	32,
	65,
	99,
	116,
	105,
	118,
	97,
	116,
	101,
	87,
	105,
	110,
	100,
	111,
	119,
	0,
	0,
	33,
	71,
	101,
	116,
	83,
	99,
	114,
	111,
	108,
	108,
	79,
	102,
	102,
	115,
	101,
	116,
	0,
	0,
	34,
	83,
	99,
	114,
	111,
	108,
	108,
	84,
	111,
	79,
	102,
	102,
	115,
	101,
	116,
	0,
	0,
	35,
	73,
	110,
	118,
	111,
	107,
	101,
	78,
	97,
	116,
	105,
	118,
	101,
	0,
	0,
	36,
	67,
	97,
	110,
	99,
	101,
	108,
	78,
	97,
	116,
	105,
	118,
	101,
	0,
	0,
	37,
	67,
	111,
	110,
	102,
	105,
	103,
	117,
	114,
	101,
	65,
	112,
	112,
	108,
	105,
	99,
	97,
	116,
	105,
	111,
	110,
	0,
	0,
	38,
	79,
	112,
	101,
	110,
	80,
	111,
	112,
	117,
	112,
	0,
	0,
	39,
	67,
	108,
	111,
	115,
	101,
	80,
	111,
	112,
	117,
	112,
	0,
	0,
	40,
	83,
	110,
	97,
	112,
	115,
	104,
	111,
	116,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	5,
	115,
	117,
	114,
	102,
	97,
	99,
	101,
	73,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	1,
	101,
	112,
	111,
	99,
	104,
	0,
	251,
	255,
	255,
	255,
	0,
	2,
	98,
	97,
	115,
	101,
	82,
	101,
	118,
	105,
	115,
	105,
	111,
	110,
	0,
	251,
	255,
	255,
	255,
	0,
	3,
	114,
	101,
	118,
	105,
	115,
	105,
	111,
	110,
	0,
	251,
	255,
	255,
	255,
	0,
	4,
	110,
	111,
	100,
	101,
	115,
	0,
	242,
	255,
	255,
	255,
	0,
	32,
	0,
	0,
	0,
	0,
	5,
	69,
	118,
	101,
	110,
	116,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	8,
	115,
	117,
	114,
	102,
	97,
	99,
	101,
	73,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	1,
	101,
	112,
	111,
	99,
	104,
	0,
	251,
	255,
	255,
	255,
	0,
	2,
	114,
	101,
	118,
	105,
	115,
	105,
	111,
	110,
	0,
	251,
	255,
	255,
	255,
	0,
	3,
	115,
	101,
	113,
	117,
	101,
	110,
	99,
	101,
	0,
	251,
	255,
	255,
	255,
	0,
	4,
	110,
	111,
	100,
	101,
	73,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	5,
	108,
	105,
	115,
	116,
	101,
	110,
	101,
	114,
	73,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	6,
	101,
	118,
	101,
	110,
	116,
	84,
	121,
	112,
	101,
	0,
	2,
	0,
	0,
	0,
	0,
	7,
	112,
	97,
	121,
	108,
	111,
	97,
	100,
	0,
	100,
	0,
	0,
	0,
	0,
	8,
	80,
	97,
	116,
	99,
	104,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	5,
	115,
	117,
	114,
	102,
	97,
	99,
	101,
	73,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	1,
	101,
	112,
	111,
	99,
	104,
	0,
	251,
	255,
	255,
	255,
	0,
	2,
	98,
	97,
	115,
	101,
	82,
	101,
	118,
	105,
	115,
	105,
	111,
	110,
	0,
	251,
	255,
	255,
	255,
	0,
	3,
	114,
	101,
	118,
	105,
	115,
	105,
	111,
	110,
	0,
	251,
	255,
	255,
	255,
	0,
	4,
	111,
	112,
	101,
	114,
	97,
	116,
	105,
	111,
	110,
	115,
	0,
	242,
	255,
	255,
	255,
	0,
	39,
	0,
	0,
	0,
	0,
	5,
	67,
	111,
	109,
	109,
	97,
	110,
	100,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	7,
	115,
	117,
	114,
	102,
	97,
	99,
	101,
	73,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	1,
	101,
	112,
	111,
	99,
	104,
	0,
	251,
	255,
	255,
	255,
	0,
	2,
	97,
	102,
	116,
	101,
	114,
	82,
	101,
	118,
	105,
	115,
	105,
	111,
	110,
	0,
	251,
	255,
	255,
	255,
	0,
	3,
	114,
	101,
	113,
	117,
	101,
	115,
	116,
	73,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	4,
	110,
	111,
	100,
	101,
	73,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	5,
	107,
	105,
	110,
	100,
	0,
	3,
	0,
	0,
	0,
	0,
	6,
	112,
	97,
	121,
	108,
	111,
	97,
	100,
	0,
	65,
	0,
	0,
	0,
	0,
	7,
	66,
	111,
	100,
	121,
	0,
	3,
	0,
	10,
	0,
	0,
	0,
	4,
	1,
	4,
	0,
	0,
	0,
	2,
	5,
	0,
	0,
	0,
	3,
	6,
	0,
	0,
	0,
	4,
	7,
	0,
	0,
	0,
	69,
	120,
	116,
	101,
	110,
	115,
	105,
	111,
	110,
	66,
	111,
	111,
	108,
	86,
	97,
	108,
	117,
	101,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	118,
	97,
	108,
	117,
	101,
	0,
	255,
	255,
	255,
	255,
	0,
	1,
	69,
	120,
	116,
	101,
	110,
	115,
	105,
	111,
	110,
	73,
	110,
	116,
	51,
	50,
	86,
	97,
	108,
	117,
	101,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	118,
	97,
	108,
	117,
	101,
	0,
	250,
	255,
	255,
	255,
	0,
	1,
	69,
	120,
	116,
	101,
	110,
	115,
	105,
	111,
	110,
	85,
	51,
	50,
	86,
	97,
	108,
	117,
	101,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	118,
	97,
	108,
	117,
	101,
	0,
	251,
	255,
	255,
	255,
	0,
	1,
	69,
	120,
	116,
	101,
	110,
	115,
	105,
	111,
	110,
	70,
	51,
	50,
	86,
	97,
	108,
	117,
	101,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	118,
	97,
	108,
	117,
	101,
	0,
	247,
	255,
	255,
	255,
	0,
	1,
	69,
	120,
	116,
	101,
	110,
	115,
	105,
	111,
	110,
	84,
	101,
	120,
	116,
	86,
	97,
	108,
	117,
	101,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	118,
	97,
	108,
	117,
	101,
	0,
	245,
	255,
	255,
	255,
	0,
	1,
	69,
	120,
	116,
	101,
	110,
	115,
	105,
	111,
	110,
	66,
	121,
	116,
	101,
	115,
	86,
	97,
	108,
	117,
	101,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	118,
	97,
	108,
	117,
	101,
	0,
	242,
	255,
	255,
	255,
	0,
	254,
	255,
	255,
	255,
	0,
	1,
	69,
	120,
	116,
	101,
	110,
	115,
	105,
	111,
	110,
	86,
	97,
	108,
	117,
	101,
	0,
	3,
	0,
	10,
	0,
	0,
	0,
	6,
	1,
	9,
	0,
	0,
	0,
	2,
	10,
	0,
	0,
	0,
	3,
	11,
	0,
	0,
	0,
	4,
	12,
	0,
	0,
	0,
	5,
	13,
	0,
	0,
	0,
	6,
	14,
	0,
	0,
	0,
	69,
	120,
	116,
	101,
	110,
	115,
	105,
	111,
	110,
	70,
	105,
	101,
	108,
	100,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	2,
	105,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	1,
	118,
	97,
	108,
	117,
	101,
	0,
	15,
	0,
	0,
	0,
	0,
	2,
	84,
	101,
	120,
	116,
	73,
	110,
	112,
	117,
	116,
	80,
	114,
	111,
	112,
	101,
	114,
	116,
	105,
	101,
	115,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	12,
	118,
	97,
	108,
	117,
	101,
	0,
	245,
	255,
	255,
	255,
	0,
	1,
	112,
	108,
	97,
	99,
	101,
	104,
	111,
	108,
	100,
	101,
	114,
	0,
	245,
	255,
	255,
	255,
	0,
	2,
	109,
	117,
	108,
	116,
	105,
	108,
	105,
	110,
	101,
	0,
	255,
	255,
	255,
	255,
	0,
	3,
	100,
	105,
	115,
	97,
	98,
	108,
	101,
	100,
	0,
	255,
	255,
	255,
	255,
	0,
	4,
	99,
	111,
	110,
	116,
	114,
	111,
	108,
	108,
	101,
	100,
	0,
	255,
	255,
	255,
	255,
	0,
	5,
	97,
	99,
	107,
	69,
	100,
	105,
	116,
	83,
	101,
	113,
	0,
	251,
	255,
	255,
	255,
	0,
	6,
	115,
	101,
	108,
	101,
	99,
	116,
	105,
	111,
	110,
	83,
	116,
	97,
	114,
	116,
	0,
	251,
	255,
	255,
	255,
	0,
	7,
	115,
	101,
	108,
	101,
	99,
	116,
	105,
	111,
	110,
	69,
	110,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	8,
	109,
	97,
	114,
	107,
	101,
	100,
	83,
	116,
	97,
	114,
	116,
	0,
	251,
	255,
	255,
	255,
	0,
	9,
	109,
	97,
	114,
	107,
	101,
	100,
	69,
	110,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	10,
	109,
	97,
	120,
	76,
	101,
	110,
	103,
	116,
	104,
	0,
	251,
	255,
	255,
	255,
	0,
	11,
	115,
	101,
	108,
	101,
	99,
	116,
	105,
	111,
	110,
	82,
	101,
	118,
	101,
	114,
	115,
	101,
	100,
	0,
	255,
	255,
	255,
	255,
	0,
	12,
	86,
	105,
	114,
	116,
	117,
	97,
	108,
	76,
	105,
	115,
	116,
	80,
	114,
	111,
	112,
	101,
	114,
	116,
	105,
	101,
	115,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	7,
	105,
	116,
	101,
	109,
	67,
	111,
	117,
	110,
	116,
	0,
	251,
	255,
	255,
	255,
	0,
	1,
	114,
	97,
	110,
	103,
	101,
	83,
	116,
	97,
	114,
	116,
	0,
	251,
	255,
	255,
	255,
	0,
	2,
	114,
	97,
	110,
	103,
	101,
	69,
	110,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	3,
	101,
	115,
	116,
	105,
	109,
	97,
	116,
	101,
	100,
	73,
	116,
	101,
	109,
	83,
	105,
	122,
	101,
	0,
	247,
	255,
	255,
	255,
	0,
	4,
	111,
	118,
	101,
	114,
	115,
	99,
	97,
	110,
	0,
	251,
	255,
	255,
	255,
	0,
	5,
	100,
	97,
	116,
	97,
	82,
	101,
	118,
	105,
	115,
	105,
	111,
	110,
	0,
	251,
	255,
	255,
	255,
	0,
	6,
	100,
	97,
	116,
	97,
	69,
	100,
	105,
	116,
	0,
	25,
	0,
	0,
	0,
	0,
	7,
	73,
	109,
	97,
	103,
	101,
	80,
	114,
	111,
	112,
	101,
	114,
	116,
	105,
	101,
	115,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	4,
	115,
	111,
	117,
	114,
	99,
	101,
	0,
	245,
	255,
	255,
	255,
	0,
	1,
	111,
	98,
	106,
	101,
	99,
	116,
	70,
	105,
	116,
	0,
	251,
	255,
	255,
	255,
	0,
	2,
	102,
	97,
	108,
	108,
	98,
	97,
	99,
	107,
	83,
	111,
	117,
	114,
	99,
	101,
	0,
	245,
	255,
	255,
	255,
	0,
	3,
	115,
	111,
	117,
	114,
	99,
	101,
	115,
	0,
	242,
	255,
	255,
	255,
	0,
	24,
	0,
	0,
	0,
	0,
	4,
	68,
	114,
	97,
	103,
	80,
	114,
	111,
	112,
	101,
	114,
	116,
	105,
	101,
	115,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	4,
	100,
	114,
	97,
	103,
	84,
	121,
	112,
	101,
	0,
	245,
	255,
	255,
	255,
	0,
	1,
	101,
	120,
	112,
	111,
	114,
	116,
	70,
	105,
	108,
	101,
	115,
	0,
	242,
	255,
	255,
	255,
	0,
	245,
	255,
	255,
	255,
	0,
	2,
	97,
	99,
	99,
	101,
	112,
	116,
	115,
	68,
	114,
	97,
	103,
	79,
	118,
	101,
	114,
	0,
	255,
	255,
	255,
	255,
	0,
	3,
	97,
	99,
	99,
	101,
	112,
	116,
	115,
	68,
	114,
	111,
	112,
	0,
	255,
	255,
	255,
	255,
	0,
	4,
	69,
	120,
	116,
	101,
	110,
	115,
	105,
	111,
	110,
	80,
	114,
	111,
	112,
	101,
	114,
	116,
	105,
	101,
	115,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	6,
	112,
	114,
	111,
	118,
	105,
	100,
	101,
	114,
	73,
	100,
	0,
	242,
	255,
	255,
	255,
	0,
	254,
	255,
	255,
	255,
	0,
	1,
	99,
	97,
	116,
	97,
	108,
	111,
	103,
	68,
	105,
	103,
	101,
	115,
	116,
	0,
	242,
	255,
	255,
	255,
	0,
	254,
	255,
	255,
	255,
	0,
	2,
	101,
	110,
	116,
	114,
	121,
	73,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	3,
	101,
	110,
	116,
	114,
	121,
	86,
	101,
	114,
	115,
	105,
	111,
	110,
	0,
	251,
	255,
	255,
	255,
	0,
	4,
	102,
	105,
	101,
	108,
	100,
	115,
	0,
	242,
	255,
	255,
	255,
	0,
	16,
	0,
	0,
	0,
	0,
	5,
	101,
	118,
	101,
	110,
	116,
	73,
	100,
	115,
	0,
	242,
	255,
	255,
	255,
	0,
	251,
	255,
	255,
	255,
	0,
	6,
	73,
	99,
	111,
	110,
	80,
	114,
	111,
	112,
	101,
	114,
	116,
	105,
	101,
	115,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	3,
	110,
	97,
	109,
	101,
	0,
	245,
	255,
	255,
	255,
	0,
	1,
	115,
	105,
	122,
	101,
	0,
	247,
	255,
	255,
	255,
	0,
	2,
	99,
	111,
	108,
	111,
	114,
	0,
	251,
	255,
	255,
	255,
	0,
	3,
	72,
	111,
	115,
	116,
	80,
	114,
	111,
	112,
	101,
	114,
	116,
	105,
	101,
	115,
	0,
	3,
	0,
	10,
	0,
	0,
	0,
	6,
	1,
	17,
	0,
	0,
	0,
	2,
	18,
	0,
	0,
	0,
	3,
	19,
	0,
	0,
	0,
	4,
	20,
	0,
	0,
	0,
	5,
	21,
	0,
	0,
	0,
	6,
	22,
	0,
	0,
	0,
	73,
	109,
	97,
	103,
	101,
	67,
	97,
	110,
	100,
	105,
	100,
	97,
	116,
	101,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	3,
	115,
	111,
	117,
	114,
	99,
	101,
	0,
	245,
	255,
	255,
	255,
	0,
	1,
	119,
	105,
	100,
	116,
	104,
	0,
	251,
	255,
	255,
	255,
	0,
	2,
	104,
	101,
	105,
	103,
	104,
	116,
	0,
	251,
	255,
	255,
	255,
	0,
	3,
	86,
	105,
	114,
	116,
	117,
	97,
	108,
	76,
	105,
	115,
	116,
	68,
	97,
	116,
	97,
	69,
	100,
	105,
	116,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	4,
	98,
	97,
	115,
	101,
	82,
	101,
	118,
	105,
	115,
	105,
	111,
	110,
	0,
	251,
	255,
	255,
	255,
	0,
	1,
	115,
	116,
	97,
	114,
	116,
	0,
	251,
	255,
	255,
	255,
	0,
	2,
	111,
	108,
	100,
	67,
	111,
	117,
	110,
	116,
	0,
	251,
	255,
	255,
	255,
	0,
	3,
	110,
	101,
	119,
	67,
	111,
	117,
	110,
	116,
	0,
	251,
	255,
	255,
	255,
	0,
	4,
	65,
	99,
	99,
	101,
	115,
	115,
	105,
	98,
	105,
	108,
	105,
	116,
	121,
	80,
	114,
	111,
	112,
	101,
	114,
	116,
	105,
	101,
	115,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	10,
	114,
	111,
	108,
	101,
	0,
	251,
	255,
	255,
	255,
	0,
	1,
	108,
	97,
	98,
	101,
	108,
	0,
	245,
	255,
	255,
	255,
	0,
	2,
	100,
	101,
	115,
	99,
	114,
	105,
	112,
	116,
	105,
	111,
	110,
	0,
	245,
	255,
	255,
	255,
	0,
	3,
	100,
	105,
	115,
	97,
	98,
	108,
	101,
	100,
	0,
	255,
	255,
	255,
	255,
	0,
	4,
	99,
	104,
	101,
	99,
	107,
	101,
	100,
	0,
	255,
	255,
	255,
	255,
	0,
	5,
	115,
	101,
	108,
	101,
	99,
	116,
	101,
	100,
	0,
	255,
	255,
	255,
	255,
	0,
	6,
	118,
	97,
	108,
	117,
	101,
	0,
	245,
	255,
	255,
	255,
	0,
	7,
	101,
	120,
	112,
	97,
	110,
	100,
	101,
	100,
	0,
	255,
	255,
	255,
	255,
	0,
	8,
	108,
	101,
	118,
	101,
	108,
	0,
	251,
	255,
	255,
	255,
	0,
	9,
	108,
	105,
	118,
	101,
	0,
	251,
	255,
	255,
	255,
	0,
	10,
	84,
	114,
	97,
	110,
	115,
	105,
	116,
	105,
	111,
	110,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	4,
	100,
	117,
	114,
	97,
	116,
	105,
	111,
	110,
	77,
	115,
	0,
	251,
	255,
	255,
	255,
	0,
	1,
	100,
	101,
	108,
	97,
	121,
	77,
	115,
	0,
	251,
	255,
	255,
	255,
	0,
	2,
	101,
	97,
	115,
	105,
	110,
	103,
	0,
	251,
	255,
	255,
	255,
	0,
	3,
	112,
	114,
	111,
	112,
	101,
	114,
	116,
	121,
	77,
	97,
	115,
	107,
	0,
	251,
	255,
	255,
	255,
	0,
	4,
	66,
	111,
	120,
	83,
	104,
	97,
	100,
	111,
	119,
	86,
	97,
	108,
	117,
	101,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	6,
	111,
	102,
	102,
	115,
	101,
	116,
	88,
	0,
	247,
	255,
	255,
	255,
	0,
	1,
	111,
	102,
	102,
	115,
	101,
	116,
	89,
	0,
	247,
	255,
	255,
	255,
	0,
	2,
	98,
	108,
	117,
	114,
	82,
	97,
	100,
	105,
	117,
	115,
	0,
	247,
	255,
	255,
	255,
	0,
	3,
	115,
	112,
	114,
	101,
	97,
	100,
	82,
	97,
	100,
	105,
	117,
	115,
	0,
	247,
	255,
	255,
	255,
	0,
	4,
	99,
	111,
	108,
	111,
	114,
	0,
	251,
	255,
	255,
	255,
	0,
	5,
	105,
	110,
	115,
	101,
	116,
	0,
	255,
	255,
	255,
	255,
	0,
	6,
	66,
	111,
	120,
	83,
	104,
	97,
	100,
	111,
	119,
	83,
	101,
	116,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	118,
	97,
	108,
	117,
	101,
	115,
	0,
	242,
	255,
	255,
	255,
	0,
	28,
	0,
	0,
	0,
	0,
	1,
	76,
	105,
	110,
	101,
	97,
	114,
	71,
	114,
	97,
	100,
	105,
	101,
	110,
	116,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	5,
	97,
	110,
	103,
	108,
	101,
	0,
	247,
	255,
	255,
	255,
	0,
	1,
	115,
	116,
	97,
	114,
	116,
	67,
	111,
	108,
	111,
	114,
	0,
	251,
	255,
	255,
	255,
	0,
	2,
	115,
	116,
	97,
	114,
	116,
	80,
	111,
	115,
	105,
	116,
	105,
	111,
	110,
	0,
	247,
	255,
	255,
	255,
	0,
	3,
	101,
	110,
	100,
	67,
	111,
	108,
	111,
	114,
	0,
	251,
	255,
	255,
	255,
	0,
	4,
	101,
	110,
	100,
	80,
	111,
	115,
	105,
	116,
	105,
	111,
	110,
	0,
	247,
	255,
	255,
	255,
	0,
	5,
	83,
	116,
	121,
	108,
	101,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	66,
	119,
	105,
	100,
	116,
	104,
	0,
	247,
	255,
	255,
	255,
	0,
	1,
	104,
	101,
	105,
	103,
	104,
	116,
	0,
	247,
	255,
	255,
	255,
	0,
	2,
	102,
	108,
	101,
	120,
	68,
	105,
	114,
	101,
	99,
	116,
	105,
	111,
	110,
	0,
	251,
	255,
	255,
	255,
	0,
	3,
	102,
	108,
	101,
	120,
	71,
	114,
	111,
	119,
	0,
	247,
	255,
	255,
	255,
	0,
	4,
	112,
	97,
	100,
	100,
	105,
	110,
	103,
	0,
	247,
	255,
	255,
	255,
	0,
	5,
	103,
	97,
	112,
	0,
	247,
	255,
	255,
	255,
	0,
	6,
	98,
	97,
	99,
	107,
	103,
	114,
	111,
	117,
	110,
	100,
	67,
	111,
	108,
	111,
	114,
	0,
	251,
	255,
	255,
	255,
	0,
	7,
	99,
	111,
	108,
	111,
	114,
	0,
	251,
	255,
	255,
	255,
	0,
	8,
	111,
	112,
	97,
	99,
	105,
	116,
	121,
	0,
	247,
	255,
	255,
	255,
	0,
	9,
	116,
	114,
	97,
	110,
	115,
	105,
	116,
	105,
	111,
	110,
	0,
	27,
	0,
	0,
	0,
	0,
	10,
	106,
	117,
	115,
	116,
	105,
	102,
	121,
	67,
	111,
	110,
	116,
	101,
	110,
	116,
	0,
	251,
	255,
	255,
	255,
	0,
	11,
	97,
	108,
	105,
	103,
	110,
	73,
	116,
	101,
	109,
	115,
	0,
	251,
	255,
	255,
	255,
	0,
	12,
	98,
	111,
	114,
	100,
	101,
	114,
	82,
	97,
	100,
	105,
	117,
	115,
	0,
	247,
	255,
	255,
	255,
	0,
	13,
	98,
	111,
	114,
	100,
	101,
	114,
	87,
	105,
	100,
	116,
	104,
	0,
	247,
	255,
	255,
	255,
	0,
	14,
	98,
	111,
	114,
	100,
	101,
	114,
	67,
	111,
	108,
	111,
	114,
	0,
	251,
	255,
	255,
	255,
	0,
	15,
	102,
	111,
	110,
	116,
	83,
	105,
	122,
	101,
	0,
	247,
	255,
	255,
	255,
	0,
	16,
	102,
	111,
	110,
	116,
	87,
	101,
	105,
	103,
	104,
	116,
	0,
	251,
	255,
	255,
	255,
	0,
	17,
	111,
	118,
	101,
	114,
	102,
	108,
	111,
	119,
	0,
	251,
	255,
	255,
	255,
	0,
	18,
	108,
	105,
	110,
	101,
	67,
	108,
	97,
	109,
	112,
	0,
	251,
	255,
	255,
	255,
	0,
	19,
	116,
	101,
	120,
	116,
	79,
	118,
	101,
	114,
	102,
	108,
	111,
	119,
	0,
	251,
	255,
	255,
	255,
	0,
	20,
	109,
	97,
	114,
	103,
	105,
	110,
	84,
	111,
	112,
	0,
	247,
	255,
	255,
	255,
	0,
	21,
	109,
	97,
	114,
	103,
	105,
	110,
	82,
	105,
	103,
	104,
	116,
	0,
	247,
	255,
	255,
	255,
	0,
	22,
	109,
	97,
	114,
	103,
	105,
	110,
	66,
	111,
	116,
	116,
	111,
	109,
	0,
	247,
	255,
	255,
	255,
	0,
	23,
	109,
	97,
	114,
	103,
	105,
	110,
	76,
	101,
	102,
	116,
	0,
	247,
	255,
	255,
	255,
	0,
	24,
	102,
	111,
	110,
	116,
	83,
	116,
	121,
	108,
	101,
	0,
	251,
	255,
	255,
	255,
	0,
	25,
	116,
	101,
	120,
	116,
	68,
	101,
	99,
	111,
	114,
	97,
	116,
	105,
	111,
	110,
	0,
	251,
	255,
	255,
	255,
	0,
	26,
	108,
	105,
	110,
	101,
	72,
	101,
	105,
	103,
	104,
	116,
	0,
	247,
	255,
	255,
	255,
	0,
	27,
	109,
	105,
	110,
	87,
	105,
	100,
	116,
	104,
	0,
	247,
	255,
	255,
	255,
	0,
	28,
	109,
	97,
	120,
	87,
	105,
	100,
	116,
	104,
	0,
	247,
	255,
	255,
	255,
	0,
	29,
	109,
	105,
	110,
	72,
	101,
	105,
	103,
	104,
	116,
	0,
	247,
	255,
	255,
	255,
	0,
	30,
	109,
	97,
	120,
	72,
	101,
	105,
	103,
	104,
	116,
	0,
	247,
	255,
	255,
	255,
	0,
	31,
	102,
	108,
	101,
	120,
	83,
	104,
	114,
	105,
	110,
	107,
	0,
	247,
	255,
	255,
	255,
	0,
	32,
	97,
	108,
	105,
	103,
	110,
	83,
	101,
	108,
	102,
	0,
	251,
	255,
	255,
	255,
	0,
	33,
	112,
	111,
	115,
	105,
	116,
	105,
	111,
	110,
	0,
	251,
	255,
	255,
	255,
	0,
	34,
	108,
	101,
	102,
	116,
	0,
	247,
	255,
	255,
	255,
	0,
	35,
	116,
	111,
	112,
	0,
	247,
	255,
	255,
	255,
	0,
	36,
	114,
	105,
	103,
	104,
	116,
	0,
	247,
	255,
	255,
	255,
	0,
	37,
	98,
	111,
	116,
	116,
	111,
	109,
	0,
	247,
	255,
	255,
	255,
	0,
	38,
	99,
	117,
	114,
	115,
	111,
	114,
	0,
	251,
	255,
	255,
	255,
	0,
	39,
	116,
	101,
	120,
	116,
	65,
	108,
	105,
	103,
	110,
	0,
	251,
	255,
	255,
	255,
	0,
	40,
	98,
	111,
	120,
	83,
	104,
	97,
	100,
	111,
	119,
	0,
	29,
	0,
	0,
	0,
	0,
	41,
	102,
	111,
	110,
	116,
	70,
	97,
	109,
	105,
	108,
	121,
	0,
	245,
	255,
	255,
	255,
	0,
	42,
	112,
	97,
	100,
	100,
	105,
	110,
	103,
	84,
	111,
	112,
	0,
	247,
	255,
	255,
	255,
	0,
	43,
	112,
	97,
	100,
	100,
	105,
	110,
	103,
	82,
	105,
	103,
	104,
	116,
	0,
	247,
	255,
	255,
	255,
	0,
	44,
	112,
	97,
	100,
	100,
	105,
	110,
	103,
	66,
	111,
	116,
	116,
	111,
	109,
	0,
	247,
	255,
	255,
	255,
	0,
	45,
	112,
	97,
	100,
	100,
	105,
	110,
	103,
	76,
	101,
	102,
	116,
	0,
	247,
	255,
	255,
	255,
	0,
	46,
	98,
	111,
	114,
	100,
	101,
	114,
	84,
	111,
	112,
	87,
	105,
	100,
	116,
	104,
	0,
	247,
	255,
	255,
	255,
	0,
	47,
	98,
	111,
	114,
	100,
	101,
	114,
	82,
	105,
	103,
	104,
	116,
	87,
	105,
	100,
	116,
	104,
	0,
	247,
	255,
	255,
	255,
	0,
	48,
	98,
	111,
	114,
	100,
	101,
	114,
	66,
	111,
	116,
	116,
	111,
	109,
	87,
	105,
	100,
	116,
	104,
	0,
	247,
	255,
	255,
	255,
	0,
	49,
	98,
	111,
	114,
	100,
	101,
	114,
	76,
	101,
	102,
	116,
	87,
	105,
	100,
	116,
	104,
	0,
	247,
	255,
	255,
	255,
	0,
	50,
	98,
	111,
	114,
	100,
	101,
	114,
	84,
	111,
	112,
	76,
	101,
	102,
	116,
	82,
	97,
	100,
	105,
	117,
	115,
	0,
	247,
	255,
	255,
	255,
	0,
	51,
	98,
	111,
	114,
	100,
	101,
	114,
	84,
	111,
	112,
	82,
	105,
	103,
	104,
	116,
	82,
	97,
	100,
	105,
	117,
	115,
	0,
	247,
	255,
	255,
	255,
	0,
	52,
	98,
	111,
	114,
	100,
	101,
	114,
	66,
	111,
	116,
	116,
	111,
	109,
	82,
	105,
	103,
	104,
	116,
	82,
	97,
	100,
	105,
	117,
	115,
	0,
	247,
	255,
	255,
	255,
	0,
	53,
	98,
	111,
	114,
	100,
	101,
	114,
	66,
	111,
	116,
	116,
	111,
	109,
	76,
	101,
	102,
	116,
	82,
	97,
	100,
	105,
	117,
	115,
	0,
	247,
	255,
	255,
	255,
	0,
	54,
	119,
	105,
	100,
	116,
	104,
	80,
	101,
	114,
	99,
	101,
	110,
	116,
	0,
	247,
	255,
	255,
	255,
	0,
	55,
	104,
	101,
	105,
	103,
	104,
	116,
	80,
	101,
	114,
	99,
	101,
	110,
	116,
	0,
	247,
	255,
	255,
	255,
	0,
	56,
	102,
	108,
	101,
	120,
	87,
	114,
	97,
	112,
	0,
	251,
	255,
	255,
	255,
	0,
	57,
	108,
	105,
	110,
	101,
	97,
	114,
	71,
	114,
	97,
	100,
	105,
	101,
	110,
	116,
	0,
	30,
	0,
	0,
	0,
	0,
	58,
	98,
	111,
	114,
	100,
	101,
	114,
	84,
	111,
	112,
	67,
	111,
	108,
	111,
	114,
	0,
	251,
	255,
	255,
	255,
	0,
	59,
	98,
	111,
	114,
	100,
	101,
	114,
	82,
	105,
	103,
	104,
	116,
	67,
	111,
	108,
	111,
	114,
	0,
	251,
	255,
	255,
	255,
	0,
	60,
	98,
	111,
	114,
	100,
	101,
	114,
	66,
	111,
	116,
	116,
	111,
	109,
	67,
	111,
	108,
	111,
	114,
	0,
	251,
	255,
	255,
	255,
	0,
	61,
	98,
	111,
	114,
	100,
	101,
	114,
	76,
	101,
	102,
	116,
	67,
	111,
	108,
	111,
	114,
	0,
	251,
	255,
	255,
	255,
	0,
	62,
	103,
	114,
	105,
	100,
	67,
	111,
	108,
	117,
	109,
	110,
	115,
	0,
	251,
	255,
	255,
	255,
	0,
	63,
	103,
	114,
	105,
	100,
	82,
	111,
	119,
	115,
	0,
	251,
	255,
	255,
	255,
	0,
	64,
	103,
	114,
	105,
	100,
	67,
	111,
	108,
	117,
	109,
	110,
	83,
	112,
	97,
	110,
	0,
	251,
	255,
	255,
	255,
	0,
	65,
	103,
	114,
	105,
	100,
	82,
	111,
	119,
	83,
	112,
	97,
	110,
	0,
	251,
	255,
	255,
	255,
	0,
	66,
	78,
	111,
	100,
	101,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	14,
	105,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	1,
	112,
	97,
	114,
	101,
	110,
	116,
	73,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	2,
	105,
	110,
	100,
	101,
	120,
	0,
	251,
	255,
	255,
	255,
	0,
	3,
	107,
	105,
	110,
	100,
	0,
	0,
	0,
	0,
	0,
	0,
	4,
	115,
	116,
	121,
	108,
	101,
	0,
	31,
	0,
	0,
	0,
	0,
	5,
	116,
	101,
	120,
	116,
	0,
	245,
	255,
	255,
	255,
	0,
	6,
	108,
	105,
	115,
	116,
	101,
	110,
	101,
	114,
	73,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	7,
	104,
	111,
	115,
	116,
	80,
	114,
	111,
	112,
	101,
	114,
	116,
	105,
	101,
	115,
	0,
	23,
	0,
	0,
	0,
	0,
	8,
	97,
	99,
	99,
	101,
	115,
	115,
	105,
	98,
	105,
	108,
	105,
	116,
	121,
	0,
	26,
	0,
	0,
	0,
	0,
	9,
	102,
	111,
	99,
	117,
	115,
	97,
	98,
	108,
	101,
	0,
	255,
	255,
	255,
	255,
	0,
	10,
	115,
	101,
	108,
	101,
	99,
	116,
	97,
	98,
	108,
	101,
	0,
	255,
	255,
	255,
	255,
	0,
	11,
	116,
	111,
	111,
	108,
	116,
	105,
	112,
	0,
	245,
	255,
	255,
	255,
	0,
	12,
	97,
	99,
	99,
	101,
	112,
	116,
	115,
	80,
	111,
	105,
	110,
	116,
	101,
	114,
	77,
	111,
	118,
	101,
	0,
	255,
	255,
	255,
	255,
	0,
	13,
	111,
	98,
	115,
	101,
	114,
	118,
	101,
	115,
	76,
	97,
	121,
	111,
	117,
	116,
	0,
	255,
	255,
	255,
	255,
	0,
	14,
	67,
	108,
	101,
	97,
	114,
	83,
	116,
	121,
	108,
	101,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	0,
	80,
	97,
	116,
	99,
	104,
	67,
	114,
	101,
	97,
	116,
	101,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	110,
	111,
	100,
	101,
	0,
	32,
	0,
	0,
	0,
	0,
	1,
	80,
	97,
	116,
	99,
	104,
	85,
	112,
	100,
	97,
	116,
	101,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	13,
	105,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	1,
	109,
	97,
	115,
	107,
	0,
	251,
	255,
	255,
	255,
	0,
	2,
	115,
	116,
	121,
	108,
	101,
	0,
	31,
	0,
	0,
	0,
	0,
	3,
	99,
	108,
	101,
	97,
	114,
	83,
	116,
	121,
	108,
	101,
	0,
	33,
	0,
	0,
	0,
	0,
	4,
	116,
	101,
	120,
	116,
	0,
	245,
	255,
	255,
	255,
	0,
	5,
	108,
	105,
	115,
	116,
	101,
	110,
	101,
	114,
	73,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	6,
	104,
	111,
	115,
	116,
	80,
	114,
	111,
	112,
	101,
	114,
	116,
	105,
	101,
	115,
	0,
	23,
	0,
	0,
	0,
	0,
	7,
	97,
	99,
	99,
	101,
	115,
	115,
	105,
	98,
	105,
	108,
	105,
	116,
	121,
	0,
	26,
	0,
	0,
	0,
	0,
	8,
	102,
	111,
	99,
	117,
	115,
	97,
	98,
	108,
	101,
	0,
	255,
	255,
	255,
	255,
	0,
	9,
	115,
	101,
	108,
	101,
	99,
	116,
	97,
	98,
	108,
	101,
	0,
	255,
	255,
	255,
	255,
	0,
	10,
	116,
	111,
	111,
	108,
	116,
	105,
	112,
	0,
	245,
	255,
	255,
	255,
	0,
	11,
	97,
	99,
	99,
	101,
	112,
	116,
	115,
	80,
	111,
	105,
	110,
	116,
	101,
	114,
	77,
	111,
	118,
	101,
	0,
	255,
	255,
	255,
	255,
	0,
	12,
	111,
	98,
	115,
	101,
	114,
	118,
	101,
	115,
	76,
	97,
	121,
	111,
	117,
	116,
	0,
	255,
	255,
	255,
	255,
	0,
	13,
	80,
	97,
	116,
	99,
	104,
	77,
	111,
	118,
	101,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	3,
	105,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	1,
	112,
	97,
	114,
	101,
	110,
	116,
	73,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	2,
	105,
	110,
	100,
	101,
	120,
	0,
	251,
	255,
	255,
	255,
	0,
	3,
	80,
	97,
	116,
	99,
	104,
	68,
	101,
	108,
	101,
	116,
	101,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	105,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	1,
	80,
	97,
	116,
	99,
	104,
	79,
	112,
	101,
	114,
	97,
	116,
	105,
	111,
	110,
	86,
	97,
	108,
	117,
	101,
	0,
	3,
	0,
	10,
	0,
	0,
	0,
	4,
	1,
	34,
	0,
	0,
	0,
	2,
	35,
	0,
	0,
	0,
	3,
	36,
	0,
	0,
	0,
	4,
	37,
	0,
	0,
	0,
	80,
	97,
	116,
	99,
	104,
	79,
	112,
	101,
	114,
	97,
	116,
	105,
	111,
	110,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	111,
	112,
	101,
	114,
	97,
	116,
	105,
	111,
	110,
	0,
	38,
	0,
	0,
	0,
	0,
	1,
	87,
	105,
	110,
	100,
	111,
	119,
	79,
	112,
	101,
	110,
	79,
	112,
	116,
	105,
	111,
	110,
	115,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	4,
	107,
	105,
	110,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	1,
	114,
	101,
	115,
	105,
	122,
	97,
	98,
	108,
	101,
	0,
	255,
	255,
	255,
	255,
	0,
	2,
	109,
	105,
	110,
	87,
	105,
	100,
	116,
	104,
	0,
	251,
	255,
	255,
	255,
	0,
	3,
	109,
	105,
	110,
	72,
	101,
	105,
	103,
	104,
	116,
	0,
	251,
	255,
	255,
	255,
	0,
	4,
	78,
	111,
	116,
	105,
	102,
	105,
	99,
	97,
	116,
	105,
	111,
	110,
	65,
	99,
	116,
	105,
	111,
	110,
	68,
	101,
	102,
	105,
	110,
	105,
	116,
	105,
	111,
	110,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	2,
	105,
	100,
	0,
	245,
	255,
	255,
	255,
	0,
	1,
	108,
	97,
	98,
	101,
	108,
	0,
	245,
	255,
	255,
	255,
	0,
	2,
	77,
	101,
	110,
	117,
	68,
	101,
	102,
	105,
	110,
	105,
	116,
	105,
	111,
	110,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	2,
	116,
	105,
	116,
	108,
	101,
	0,
	245,
	255,
	255,
	255,
	0,
	1,
	105,
	116,
	101,
	109,
	115,
	0,
	242,
	255,
	255,
	255,
	0,
	47,
	0,
	0,
	0,
	0,
	2,
	77,
	101,
	110,
	117,
	83,
	101,
	112,
	97,
	114,
	97,
	116,
	111,
	114,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	0,
	77,
	101,
	110,
	117,
	65,
	99,
	116,
	105,
	111,
	110,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	3,
	110,
	97,
	109,
	101,
	0,
	245,
	255,
	255,
	255,
	0,
	1,
	100,
	105,
	115,
	97,
	98,
	108,
	101,
	100,
	0,
	255,
	255,
	255,
	255,
	0,
	2,
	99,
	104,
	101,
	99,
	107,
	101,
	100,
	0,
	255,
	255,
	255,
	255,
	0,
	3,
	77,
	101,
	110,
	117,
	83,
	117,
	98,
	109,
	101,
	110,
	117,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	109,
	101,
	110,
	117,
	0,
	42,
	0,
	0,
	0,
	0,
	1,
	77,
	101,
	110,
	117,
	73,
	116,
	101,
	109,
	86,
	97,
	108,
	117,
	101,
	0,
	3,
	0,
	10,
	0,
	0,
	0,
	3,
	1,
	43,
	0,
	0,
	0,
	2,
	44,
	0,
	0,
	0,
	3,
	45,
	0,
	0,
	0,
	77,
	101,
	110,
	117,
	73,
	116,
	101,
	109,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	118,
	97,
	108,
	117,
	101,
	0,
	46,
	0,
	0,
	0,
	0,
	1,
	75,
	101,
	121,
	98,
	105,
	110,
	100,
	105,
	110,
	103,
	68,
	101,
	102,
	105,
	110,
	105,
	116,
	105,
	111,
	110,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	2,
	107,
	101,
	121,
	115,
	116,
	114,
	111,
	107,
	101,
	115,
	0,
	245,
	255,
	255,
	255,
	0,
	1,
	97,
	99,
	116,
	105,
	111,
	110,
	78,
	97,
	109,
	101,
	0,
	245,
	255,
	255,
	255,
	0,
	2,
	85,
	51,
	50,
	80,
	97,
	105,
	114,
	67,
	111,
	109,
	109,
	97,
	110,
	100,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	2,
	102,
	105,
	114,
	115,
	116,
	0,
	251,
	255,
	255,
	255,
	0,
	1,
	115,
	101,
	99,
	111,
	110,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	2,
	70,
	108,
	111,
	97,
	116,
	67,
	111,
	109,
	109,
	97,
	110,
	100,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	118,
	97,
	108,
	117,
	101,
	0,
	247,
	255,
	255,
	255,
	0,
	1,
	84,
	101,
	120,
	116,
	67,
	111,
	109,
	109,
	97,
	110,
	100,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	118,
	97,
	108,
	117,
	101,
	0,
	245,
	255,
	255,
	255,
	0,
	1,
	83,
	116,
	114,
	105,
	110,
	103,
	80,
	97,
	105,
	114,
	67,
	111,
	109,
	109,
	97,
	110,
	100,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	2,
	112,
	97,
	116,
	104,
	0,
	245,
	255,
	255,
	255,
	0,
	1,
	99,
	111,
	110,
	116,
	101,
	110,
	116,
	0,
	245,
	255,
	255,
	255,
	0,
	2,
	79,
	112,
	101,
	110,
	83,
	117,
	114,
	102,
	97,
	99,
	101,
	67,
	111,
	109,
	109,
	97,
	110,
	100,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	4,
	116,
	105,
	116,
	108,
	101,
	0,
	245,
	255,
	255,
	255,
	0,
	1,
	119,
	105,
	100,
	116,
	104,
	0,
	251,
	255,
	255,
	255,
	0,
	2,
	104,
	101,
	105,
	103,
	104,
	116,
	0,
	251,
	255,
	255,
	255,
	0,
	3,
	111,
	112,
	116,
	105,
	111,
	110,
	115,
	0,
	40,
	0,
	0,
	0,
	0,
	4,
	70,
	105,
	108,
	101,
	68,
	105,
	97,
	108,
	111,
	103,
	79,
	112,
	101,
	110,
	67,
	111,
	109,
	109,
	97,
	110,
	100,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	3,
	116,
	105,
	116,
	108,
	101,
	0,
	245,
	255,
	255,
	255,
	0,
	1,
	100,
	105,
	114,
	101,
	99,
	116,
	111,
	114,
	105,
	101,
	115,
	0,
	255,
	255,
	255,
	255,
	0,
	2,
	109,
	117,
	108,
	116,
	105,
	112,
	108,
	101,
	0,
	255,
	255,
	255,
	255,
	0,
	3,
	78,
	111,
	116,
	105,
	102,
	105,
	99,
	97,
	116,
	105,
	111,
	110,
	67,
	111,
	109,
	109,
	97,
	110,
	100,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	3,
	116,
	105,
	116,
	108,
	101,
	0,
	245,
	255,
	255,
	255,
	0,
	1,
	98,
	111,
	100,
	121,
	0,
	245,
	255,
	255,
	255,
	0,
	2,
	97,
	99,
	116,
	105,
	111,
	110,
	115,
	0,
	242,
	255,
	255,
	255,
	0,
	41,
	0,
	0,
	0,
	0,
	3,
	77,
	101,
	110,
	117,
	115,
	67,
	111,
	109,
	109,
	97,
	110,
	100,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	109,
	101,
	110,
	117,
	115,
	0,
	242,
	255,
	255,
	255,
	0,
	42,
	0,
	0,
	0,
	0,
	1,
	75,
	101,
	121,
	98,
	105,
	110,
	100,
	105,
	110,
	103,
	115,
	67,
	111,
	109,
	109,
	97,
	110,
	100,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	98,
	105,
	110,
	100,
	105,
	110,
	103,
	115,
	0,
	242,
	255,
	255,
	255,
	0,
	48,
	0,
	0,
	0,
	0,
	1,
	67,
	108,
	105,
	112,
	98,
	111,
	97,
	114,
	100,
	73,
	109,
	97,
	103,
	101,
	67,
	111,
	109,
	109,
	97,
	110,
	100,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	2,
	102,
	111,
	114,
	109,
	97,
	116,
	0,
	251,
	255,
	255,
	255,
	0,
	1,
	98,
	121,
	116,
	101,
	115,
	0,
	242,
	255,
	255,
	255,
	0,
	254,
	255,
	255,
	255,
	0,
	2,
	67,
	108,
	111,
	115,
	101,
	82,
	101,
	115,
	111,
	108,
	117,
	116,
	105,
	111,
	110,
	67,
	111,
	109,
	109,
	97,
	110,
	100,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	2,
	114,
	101,
	113,
	117,
	101,
	115,
	116,
	73,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	1,
	97,
	108,
	108,
	111,
	119,
	0,
	255,
	255,
	255,
	255,
	0,
	2,
	73,
	110,
	118,
	111,
	107,
	101,
	78,
	97,
	116,
	105,
	118,
	101,
	67,
	111,
	109,
	109,
	97,
	110,
	100,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	4,
	109,
	111,
	100,
	117,
	108,
	101,
	73,
	100,
	0,
	242,
	255,
	255,
	255,
	0,
	254,
	255,
	255,
	255,
	0,
	1,
	109,
	111,
	100,
	117,
	108,
	101,
	68,
	105,
	103,
	101,
	115,
	116,
	0,
	242,
	255,
	255,
	255,
	0,
	254,
	255,
	255,
	255,
	0,
	2,
	102,
	117,
	110,
	99,
	116,
	105,
	111,
	110,
	73,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	3,
	97,
	114,
	103,
	115,
	0,
	242,
	255,
	255,
	255,
	0,
	254,
	255,
	255,
	255,
	0,
	4,
	67,
	97,
	110,
	99,
	101,
	108,
	78,
	97,
	116,
	105,
	118,
	101,
	67,
	111,
	109,
	109,
	97,
	110,
	100,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	114,
	101,
	113,
	117,
	101,
	115,
	116,
	73,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	1,
	67,
	111,
	110,
	102,
	105,
	103,
	117,
	114,
	101,
	65,
	112,
	112,
	108,
	105,
	99,
	97,
	116,
	105,
	111,
	110,
	67,
	111,
	109,
	109,
	97,
	110,
	100,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	3,
	107,
	101,
	101,
	112,
	65,
	108,
	105,
	118,
	101,
	0,
	255,
	255,
	255,
	255,
	0,
	1,
	113,
	117,
	105,
	116,
	0,
	255,
	255,
	255,
	255,
	0,
	2,
	97,
	99,
	107,
	110,
	111,
	119,
	108,
	101,
	100,
	103,
	101,
	100,
	83,
	101,
	113,
	117,
	101,
	110,
	99,
	101,
	0,
	251,
	255,
	255,
	255,
	0,
	3,
	79,
	112,
	101,
	110,
	80,
	111,
	112,
	117,
	112,
	67,
	111,
	109,
	109,
	97,
	110,
	100,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	5,
	97,
	110,
	99,
	104,
	111,
	114,
	78,
	111,
	100,
	101,
	73,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	1,
	119,
	105,
	100,
	116,
	104,
	0,
	251,
	255,
	255,
	255,
	0,
	2,
	104,
	101,
	105,
	103,
	104,
	116,
	0,
	251,
	255,
	255,
	255,
	0,
	3,
	112,
	108,
	97,
	99,
	101,
	109,
	101,
	110,
	116,
	0,
	251,
	255,
	255,
	255,
	0,
	4,
	103,
	97,
	112,
	0,
	247,
	255,
	255,
	255,
	0,
	5,
	67,
	108,
	111,
	115,
	101,
	80,
	111,
	112,
	117,
	112,
	67,
	111,
	109,
	109,
	97,
	110,
	100,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	114,
	101,
	113,
	117,
	101,
	115,
	116,
	73,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	1,
	67,
	111,
	109,
	109,
	97,
	110,
	100,
	80,
	97,
	121,
	108,
	111,
	97,
	100,
	0,
	3,
	0,
	10,
	0,
	0,
	0,
	16,
	1,
	49,
	0,
	0,
	0,
	2,
	50,
	0,
	0,
	0,
	3,
	51,
	0,
	0,
	0,
	4,
	52,
	0,
	0,
	0,
	5,
	53,
	0,
	0,
	0,
	6,
	54,
	0,
	0,
	0,
	7,
	55,
	0,
	0,
	0,
	8,
	56,
	0,
	0,
	0,
	9,
	57,
	0,
	0,
	0,
	10,
	58,
	0,
	0,
	0,
	11,
	59,
	0,
	0,
	0,
	12,
	60,
	0,
	0,
	0,
	13,
	61,
	0,
	0,
	0,
	14,
	62,
	0,
	0,
	0,
	15,
	63,
	0,
	0,
	0,
	16,
	64,
	0,
	0,
	0,
	78,
	117,
	109,
	98,
	101,
	114,
	86,
	97,
	108,
	117,
	101,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	118,
	97,
	108,
	117,
	101,
	0,
	251,
	255,
	255,
	255,
	0,
	1,
	80,
	97,
	105,
	114,
	86,
	97,
	108,
	117,
	101,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	2,
	119,
	105,
	100,
	116,
	104,
	0,
	247,
	255,
	255,
	255,
	0,
	1,
	104,
	101,
	105,
	103,
	104,
	116,
	0,
	247,
	255,
	255,
	255,
	0,
	2,
	66,
	111,
	111,
	108,
	86,
	97,
	108,
	117,
	101,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	118,
	97,
	108,
	117,
	101,
	0,
	255,
	255,
	255,
	255,
	0,
	1,
	84,
	101,
	120,
	116,
	86,
	97,
	108,
	117,
	101,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	118,
	97,
	108,
	117,
	101,
	0,
	245,
	255,
	255,
	255,
	0,
	1,
	80,
	97,
	116,
	104,
	115,
	86,
	97,
	108,
	117,
	101,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	112,
	97,
	116,
	104,
	115,
	0,
	242,
	255,
	255,
	255,
	0,
	245,
	255,
	255,
	255,
	0,
	1,
	70,
	105,
	108,
	101,
	84,
	101,
	120,
	116,
	86,
	97,
	108,
	117,
	101,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	118,
	97,
	108,
	117,
	101,
	0,
	245,
	255,
	255,
	255,
	0,
	1,
	73,
	109,
	97,
	103,
	101,
	86,
	97,
	108,
	117,
	101,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	2,
	102,
	111,
	114,
	109,
	97,
	116,
	0,
	251,
	255,
	255,
	255,
	0,
	1,
	98,
	121,
	116,
	101,
	115,
	0,
	242,
	255,
	255,
	255,
	0,
	254,
	255,
	255,
	255,
	0,
	2,
	66,
	111,
	117,
	110,
	100,
	115,
	86,
	97,
	108,
	117,
	101,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	4,
	120,
	0,
	247,
	255,
	255,
	255,
	0,
	1,
	121,
	0,
	247,
	255,
	255,
	255,
	0,
	2,
	119,
	105,
	100,
	116,
	104,
	0,
	247,
	255,
	255,
	255,
	0,
	3,
	104,
	101,
	105,
	103,
	104,
	116,
	0,
	247,
	255,
	255,
	255,
	0,
	4,
	87,
	105,
	110,
	100,
	111,
	119,
	83,
	116,
	97,
	116,
	101,
	86,
	97,
	108,
	117,
	101,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	2,
	102,
	117,
	108,
	108,
	115,
	99,
	114,
	101,
	101,
	110,
	0,
	255,
	255,
	255,
	255,
	0,
	1,
	109,
	97,
	120,
	105,
	109,
	105,
	122,
	101,
	100,
	0,
	255,
	255,
	255,
	255,
	0,
	2,
	83,
	99,
	114,
	111,
	108,
	108,
	79,
	102,
	102,
	115,
	101,
	116,
	86,
	97,
	108,
	117,
	101,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	118,
	97,
	108,
	117,
	101,
	0,
	247,
	255,
	255,
	255,
	0,
	1,
	66,
	121,
	116,
	101,
	115,
	86,
	97,
	108,
	117,
	101,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	118,
	97,
	108,
	117,
	101,
	0,
	242,
	255,
	255,
	255,
	0,
	254,
	255,
	255,
	255,
	0,
	1,
	67,
	111,
	109,
	109,
	97,
	110,
	100,
	86,
	97,
	108,
	117,
	101,
	0,
	3,
	0,
	10,
	0,
	0,
	0,
	11,
	1,
	66,
	0,
	0,
	0,
	2,
	67,
	0,
	0,
	0,
	3,
	68,
	0,
	0,
	0,
	4,
	69,
	0,
	0,
	0,
	5,
	70,
	0,
	0,
	0,
	6,
	71,
	0,
	0,
	0,
	7,
	72,
	0,
	0,
	0,
	8,
	73,
	0,
	0,
	0,
	9,
	74,
	0,
	0,
	0,
	10,
	75,
	0,
	0,
	0,
	11,
	76,
	0,
	0,
	0,
	84,
	101,
	120,
	116,
	73,
	110,
	112,
	117,
	116,
	69,
	118,
	101,
	110,
	116,
	68,
	97,
	116,
	97,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	7,
	116,
	101,
	120,
	116,
	0,
	245,
	255,
	255,
	255,
	0,
	1,
	115,
	101,
	108,
	101,
	99,
	116,
	105,
	111,
	110,
	83,
	116,
	97,
	114,
	116,
	0,
	251,
	255,
	255,
	255,
	0,
	2,
	115,
	101,
	108,
	101,
	99,
	116,
	105,
	111,
	110,
	69,
	110,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	3,
	109,
	97,
	114,
	107,
	101,
	100,
	83,
	116,
	97,
	114,
	116,
	0,
	251,
	255,
	255,
	255,
	0,
	4,
	109,
	97,
	114,
	107,
	101,
	100,
	69,
	110,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	5,
	101,
	100,
	105,
	116,
	83,
	101,
	113,
	0,
	251,
	255,
	255,
	255,
	0,
	6,
	114,
	101,
	118,
	101,
	114,
	115,
	101,
	100,
	0,
	255,
	255,
	255,
	255,
	0,
	7,
	67,
	111,
	109,
	109,
	97,
	110,
	100,
	82,
	101,
	115,
	117,
	108,
	116,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	6,
	114,
	101,
	113,
	117,
	101,
	115,
	116,
	73,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	1,
	99,
	111,
	109,
	109,
	97,
	110,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	2,
	110,
	111,
	100,
	101,
	73,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	3,
	115,
	117,
	99,
	99,
	101,
	115,
	115,
	0,
	255,
	255,
	255,
	255,
	0,
	4,
	101,
	114,
	114,
	111,
	114,
	0,
	245,
	255,
	255,
	255,
	0,
	5,
	118,
	97,
	108,
	117,
	101,
	0,
	77,
	0,
	0,
	0,
	0,
	6,
	86,
	105,
	115,
	105,
	98,
	108,
	101,
	82,
	97,
	110,
	103,
	101,
	69,
	118,
	101,
	110,
	116,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	2,
	115,
	116,
	97,
	114,
	116,
	0,
	251,
	255,
	255,
	255,
	0,
	1,
	101,
	110,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	2,
	65,
	110,
	105,
	109,
	97,
	116,
	105,
	111,
	110,
	67,
	111,
	109,
	112,
	108,
	101,
	116,
	101,
	69,
	118,
	101,
	110,
	116,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	103,
	101,
	110,
	101,
	114,
	97,
	116,
	105,
	111,
	110,
	0,
	251,
	255,
	255,
	255,
	0,
	1,
	75,
	101,
	121,
	69,
	118,
	101,
	110,
	116,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	3,
	107,
	101,
	121,
	0,
	245,
	255,
	255,
	255,
	0,
	1,
	109,
	111,
	100,
	105,
	102,
	105,
	101,
	114,
	115,
	0,
	242,
	255,
	255,
	255,
	0,
	245,
	255,
	255,
	255,
	0,
	2,
	97,
	99,
	116,
	105,
	111,
	110,
	0,
	251,
	255,
	255,
	255,
	0,
	3,
	80,
	111,
	105,
	110,
	116,
	101,
	114,
	69,
	118,
	101,
	110,
	116,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	6,
	98,
	117,
	116,
	116,
	111,
	110,
	0,
	251,
	255,
	255,
	255,
	0,
	1,
	109,
	111,
	100,
	105,
	102,
	105,
	101,
	114,
	115,
	0,
	242,
	255,
	255,
	255,
	0,
	245,
	255,
	255,
	255,
	0,
	2,
	97,
	99,
	116,
	105,
	111,
	110,
	0,
	251,
	255,
	255,
	255,
	0,
	3,
	99,
	108,
	105,
	99,
	107,
	67,
	111,
	117,
	110,
	116,
	0,
	251,
	255,
	255,
	255,
	0,
	4,
	120,
	0,
	247,
	255,
	255,
	255,
	0,
	5,
	121,
	0,
	247,
	255,
	255,
	255,
	0,
	6,
	80,
	111,
	105,
	110,
	116,
	101,
	114,
	77,
	111,
	118,
	101,
	69,
	118,
	101,
	110,
	116,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	3,
	109,
	111,
	100,
	105,
	102,
	105,
	101,
	114,
	115,
	0,
	242,
	255,
	255,
	255,
	0,
	245,
	255,
	255,
	255,
	0,
	1,
	120,
	0,
	247,
	255,
	255,
	255,
	0,
	2,
	121,
	0,
	247,
	255,
	255,
	255,
	0,
	3,
	83,
	99,
	114,
	111,
	108,
	108,
	69,
	118,
	101,
	110,
	116,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	6,
	100,
	101,
	108,
	116,
	97,
	75,
	105,
	110,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	1,
	100,
	120,
	0,
	247,
	255,
	255,
	255,
	0,
	2,
	100,
	121,
	0,
	247,
	255,
	255,
	255,
	0,
	3,
	120,
	0,
	247,
	255,
	255,
	255,
	0,
	4,
	121,
	0,
	247,
	255,
	255,
	255,
	0,
	5,
	109,
	111,
	100,
	105,
	102,
	105,
	101,
	114,
	115,
	0,
	242,
	255,
	255,
	255,
	0,
	245,
	255,
	255,
	255,
	0,
	6,
	83,
	117,
	98,
	109,
	105,
	116,
	69,
	118,
	101,
	110,
	116,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	116,
	101,
	120,
	116,
	0,
	245,
	255,
	255,
	255,
	0,
	1,
	87,
	105,
	110,
	100,
	111,
	119,
	82,
	101,
	115,
	105,
	122,
	101,
	69,
	118,
	101,
	110,
	116,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	3,
	119,
	105,
	100,
	116,
	104,
	0,
	247,
	255,
	255,
	255,
	0,
	1,
	104,
	101,
	105,
	103,
	104,
	116,
	0,
	247,
	255,
	255,
	255,
	0,
	2,
	115,
	99,
	97,
	108,
	101,
	70,
	97,
	99,
	116,
	111,
	114,
	0,
	247,
	255,
	255,
	255,
	0,
	3,
	87,
	105,
	110,
	100,
	111,
	119,
	65,
	99,
	116,
	105,
	118,
	97,
	116,
	105,
	111,
	110,
	69,
	118,
	101,
	110,
	116,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	97,
	99,
	116,
	105,
	118,
	101,
	0,
	255,
	255,
	255,
	255,
	0,
	1,
	65,
	99,
	116,
	105,
	111,
	110,
	69,
	118,
	101,
	110,
	116,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	97,
	99,
	116,
	105,
	111,
	110,
	0,
	245,
	255,
	255,
	255,
	0,
	1,
	87,
	105,
	110,
	100,
	111,
	119,
	65,
	112,
	112,
	101,
	97,
	114,
	97,
	110,
	99,
	101,
	69,
	118,
	101,
	110,
	116,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	97,
	112,
	112,
	101,
	97,
	114,
	97,
	110,
	99,
	101,
	0,
	1,
	0,
	0,
	0,
	0,
	1,
	76,
	97,
	121,
	111,
	117,
	116,
	69,
	118,
	101,
	110,
	116,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	4,
	120,
	0,
	247,
	255,
	255,
	255,
	0,
	1,
	121,
	0,
	247,
	255,
	255,
	255,
	0,
	2,
	119,
	105,
	100,
	116,
	104,
	0,
	247,
	255,
	255,
	255,
	0,
	3,
	104,
	101,
	105,
	103,
	104,
	116,
	0,
	247,
	255,
	255,
	255,
	0,
	4,
	68,
	114,
	97,
	103,
	79,
	118,
	101,
	114,
	69,
	118,
	101,
	110,
	116,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	100,
	114,
	97,
	103,
	84,
	121,
	112,
	101,
	0,
	245,
	255,
	255,
	255,
	0,
	1,
	68,
	114,
	97,
	103,
	68,
	114,
	111,
	112,
	69,
	118,
	101,
	110,
	116,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	100,
	114,
	97,
	103,
	84,
	121,
	112,
	101,
	0,
	245,
	255,
	255,
	255,
	0,
	1,
	69,
	120,
	116,
	101,
	114,
	110,
	97,
	108,
	70,
	105,
	108,
	101,
	68,
	114,
	111,
	112,
	69,
	118,
	101,
	110,
	116,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	112,
	97,
	116,
	104,
	115,
	0,
	242,
	255,
	255,
	255,
	0,
	245,
	255,
	255,
	255,
	0,
	1,
	78,
	111,
	116,
	105,
	102,
	105,
	99,
	97,
	116,
	105,
	111,
	110,
	82,
	101,
	115,
	112,
	111,
	110,
	115,
	101,
	69,
	118,
	101,
	110,
	116,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	2,
	116,
	97,
	103,
	0,
	245,
	255,
	255,
	255,
	0,
	1,
	97,
	99,
	116,
	105,
	111,
	110,
	73,
	100,
	0,
	245,
	255,
	255,
	255,
	0,
	2,
	80,
	111,
	105,
	110,
	116,
	101,
	114,
	68,
	111,
	119,
	110,
	79,
	117,
	116,
	115,
	105,
	100,
	101,
	69,
	118,
	101,
	110,
	116,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	2,
	120,
	0,
	247,
	255,
	255,
	255,
	0,
	1,
	121,
	0,
	247,
	255,
	255,
	255,
	0,
	2,
	67,
	108,
	111,
	115,
	101,
	82,
	101,
	113,
	117,
	101,
	115,
	116,
	101,
	100,
	69,
	118,
	101,
	110,
	116,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	1,
	114,
	101,
	113,
	117,
	101,
	115,
	116,
	73,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	1,
	69,
	120,
	116,
	101,
	110,
	115,
	105,
	111,
	110,
	69,
	118,
	101,
	110,
	116,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	2,
	101,
	118,
	101,
	110,
	116,
	73,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	1,
	102,
	105,
	101,
	108,
	100,
	115,
	0,
	242,
	255,
	255,
	255,
	0,
	16,
	0,
	0,
	0,
	0,
	2,
	65,
	112,
	112,
	108,
	105,
	99,
	97,
	116,
	105,
	111,
	110,
	65,
	99,
	116,
	105,
	118,
	97,
	116,
	105,
	111,
	110,
	69,
	118,
	101,
	110,
	116,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	3,
	116,
	97,
	114,
	103,
	101,
	116,
	83,
	117,
	114,
	102,
	97,
	99,
	101,
	73,
	100,
	0,
	251,
	255,
	255,
	255,
	0,
	1,
	114,
	101,
	97,
	115,
	111,
	110,
	0,
	245,
	255,
	255,
	255,
	0,
	2,
	117,
	114,
	108,
	115,
	0,
	242,
	255,
	255,
	255,
	0,
	245,
	255,
	255,
	255,
	0,
	3,
	69,
	118,
	101,
	110,
	116,
	80,
	97,
	121,
	108,
	111,
	97,
	100,
	0,
	3,
	0,
	10,
	0,
	0,
	0,
	22,
	1,
	78,
	0,
	0,
	0,
	2,
	79,
	0,
	0,
	0,
	3,
	80,
	0,
	0,
	0,
	4,
	81,
	0,
	0,
	0,
	5,
	82,
	0,
	0,
	0,
	6,
	83,
	0,
	0,
	0,
	7,
	84,
	0,
	0,
	0,
	8,
	85,
	0,
	0,
	0,
	9,
	86,
	0,
	0,
	0,
	10,
	87,
	0,
	0,
	0,
	11,
	88,
	0,
	0,
	0,
	12,
	89,
	0,
	0,
	0,
	13,
	90,
	0,
	0,
	0,
	14,
	91,
	0,
	0,
	0,
	15,
	92,
	0,
	0,
	0,
	16,
	93,
	0,
	0,
	0,
	17,
	94,
	0,
	0,
	0,
	18,
	95,
	0,
	0,
	0,
	19,
	96,
	0,
	0,
	0,
	20,
	97,
	0,
	0,
	0,
	21,
	98,
	0,
	0,
	0,
	22,
	99,
	0,
	0,
	0,
	69,
	110,
	118,
	101,
	108,
	111,
	112,
	101,
	0,
	2,
	0,
	5,
	0,
	0,
	0,
	2,
	112,
	114,
	111,
	116,
	111,
	99,
	111,
	108,
	86,
	101,
	114,
	115,
	105,
	111,
	110,
	0,
	251,
	255,
	255,
	255,
	0,
	1,
	98,
	111,
	100,
	121,
	0,
	8,
	0,
	0,
	0,
	0,
	2,
	0,
	0,
	0,
	0
]);
var NodeKind = /* @__PURE__ */ function(NodeKind) {
	NodeKind[NodeKind["Unspecified"] = 0] = "Unspecified";
	NodeKind[NodeKind["View"] = 1] = "View";
	NodeKind[NodeKind["Text"] = 2] = "Text";
	NodeKind[NodeKind["Pressable"] = 3] = "Pressable";
	NodeKind[NodeKind["RawText"] = 4] = "RawText";
	NodeKind[NodeKind["TextInput"] = 5] = "TextInput";
	NodeKind[NodeKind["VirtualList"] = 6] = "VirtualList";
	NodeKind[NodeKind["Image"] = 7] = "Image";
	NodeKind[NodeKind["Extension"] = 8] = "Extension";
	NodeKind[NodeKind["Icon"] = 9] = "Icon";
	return NodeKind;
}({});
var WindowAppearance = /* @__PURE__ */ function(WindowAppearance) {
	WindowAppearance[WindowAppearance["Unspecified"] = 0] = "Unspecified";
	WindowAppearance[WindowAppearance["Light"] = 1] = "Light";
	WindowAppearance[WindowAppearance["Dark"] = 2] = "Dark";
	return WindowAppearance;
}({});
var Snapshot = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return Snapshot.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		Snapshot.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.surfaceId !== void 0) {
			view.writeByte(1);
			view.writeUint32(record.surfaceId);
		}
		if (record.epoch !== void 0) {
			view.writeByte(2);
			view.writeUint32(record.epoch);
		}
		if (record.baseRevision !== void 0) {
			view.writeByte(3);
			view.writeUint32(record.baseRevision);
		}
		if (record.revision !== void 0) {
			view.writeByte(4);
			view.writeUint32(record.revision);
		}
		if (record.nodes !== void 0) {
			view.writeByte(5);
			{
				const length0 = record.nodes.length;
				view.writeUint32(length0);
				for (let i0 = 0; i0 < length0; i0++) Node.encodeInto(record.nodes[i0], view);
			}
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return Snapshot(Snapshot.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.surfaceId = view.readUint32();
				break;
			case 2:
				message.epoch = view.readUint32();
				break;
			case 3:
				message.baseRevision = view.readUint32();
				break;
			case 4:
				message.revision = view.readUint32();
				break;
			case 5:
				{
					const length0 = view.readUint32();
					message.nodes = [];
					for (let i0 = 0; i0 < length0; i0++) {
						let x0;
						x0 = Node.readFrom(view);
						message.nodes[i0] = x0;
					}
				}
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var Event = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return Event.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		Event.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.surfaceId !== void 0) {
			view.writeByte(1);
			view.writeUint32(record.surfaceId);
		}
		if (record.epoch !== void 0) {
			view.writeByte(2);
			view.writeUint32(record.epoch);
		}
		if (record.revision !== void 0) {
			view.writeByte(3);
			view.writeUint32(record.revision);
		}
		if (record.sequence !== void 0) {
			view.writeByte(4);
			view.writeUint32(record.sequence);
		}
		if (record.nodeId !== void 0) {
			view.writeByte(5);
			view.writeUint32(record.nodeId);
		}
		if (record.listenerId !== void 0) {
			view.writeByte(6);
			view.writeUint32(record.listenerId);
		}
		if (record.eventType !== void 0) {
			view.writeByte(7);
			view.writeByte(record.eventType);
		}
		if (record.payload !== void 0) {
			view.writeByte(8);
			EventPayload.encodeInto(record.payload, view);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return Event(Event.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.surfaceId = view.readUint32();
				break;
			case 2:
				message.epoch = view.readUint32();
				break;
			case 3:
				message.revision = view.readUint32();
				break;
			case 4:
				message.sequence = view.readUint32();
				break;
			case 5:
				message.nodeId = view.readUint32();
				break;
			case 6:
				message.listenerId = view.readUint32();
				break;
			case 7:
				message.eventType = view.readByte();
				break;
			case 8:
				message.payload = EventPayload.readFrom(view);
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var Patch = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return Patch.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		Patch.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.surfaceId !== void 0) {
			view.writeByte(1);
			view.writeUint32(record.surfaceId);
		}
		if (record.epoch !== void 0) {
			view.writeByte(2);
			view.writeUint32(record.epoch);
		}
		if (record.baseRevision !== void 0) {
			view.writeByte(3);
			view.writeUint32(record.baseRevision);
		}
		if (record.revision !== void 0) {
			view.writeByte(4);
			view.writeUint32(record.revision);
		}
		if (record.operations !== void 0) {
			view.writeByte(5);
			{
				const length0 = record.operations.length;
				view.writeUint32(length0);
				for (let i0 = 0; i0 < length0; i0++) PatchOperation.encodeInto(record.operations[i0], view);
			}
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return Patch(Patch.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.surfaceId = view.readUint32();
				break;
			case 2:
				message.epoch = view.readUint32();
				break;
			case 3:
				message.baseRevision = view.readUint32();
				break;
			case 4:
				message.revision = view.readUint32();
				break;
			case 5:
				{
					const length0 = view.readUint32();
					message.operations = [];
					for (let i0 = 0; i0 < length0; i0++) {
						let x0;
						x0 = PatchOperation.readFrom(view);
						message.operations[i0] = x0;
					}
				}
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var Command = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return Command.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		Command.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.surfaceId !== void 0) {
			view.writeByte(1);
			view.writeUint32(record.surfaceId);
		}
		if (record.epoch !== void 0) {
			view.writeByte(2);
			view.writeUint32(record.epoch);
		}
		if (record.afterRevision !== void 0) {
			view.writeByte(3);
			view.writeUint32(record.afterRevision);
		}
		if (record.requestId !== void 0) {
			view.writeByte(4);
			view.writeUint32(record.requestId);
		}
		if (record.nodeId !== void 0) {
			view.writeByte(5);
			view.writeUint32(record.nodeId);
		}
		if (record.kind !== void 0) {
			view.writeByte(6);
			view.writeByte(record.kind);
		}
		if (record.payload !== void 0) {
			view.writeByte(7);
			CommandPayload.encodeInto(record.payload, view);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return Command(Command.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.surfaceId = view.readUint32();
				break;
			case 2:
				message.epoch = view.readUint32();
				break;
			case 3:
				message.afterRevision = view.readUint32();
				break;
			case 4:
				message.requestId = view.readUint32();
				break;
			case 5:
				message.nodeId = view.readUint32();
				break;
			case 6:
				message.kind = view.readByte();
				break;
			case 7:
				message.payload = CommandPayload.readFrom(view);
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var Body = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return Body.encode(this);
		}
	};
}, {
	fromSnapshot(value) {
		return Body({
			tag: 1,
			value
		});
	},
	fromEvent(value) {
		return Body({
			tag: 2,
			value
		});
	},
	fromPatch(value) {
		return Body({
			tag: 3,
			value
		});
	},
	fromCommand(value) {
		return Body({
			tag: 4,
			value
		});
	},
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		Body.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length + 1;
		view.writeByte(record.tag);
		switch (record.tag) {
			case 1:
				Snapshot.encodeInto(record.value, view);
				break;
			case 2:
				Event.encodeInto(record.value, view);
				break;
			case 3:
				Patch.encodeInto(record.value, view);
				break;
			case 4: Command.encodeInto(record.value, view);
		}
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return Body(Body.readFrom(view));
	},
	readFrom(view) {
		const length = view.readMessageLength();
		const end = view.index + 1 + length;
		const tag = view.readByte();
		switch (tag) {
			case 1: return {
				tag: 1,
				value: Snapshot.readFrom(view)
			};
			case 2: return {
				tag: 2,
				value: Event.readFrom(view)
			};
			case 3: return {
				tag: 3,
				value: Patch.readFrom(view)
			};
			case 4: return {
				tag: 4,
				value: Command.readFrom(view)
			};
			default:
				view.index = end;
				throw new BebopRuntimeError(`Unknown union discriminator: ${tag}`);
		}
	}
}));
var ExtensionBoolValue = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return ExtensionBoolValue.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		ExtensionBoolValue.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.value !== void 0) {
			view.writeByte(1);
			view.writeByte(Number(record.value));
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return ExtensionBoolValue(ExtensionBoolValue.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.value = !!view.readByte();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var ExtensionInt32Value = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return ExtensionInt32Value.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		ExtensionInt32Value.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.value !== void 0) {
			view.writeByte(1);
			view.writeInt32(record.value);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return ExtensionInt32Value(ExtensionInt32Value.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.value = view.readInt32();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var ExtensionU32Value = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return ExtensionU32Value.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		ExtensionU32Value.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.value !== void 0) {
			view.writeByte(1);
			view.writeUint32(record.value);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return ExtensionU32Value(ExtensionU32Value.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.value = view.readUint32();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var ExtensionF32Value = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return ExtensionF32Value.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		ExtensionF32Value.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.value !== void 0) {
			view.writeByte(1);
			view.writeFloat32(record.value);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return ExtensionF32Value(ExtensionF32Value.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.value = view.readFloat32();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var ExtensionTextValue = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return ExtensionTextValue.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		ExtensionTextValue.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.value !== void 0) {
			view.writeByte(1);
			view.writeString(record.value);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return ExtensionTextValue(ExtensionTextValue.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.value = view.readString();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var ExtensionBytesValue = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return ExtensionBytesValue.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		ExtensionBytesValue.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.value !== void 0) {
			view.writeByte(1);
			view.writeBytes(record.value);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return ExtensionBytesValue(ExtensionBytesValue.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.value = view.readBytes();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var ExtensionValue = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return ExtensionValue.encode(this);
		}
	};
}, {
	fromExtensionBoolValue(value) {
		return ExtensionValue({
			tag: 1,
			value
		});
	},
	fromExtensionInt32Value(value) {
		return ExtensionValue({
			tag: 2,
			value
		});
	},
	fromExtensionU32Value(value) {
		return ExtensionValue({
			tag: 3,
			value
		});
	},
	fromExtensionF32Value(value) {
		return ExtensionValue({
			tag: 4,
			value
		});
	},
	fromExtensionTextValue(value) {
		return ExtensionValue({
			tag: 5,
			value
		});
	},
	fromExtensionBytesValue(value) {
		return ExtensionValue({
			tag: 6,
			value
		});
	},
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		ExtensionValue.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length + 1;
		view.writeByte(record.tag);
		switch (record.tag) {
			case 1:
				ExtensionBoolValue.encodeInto(record.value, view);
				break;
			case 2:
				ExtensionInt32Value.encodeInto(record.value, view);
				break;
			case 3:
				ExtensionU32Value.encodeInto(record.value, view);
				break;
			case 4:
				ExtensionF32Value.encodeInto(record.value, view);
				break;
			case 5:
				ExtensionTextValue.encodeInto(record.value, view);
				break;
			case 6: ExtensionBytesValue.encodeInto(record.value, view);
		}
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return ExtensionValue(ExtensionValue.readFrom(view));
	},
	readFrom(view) {
		const length = view.readMessageLength();
		const end = view.index + 1 + length;
		const tag = view.readByte();
		switch (tag) {
			case 1: return {
				tag: 1,
				value: ExtensionBoolValue.readFrom(view)
			};
			case 2: return {
				tag: 2,
				value: ExtensionInt32Value.readFrom(view)
			};
			case 3: return {
				tag: 3,
				value: ExtensionU32Value.readFrom(view)
			};
			case 4: return {
				tag: 4,
				value: ExtensionF32Value.readFrom(view)
			};
			case 5: return {
				tag: 5,
				value: ExtensionTextValue.readFrom(view)
			};
			case 6: return {
				tag: 6,
				value: ExtensionBytesValue.readFrom(view)
			};
			default:
				view.index = end;
				throw new BebopRuntimeError(`Unknown union discriminator: ${tag}`);
		}
	}
}));
var ExtensionField = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return ExtensionField.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		ExtensionField.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.id !== void 0) {
			view.writeByte(1);
			view.writeUint32(record.id);
		}
		if (record.value !== void 0) {
			view.writeByte(2);
			ExtensionValue.encodeInto(record.value, view);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return ExtensionField(ExtensionField.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.id = view.readUint32();
				break;
			case 2:
				message.value = ExtensionValue.readFrom(view);
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var TextInputProperties = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return TextInputProperties.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		TextInputProperties.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.value !== void 0) {
			view.writeByte(1);
			view.writeString(record.value);
		}
		if (record.placeholder !== void 0) {
			view.writeByte(2);
			view.writeString(record.placeholder);
		}
		if (record.multiline !== void 0) {
			view.writeByte(3);
			view.writeByte(Number(record.multiline));
		}
		if (record.disabled !== void 0) {
			view.writeByte(4);
			view.writeByte(Number(record.disabled));
		}
		if (record.controlled !== void 0) {
			view.writeByte(5);
			view.writeByte(Number(record.controlled));
		}
		if (record.ackEditSeq !== void 0) {
			view.writeByte(6);
			view.writeUint32(record.ackEditSeq);
		}
		if (record.selectionStart !== void 0) {
			view.writeByte(7);
			view.writeUint32(record.selectionStart);
		}
		if (record.selectionEnd !== void 0) {
			view.writeByte(8);
			view.writeUint32(record.selectionEnd);
		}
		if (record.markedStart !== void 0) {
			view.writeByte(9);
			view.writeUint32(record.markedStart);
		}
		if (record.markedEnd !== void 0) {
			view.writeByte(10);
			view.writeUint32(record.markedEnd);
		}
		if (record.maxLength !== void 0) {
			view.writeByte(11);
			view.writeUint32(record.maxLength);
		}
		if (record.selectionReversed !== void 0) {
			view.writeByte(12);
			view.writeByte(Number(record.selectionReversed));
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return TextInputProperties(TextInputProperties.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.value = view.readString();
				break;
			case 2:
				message.placeholder = view.readString();
				break;
			case 3:
				message.multiline = !!view.readByte();
				break;
			case 4:
				message.disabled = !!view.readByte();
				break;
			case 5:
				message.controlled = !!view.readByte();
				break;
			case 6:
				message.ackEditSeq = view.readUint32();
				break;
			case 7:
				message.selectionStart = view.readUint32();
				break;
			case 8:
				message.selectionEnd = view.readUint32();
				break;
			case 9:
				message.markedStart = view.readUint32();
				break;
			case 10:
				message.markedEnd = view.readUint32();
				break;
			case 11:
				message.maxLength = view.readUint32();
				break;
			case 12:
				message.selectionReversed = !!view.readByte();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var VirtualListProperties = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return VirtualListProperties.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		VirtualListProperties.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.itemCount !== void 0) {
			view.writeByte(1);
			view.writeUint32(record.itemCount);
		}
		if (record.rangeStart !== void 0) {
			view.writeByte(2);
			view.writeUint32(record.rangeStart);
		}
		if (record.rangeEnd !== void 0) {
			view.writeByte(3);
			view.writeUint32(record.rangeEnd);
		}
		if (record.estimatedItemSize !== void 0) {
			view.writeByte(4);
			view.writeFloat32(record.estimatedItemSize);
		}
		if (record.overscan !== void 0) {
			view.writeByte(5);
			view.writeUint32(record.overscan);
		}
		if (record.dataRevision !== void 0) {
			view.writeByte(6);
			view.writeUint32(record.dataRevision);
		}
		if (record.dataEdit !== void 0) {
			view.writeByte(7);
			VirtualListDataEdit.encodeInto(record.dataEdit, view);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return VirtualListProperties(VirtualListProperties.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.itemCount = view.readUint32();
				break;
			case 2:
				message.rangeStart = view.readUint32();
				break;
			case 3:
				message.rangeEnd = view.readUint32();
				break;
			case 4:
				message.estimatedItemSize = view.readFloat32();
				break;
			case 5:
				message.overscan = view.readUint32();
				break;
			case 6:
				message.dataRevision = view.readUint32();
				break;
			case 7:
				message.dataEdit = VirtualListDataEdit.readFrom(view);
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var ImageProperties = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return ImageProperties.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		ImageProperties.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.source !== void 0) {
			view.writeByte(1);
			view.writeString(record.source);
		}
		if (record.objectFit !== void 0) {
			view.writeByte(2);
			view.writeUint32(record.objectFit);
		}
		if (record.fallbackSource !== void 0) {
			view.writeByte(3);
			view.writeString(record.fallbackSource);
		}
		if (record.sources !== void 0) {
			view.writeByte(4);
			{
				const length0 = record.sources.length;
				view.writeUint32(length0);
				for (let i0 = 0; i0 < length0; i0++) ImageCandidate.encodeInto(record.sources[i0], view);
			}
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return ImageProperties(ImageProperties.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.source = view.readString();
				break;
			case 2:
				message.objectFit = view.readUint32();
				break;
			case 3:
				message.fallbackSource = view.readString();
				break;
			case 4:
				{
					const length0 = view.readUint32();
					message.sources = [];
					for (let i0 = 0; i0 < length0; i0++) {
						let x0;
						x0 = ImageCandidate.readFrom(view);
						message.sources[i0] = x0;
					}
				}
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var DragProperties = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return DragProperties.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		DragProperties.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.dragType !== void 0) {
			view.writeByte(1);
			view.writeString(record.dragType);
		}
		if (record.exportFiles !== void 0) {
			view.writeByte(2);
			{
				const length0 = record.exportFiles.length;
				view.writeUint32(length0);
				for (let i0 = 0; i0 < length0; i0++) view.writeString(record.exportFiles[i0]);
			}
		}
		if (record.acceptsDragOver !== void 0) {
			view.writeByte(3);
			view.writeByte(Number(record.acceptsDragOver));
		}
		if (record.acceptsDrop !== void 0) {
			view.writeByte(4);
			view.writeByte(Number(record.acceptsDrop));
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return DragProperties(DragProperties.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.dragType = view.readString();
				break;
			case 2:
				{
					const length0 = view.readUint32();
					message.exportFiles = [];
					for (let i0 = 0; i0 < length0; i0++) {
						let x0;
						x0 = view.readString();
						message.exportFiles[i0] = x0;
					}
				}
				break;
			case 3:
				message.acceptsDragOver = !!view.readByte();
				break;
			case 4:
				message.acceptsDrop = !!view.readByte();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var ExtensionProperties = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return ExtensionProperties.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		ExtensionProperties.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.providerId !== void 0) {
			view.writeByte(1);
			view.writeBytes(record.providerId);
		}
		if (record.catalogDigest !== void 0) {
			view.writeByte(2);
			view.writeBytes(record.catalogDigest);
		}
		if (record.entryId !== void 0) {
			view.writeByte(3);
			view.writeUint32(record.entryId);
		}
		if (record.entryVersion !== void 0) {
			view.writeByte(4);
			view.writeUint32(record.entryVersion);
		}
		if (record.fields !== void 0) {
			view.writeByte(5);
			{
				const length0 = record.fields.length;
				view.writeUint32(length0);
				for (let i0 = 0; i0 < length0; i0++) ExtensionField.encodeInto(record.fields[i0], view);
			}
		}
		if (record.eventIds !== void 0) {
			view.writeByte(6);
			{
				const length0 = record.eventIds.length;
				view.writeUint32(length0);
				for (let i0 = 0; i0 < length0; i0++) view.writeUint32(record.eventIds[i0]);
			}
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return ExtensionProperties(ExtensionProperties.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.providerId = view.readBytes();
				break;
			case 2:
				message.catalogDigest = view.readBytes();
				break;
			case 3:
				message.entryId = view.readUint32();
				break;
			case 4:
				message.entryVersion = view.readUint32();
				break;
			case 5:
				{
					const length0 = view.readUint32();
					message.fields = [];
					for (let i0 = 0; i0 < length0; i0++) {
						let x0;
						x0 = ExtensionField.readFrom(view);
						message.fields[i0] = x0;
					}
				}
				break;
			case 6:
				{
					const length0 = view.readUint32();
					message.eventIds = [];
					for (let i0 = 0; i0 < length0; i0++) {
						let x0;
						x0 = view.readUint32();
						message.eventIds[i0] = x0;
					}
				}
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var IconProperties = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return IconProperties.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		IconProperties.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.name !== void 0) {
			view.writeByte(1);
			view.writeString(record.name);
		}
		if (record.size !== void 0) {
			view.writeByte(2);
			view.writeFloat32(record.size);
		}
		if (record.color !== void 0) {
			view.writeByte(3);
			view.writeUint32(record.color);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return IconProperties(IconProperties.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.name = view.readString();
				break;
			case 2:
				message.size = view.readFloat32();
				break;
			case 3:
				message.color = view.readUint32();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var HostProperties = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return HostProperties.encode(this);
		}
	};
}, {
	fromTextInputProperties(value) {
		return HostProperties({
			tag: 1,
			value
		});
	},
	fromVirtualListProperties(value) {
		return HostProperties({
			tag: 2,
			value
		});
	},
	fromImageProperties(value) {
		return HostProperties({
			tag: 3,
			value
		});
	},
	fromDragProperties(value) {
		return HostProperties({
			tag: 4,
			value
		});
	},
	fromExtensionProperties(value) {
		return HostProperties({
			tag: 5,
			value
		});
	},
	fromIconProperties(value) {
		return HostProperties({
			tag: 6,
			value
		});
	},
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		HostProperties.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length + 1;
		view.writeByte(record.tag);
		switch (record.tag) {
			case 1:
				TextInputProperties.encodeInto(record.value, view);
				break;
			case 2:
				VirtualListProperties.encodeInto(record.value, view);
				break;
			case 3:
				ImageProperties.encodeInto(record.value, view);
				break;
			case 4:
				DragProperties.encodeInto(record.value, view);
				break;
			case 5:
				ExtensionProperties.encodeInto(record.value, view);
				break;
			case 6: IconProperties.encodeInto(record.value, view);
		}
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return HostProperties(HostProperties.readFrom(view));
	},
	readFrom(view) {
		const length = view.readMessageLength();
		const end = view.index + 1 + length;
		const tag = view.readByte();
		switch (tag) {
			case 1: return {
				tag: 1,
				value: TextInputProperties.readFrom(view)
			};
			case 2: return {
				tag: 2,
				value: VirtualListProperties.readFrom(view)
			};
			case 3: return {
				tag: 3,
				value: ImageProperties.readFrom(view)
			};
			case 4: return {
				tag: 4,
				value: DragProperties.readFrom(view)
			};
			case 5: return {
				tag: 5,
				value: ExtensionProperties.readFrom(view)
			};
			case 6: return {
				tag: 6,
				value: IconProperties.readFrom(view)
			};
			default:
				view.index = end;
				throw new BebopRuntimeError(`Unknown union discriminator: ${tag}`);
		}
	}
}));
var ImageCandidate = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return ImageCandidate.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		ImageCandidate.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.source !== void 0) {
			view.writeByte(1);
			view.writeString(record.source);
		}
		if (record.width !== void 0) {
			view.writeByte(2);
			view.writeUint32(record.width);
		}
		if (record.height !== void 0) {
			view.writeByte(3);
			view.writeUint32(record.height);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return ImageCandidate(ImageCandidate.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.source = view.readString();
				break;
			case 2:
				message.width = view.readUint32();
				break;
			case 3:
				message.height = view.readUint32();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var VirtualListDataEdit = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return VirtualListDataEdit.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		VirtualListDataEdit.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.baseRevision !== void 0) {
			view.writeByte(1);
			view.writeUint32(record.baseRevision);
		}
		if (record.start !== void 0) {
			view.writeByte(2);
			view.writeUint32(record.start);
		}
		if (record.oldCount !== void 0) {
			view.writeByte(3);
			view.writeUint32(record.oldCount);
		}
		if (record.newCount !== void 0) {
			view.writeByte(4);
			view.writeUint32(record.newCount);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return VirtualListDataEdit(VirtualListDataEdit.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.baseRevision = view.readUint32();
				break;
			case 2:
				message.start = view.readUint32();
				break;
			case 3:
				message.oldCount = view.readUint32();
				break;
			case 4:
				message.newCount = view.readUint32();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var AccessibilityProperties = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return AccessibilityProperties.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		AccessibilityProperties.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.role !== void 0) {
			view.writeByte(1);
			view.writeUint32(record.role);
		}
		if (record.label !== void 0) {
			view.writeByte(2);
			view.writeString(record.label);
		}
		if (record.description !== void 0) {
			view.writeByte(3);
			view.writeString(record.description);
		}
		if (record.disabled !== void 0) {
			view.writeByte(4);
			view.writeByte(Number(record.disabled));
		}
		if (record.checked !== void 0) {
			view.writeByte(5);
			view.writeByte(Number(record.checked));
		}
		if (record.selected !== void 0) {
			view.writeByte(6);
			view.writeByte(Number(record.selected));
		}
		if (record.value !== void 0) {
			view.writeByte(7);
			view.writeString(record.value);
		}
		if (record.expanded !== void 0) {
			view.writeByte(8);
			view.writeByte(Number(record.expanded));
		}
		if (record.level !== void 0) {
			view.writeByte(9);
			view.writeUint32(record.level);
		}
		if (record.live !== void 0) {
			view.writeByte(10);
			view.writeUint32(record.live);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return AccessibilityProperties(AccessibilityProperties.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.role = view.readUint32();
				break;
			case 2:
				message.label = view.readString();
				break;
			case 3:
				message.description = view.readString();
				break;
			case 4:
				message.disabled = !!view.readByte();
				break;
			case 5:
				message.checked = !!view.readByte();
				break;
			case 6:
				message.selected = !!view.readByte();
				break;
			case 7:
				message.value = view.readString();
				break;
			case 8:
				message.expanded = !!view.readByte();
				break;
			case 9:
				message.level = view.readUint32();
				break;
			case 10:
				message.live = view.readUint32();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var Transition$1 = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return Transition$1.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		Transition$1.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.durationMs !== void 0) {
			view.writeByte(1);
			view.writeUint32(record.durationMs);
		}
		if (record.delayMs !== void 0) {
			view.writeByte(2);
			view.writeUint32(record.delayMs);
		}
		if (record.easing !== void 0) {
			view.writeByte(3);
			view.writeUint32(record.easing);
		}
		if (record.propertyMask !== void 0) {
			view.writeByte(4);
			view.writeUint32(record.propertyMask);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return Transition$1(Transition$1.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.durationMs = view.readUint32();
				break;
			case 2:
				message.delayMs = view.readUint32();
				break;
			case 3:
				message.easing = view.readUint32();
				break;
			case 4:
				message.propertyMask = view.readUint32();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var BoxShadowValue = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return BoxShadowValue.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		BoxShadowValue.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.offsetX !== void 0) {
			view.writeByte(1);
			view.writeFloat32(record.offsetX);
		}
		if (record.offsetY !== void 0) {
			view.writeByte(2);
			view.writeFloat32(record.offsetY);
		}
		if (record.blurRadius !== void 0) {
			view.writeByte(3);
			view.writeFloat32(record.blurRadius);
		}
		if (record.spreadRadius !== void 0) {
			view.writeByte(4);
			view.writeFloat32(record.spreadRadius);
		}
		if (record.color !== void 0) {
			view.writeByte(5);
			view.writeUint32(record.color);
		}
		if (record.inset !== void 0) {
			view.writeByte(6);
			view.writeByte(Number(record.inset));
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return BoxShadowValue(BoxShadowValue.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.offsetX = view.readFloat32();
				break;
			case 2:
				message.offsetY = view.readFloat32();
				break;
			case 3:
				message.blurRadius = view.readFloat32();
				break;
			case 4:
				message.spreadRadius = view.readFloat32();
				break;
			case 5:
				message.color = view.readUint32();
				break;
			case 6:
				message.inset = !!view.readByte();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var BoxShadowSet = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return BoxShadowSet.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		BoxShadowSet.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.values !== void 0) {
			view.writeByte(1);
			{
				const length0 = record.values.length;
				view.writeUint32(length0);
				for (let i0 = 0; i0 < length0; i0++) BoxShadowValue.encodeInto(record.values[i0], view);
			}
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return BoxShadowSet(BoxShadowSet.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				{
					const length0 = view.readUint32();
					message.values = [];
					for (let i0 = 0; i0 < length0; i0++) {
						let x0;
						x0 = BoxShadowValue.readFrom(view);
						message.values[i0] = x0;
					}
				}
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var LinearGradient = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return LinearGradient.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		LinearGradient.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.angle !== void 0) {
			view.writeByte(1);
			view.writeFloat32(record.angle);
		}
		if (record.startColor !== void 0) {
			view.writeByte(2);
			view.writeUint32(record.startColor);
		}
		if (record.startPosition !== void 0) {
			view.writeByte(3);
			view.writeFloat32(record.startPosition);
		}
		if (record.endColor !== void 0) {
			view.writeByte(4);
			view.writeUint32(record.endColor);
		}
		if (record.endPosition !== void 0) {
			view.writeByte(5);
			view.writeFloat32(record.endPosition);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return LinearGradient(LinearGradient.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.angle = view.readFloat32();
				break;
			case 2:
				message.startColor = view.readUint32();
				break;
			case 3:
				message.startPosition = view.readFloat32();
				break;
			case 4:
				message.endColor = view.readUint32();
				break;
			case 5:
				message.endPosition = view.readFloat32();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var Style = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return Style.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		Style.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.width !== void 0) {
			view.writeByte(1);
			view.writeFloat32(record.width);
		}
		if (record.height !== void 0) {
			view.writeByte(2);
			view.writeFloat32(record.height);
		}
		if (record.flexDirection !== void 0) {
			view.writeByte(3);
			view.writeUint32(record.flexDirection);
		}
		if (record.flexGrow !== void 0) {
			view.writeByte(4);
			view.writeFloat32(record.flexGrow);
		}
		if (record.padding !== void 0) {
			view.writeByte(5);
			view.writeFloat32(record.padding);
		}
		if (record.gap !== void 0) {
			view.writeByte(6);
			view.writeFloat32(record.gap);
		}
		if (record.backgroundColor !== void 0) {
			view.writeByte(7);
			view.writeUint32(record.backgroundColor);
		}
		if (record.color !== void 0) {
			view.writeByte(8);
			view.writeUint32(record.color);
		}
		if (record.opacity !== void 0) {
			view.writeByte(9);
			view.writeFloat32(record.opacity);
		}
		if (record.transition !== void 0) {
			view.writeByte(10);
			Transition$1.encodeInto(record.transition, view);
		}
		if (record.justifyContent !== void 0) {
			view.writeByte(11);
			view.writeUint32(record.justifyContent);
		}
		if (record.alignItems !== void 0) {
			view.writeByte(12);
			view.writeUint32(record.alignItems);
		}
		if (record.borderRadius !== void 0) {
			view.writeByte(13);
			view.writeFloat32(record.borderRadius);
		}
		if (record.borderWidth !== void 0) {
			view.writeByte(14);
			view.writeFloat32(record.borderWidth);
		}
		if (record.borderColor !== void 0) {
			view.writeByte(15);
			view.writeUint32(record.borderColor);
		}
		if (record.fontSize !== void 0) {
			view.writeByte(16);
			view.writeFloat32(record.fontSize);
		}
		if (record.fontWeight !== void 0) {
			view.writeByte(17);
			view.writeUint32(record.fontWeight);
		}
		if (record.overflow !== void 0) {
			view.writeByte(18);
			view.writeUint32(record.overflow);
		}
		if (record.lineClamp !== void 0) {
			view.writeByte(19);
			view.writeUint32(record.lineClamp);
		}
		if (record.textOverflow !== void 0) {
			view.writeByte(20);
			view.writeUint32(record.textOverflow);
		}
		if (record.marginTop !== void 0) {
			view.writeByte(21);
			view.writeFloat32(record.marginTop);
		}
		if (record.marginRight !== void 0) {
			view.writeByte(22);
			view.writeFloat32(record.marginRight);
		}
		if (record.marginBottom !== void 0) {
			view.writeByte(23);
			view.writeFloat32(record.marginBottom);
		}
		if (record.marginLeft !== void 0) {
			view.writeByte(24);
			view.writeFloat32(record.marginLeft);
		}
		if (record.fontStyle !== void 0) {
			view.writeByte(25);
			view.writeUint32(record.fontStyle);
		}
		if (record.textDecoration !== void 0) {
			view.writeByte(26);
			view.writeUint32(record.textDecoration);
		}
		if (record.lineHeight !== void 0) {
			view.writeByte(27);
			view.writeFloat32(record.lineHeight);
		}
		if (record.minWidth !== void 0) {
			view.writeByte(28);
			view.writeFloat32(record.minWidth);
		}
		if (record.maxWidth !== void 0) {
			view.writeByte(29);
			view.writeFloat32(record.maxWidth);
		}
		if (record.minHeight !== void 0) {
			view.writeByte(30);
			view.writeFloat32(record.minHeight);
		}
		if (record.maxHeight !== void 0) {
			view.writeByte(31);
			view.writeFloat32(record.maxHeight);
		}
		if (record.flexShrink !== void 0) {
			view.writeByte(32);
			view.writeFloat32(record.flexShrink);
		}
		if (record.alignSelf !== void 0) {
			view.writeByte(33);
			view.writeUint32(record.alignSelf);
		}
		if (record.position !== void 0) {
			view.writeByte(34);
			view.writeUint32(record.position);
		}
		if (record.left !== void 0) {
			view.writeByte(35);
			view.writeFloat32(record.left);
		}
		if (record.top !== void 0) {
			view.writeByte(36);
			view.writeFloat32(record.top);
		}
		if (record.right !== void 0) {
			view.writeByte(37);
			view.writeFloat32(record.right);
		}
		if (record.bottom !== void 0) {
			view.writeByte(38);
			view.writeFloat32(record.bottom);
		}
		if (record.cursor !== void 0) {
			view.writeByte(39);
			view.writeUint32(record.cursor);
		}
		if (record.textAlign !== void 0) {
			view.writeByte(40);
			view.writeUint32(record.textAlign);
		}
		if (record.boxShadow !== void 0) {
			view.writeByte(41);
			BoxShadowSet.encodeInto(record.boxShadow, view);
		}
		if (record.fontFamily !== void 0) {
			view.writeByte(42);
			view.writeString(record.fontFamily);
		}
		if (record.paddingTop !== void 0) {
			view.writeByte(43);
			view.writeFloat32(record.paddingTop);
		}
		if (record.paddingRight !== void 0) {
			view.writeByte(44);
			view.writeFloat32(record.paddingRight);
		}
		if (record.paddingBottom !== void 0) {
			view.writeByte(45);
			view.writeFloat32(record.paddingBottom);
		}
		if (record.paddingLeft !== void 0) {
			view.writeByte(46);
			view.writeFloat32(record.paddingLeft);
		}
		if (record.borderTopWidth !== void 0) {
			view.writeByte(47);
			view.writeFloat32(record.borderTopWidth);
		}
		if (record.borderRightWidth !== void 0) {
			view.writeByte(48);
			view.writeFloat32(record.borderRightWidth);
		}
		if (record.borderBottomWidth !== void 0) {
			view.writeByte(49);
			view.writeFloat32(record.borderBottomWidth);
		}
		if (record.borderLeftWidth !== void 0) {
			view.writeByte(50);
			view.writeFloat32(record.borderLeftWidth);
		}
		if (record.borderTopLeftRadius !== void 0) {
			view.writeByte(51);
			view.writeFloat32(record.borderTopLeftRadius);
		}
		if (record.borderTopRightRadius !== void 0) {
			view.writeByte(52);
			view.writeFloat32(record.borderTopRightRadius);
		}
		if (record.borderBottomRightRadius !== void 0) {
			view.writeByte(53);
			view.writeFloat32(record.borderBottomRightRadius);
		}
		if (record.borderBottomLeftRadius !== void 0) {
			view.writeByte(54);
			view.writeFloat32(record.borderBottomLeftRadius);
		}
		if (record.widthPercent !== void 0) {
			view.writeByte(55);
			view.writeFloat32(record.widthPercent);
		}
		if (record.heightPercent !== void 0) {
			view.writeByte(56);
			view.writeFloat32(record.heightPercent);
		}
		if (record.flexWrap !== void 0) {
			view.writeByte(57);
			view.writeUint32(record.flexWrap);
		}
		if (record.linearGradient !== void 0) {
			view.writeByte(58);
			LinearGradient.encodeInto(record.linearGradient, view);
		}
		if (record.borderTopColor !== void 0) {
			view.writeByte(59);
			view.writeUint32(record.borderTopColor);
		}
		if (record.borderRightColor !== void 0) {
			view.writeByte(60);
			view.writeUint32(record.borderRightColor);
		}
		if (record.borderBottomColor !== void 0) {
			view.writeByte(61);
			view.writeUint32(record.borderBottomColor);
		}
		if (record.borderLeftColor !== void 0) {
			view.writeByte(62);
			view.writeUint32(record.borderLeftColor);
		}
		if (record.gridColumns !== void 0) {
			view.writeByte(63);
			view.writeUint32(record.gridColumns);
		}
		if (record.gridRows !== void 0) {
			view.writeByte(64);
			view.writeUint32(record.gridRows);
		}
		if (record.gridColumnSpan !== void 0) {
			view.writeByte(65);
			view.writeUint32(record.gridColumnSpan);
		}
		if (record.gridRowSpan !== void 0) {
			view.writeByte(66);
			view.writeUint32(record.gridRowSpan);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return Style(Style.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.width = view.readFloat32();
				break;
			case 2:
				message.height = view.readFloat32();
				break;
			case 3:
				message.flexDirection = view.readUint32();
				break;
			case 4:
				message.flexGrow = view.readFloat32();
				break;
			case 5:
				message.padding = view.readFloat32();
				break;
			case 6:
				message.gap = view.readFloat32();
				break;
			case 7:
				message.backgroundColor = view.readUint32();
				break;
			case 8:
				message.color = view.readUint32();
				break;
			case 9:
				message.opacity = view.readFloat32();
				break;
			case 10:
				message.transition = Transition$1.readFrom(view);
				break;
			case 11:
				message.justifyContent = view.readUint32();
				break;
			case 12:
				message.alignItems = view.readUint32();
				break;
			case 13:
				message.borderRadius = view.readFloat32();
				break;
			case 14:
				message.borderWidth = view.readFloat32();
				break;
			case 15:
				message.borderColor = view.readUint32();
				break;
			case 16:
				message.fontSize = view.readFloat32();
				break;
			case 17:
				message.fontWeight = view.readUint32();
				break;
			case 18:
				message.overflow = view.readUint32();
				break;
			case 19:
				message.lineClamp = view.readUint32();
				break;
			case 20:
				message.textOverflow = view.readUint32();
				break;
			case 21:
				message.marginTop = view.readFloat32();
				break;
			case 22:
				message.marginRight = view.readFloat32();
				break;
			case 23:
				message.marginBottom = view.readFloat32();
				break;
			case 24:
				message.marginLeft = view.readFloat32();
				break;
			case 25:
				message.fontStyle = view.readUint32();
				break;
			case 26:
				message.textDecoration = view.readUint32();
				break;
			case 27:
				message.lineHeight = view.readFloat32();
				break;
			case 28:
				message.minWidth = view.readFloat32();
				break;
			case 29:
				message.maxWidth = view.readFloat32();
				break;
			case 30:
				message.minHeight = view.readFloat32();
				break;
			case 31:
				message.maxHeight = view.readFloat32();
				break;
			case 32:
				message.flexShrink = view.readFloat32();
				break;
			case 33:
				message.alignSelf = view.readUint32();
				break;
			case 34:
				message.position = view.readUint32();
				break;
			case 35:
				message.left = view.readFloat32();
				break;
			case 36:
				message.top = view.readFloat32();
				break;
			case 37:
				message.right = view.readFloat32();
				break;
			case 38:
				message.bottom = view.readFloat32();
				break;
			case 39:
				message.cursor = view.readUint32();
				break;
			case 40:
				message.textAlign = view.readUint32();
				break;
			case 41:
				message.boxShadow = BoxShadowSet.readFrom(view);
				break;
			case 42:
				message.fontFamily = view.readString();
				break;
			case 43:
				message.paddingTop = view.readFloat32();
				break;
			case 44:
				message.paddingRight = view.readFloat32();
				break;
			case 45:
				message.paddingBottom = view.readFloat32();
				break;
			case 46:
				message.paddingLeft = view.readFloat32();
				break;
			case 47:
				message.borderTopWidth = view.readFloat32();
				break;
			case 48:
				message.borderRightWidth = view.readFloat32();
				break;
			case 49:
				message.borderBottomWidth = view.readFloat32();
				break;
			case 50:
				message.borderLeftWidth = view.readFloat32();
				break;
			case 51:
				message.borderTopLeftRadius = view.readFloat32();
				break;
			case 52:
				message.borderTopRightRadius = view.readFloat32();
				break;
			case 53:
				message.borderBottomRightRadius = view.readFloat32();
				break;
			case 54:
				message.borderBottomLeftRadius = view.readFloat32();
				break;
			case 55:
				message.widthPercent = view.readFloat32();
				break;
			case 56:
				message.heightPercent = view.readFloat32();
				break;
			case 57:
				message.flexWrap = view.readUint32();
				break;
			case 58:
				message.linearGradient = LinearGradient.readFrom(view);
				break;
			case 59:
				message.borderTopColor = view.readUint32();
				break;
			case 60:
				message.borderRightColor = view.readUint32();
				break;
			case 61:
				message.borderBottomColor = view.readUint32();
				break;
			case 62:
				message.borderLeftColor = view.readUint32();
				break;
			case 63:
				message.gridColumns = view.readUint32();
				break;
			case 64:
				message.gridRows = view.readUint32();
				break;
			case 65:
				message.gridColumnSpan = view.readUint32();
				break;
			case 66:
				message.gridRowSpan = view.readUint32();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var Node = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return Node.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		Node.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.id !== void 0) {
			view.writeByte(1);
			view.writeUint32(record.id);
		}
		if (record.parentId !== void 0) {
			view.writeByte(2);
			view.writeUint32(record.parentId);
		}
		if (record.index !== void 0) {
			view.writeByte(3);
			view.writeUint32(record.index);
		}
		if (record.kind !== void 0) {
			view.writeByte(4);
			view.writeByte(record.kind);
		}
		if (record.style !== void 0) {
			view.writeByte(5);
			Style.encodeInto(record.style, view);
		}
		if (record.text !== void 0) {
			view.writeByte(6);
			view.writeString(record.text);
		}
		if (record.listenerId !== void 0) {
			view.writeByte(7);
			view.writeUint32(record.listenerId);
		}
		if (record.hostProperties !== void 0) {
			view.writeByte(8);
			HostProperties.encodeInto(record.hostProperties, view);
		}
		if (record.accessibility !== void 0) {
			view.writeByte(9);
			AccessibilityProperties.encodeInto(record.accessibility, view);
		}
		if (record.focusable !== void 0) {
			view.writeByte(10);
			view.writeByte(Number(record.focusable));
		}
		if (record.selectable !== void 0) {
			view.writeByte(11);
			view.writeByte(Number(record.selectable));
		}
		if (record.tooltip !== void 0) {
			view.writeByte(12);
			view.writeString(record.tooltip);
		}
		if (record.acceptsPointerMove !== void 0) {
			view.writeByte(13);
			view.writeByte(Number(record.acceptsPointerMove));
		}
		if (record.observesLayout !== void 0) {
			view.writeByte(14);
			view.writeByte(Number(record.observesLayout));
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return Node(Node.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.id = view.readUint32();
				break;
			case 2:
				message.parentId = view.readUint32();
				break;
			case 3:
				message.index = view.readUint32();
				break;
			case 4:
				message.kind = view.readByte();
				break;
			case 5:
				message.style = Style.readFrom(view);
				break;
			case 6:
				message.text = view.readString();
				break;
			case 7:
				message.listenerId = view.readUint32();
				break;
			case 8:
				message.hostProperties = HostProperties.readFrom(view);
				break;
			case 9:
				message.accessibility = AccessibilityProperties.readFrom(view);
				break;
			case 10:
				message.focusable = !!view.readByte();
				break;
			case 11:
				message.selectable = !!view.readByte();
				break;
			case 12:
				message.tooltip = view.readString();
				break;
			case 13:
				message.acceptsPointerMove = !!view.readByte();
				break;
			case 14:
				message.observesLayout = !!view.readByte();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var ClearStyle = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return ClearStyle.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		ClearStyle.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return ClearStyle(ClearStyle.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			default:
				view.index = end;
				return message;
		}
	}
}));
var PatchCreate = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return PatchCreate.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		PatchCreate.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.node !== void 0) {
			view.writeByte(1);
			Node.encodeInto(record.node, view);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return PatchCreate(PatchCreate.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.node = Node.readFrom(view);
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var PatchUpdate = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return PatchUpdate.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		PatchUpdate.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.id !== void 0) {
			view.writeByte(1);
			view.writeUint32(record.id);
		}
		if (record.mask !== void 0) {
			view.writeByte(2);
			view.writeUint32(record.mask);
		}
		if (record.style !== void 0) {
			view.writeByte(3);
			Style.encodeInto(record.style, view);
		}
		if (record.clearStyle !== void 0) {
			view.writeByte(4);
			ClearStyle.encodeInto(record.clearStyle, view);
		}
		if (record.text !== void 0) {
			view.writeByte(5);
			view.writeString(record.text);
		}
		if (record.listenerId !== void 0) {
			view.writeByte(6);
			view.writeUint32(record.listenerId);
		}
		if (record.hostProperties !== void 0) {
			view.writeByte(7);
			HostProperties.encodeInto(record.hostProperties, view);
		}
		if (record.accessibility !== void 0) {
			view.writeByte(8);
			AccessibilityProperties.encodeInto(record.accessibility, view);
		}
		if (record.focusable !== void 0) {
			view.writeByte(9);
			view.writeByte(Number(record.focusable));
		}
		if (record.selectable !== void 0) {
			view.writeByte(10);
			view.writeByte(Number(record.selectable));
		}
		if (record.tooltip !== void 0) {
			view.writeByte(11);
			view.writeString(record.tooltip);
		}
		if (record.acceptsPointerMove !== void 0) {
			view.writeByte(12);
			view.writeByte(Number(record.acceptsPointerMove));
		}
		if (record.observesLayout !== void 0) {
			view.writeByte(13);
			view.writeByte(Number(record.observesLayout));
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return PatchUpdate(PatchUpdate.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.id = view.readUint32();
				break;
			case 2:
				message.mask = view.readUint32();
				break;
			case 3:
				message.style = Style.readFrom(view);
				break;
			case 4:
				message.clearStyle = ClearStyle.readFrom(view);
				break;
			case 5:
				message.text = view.readString();
				break;
			case 6:
				message.listenerId = view.readUint32();
				break;
			case 7:
				message.hostProperties = HostProperties.readFrom(view);
				break;
			case 8:
				message.accessibility = AccessibilityProperties.readFrom(view);
				break;
			case 9:
				message.focusable = !!view.readByte();
				break;
			case 10:
				message.selectable = !!view.readByte();
				break;
			case 11:
				message.tooltip = view.readString();
				break;
			case 12:
				message.acceptsPointerMove = !!view.readByte();
				break;
			case 13:
				message.observesLayout = !!view.readByte();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var PatchMove = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return PatchMove.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		PatchMove.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.id !== void 0) {
			view.writeByte(1);
			view.writeUint32(record.id);
		}
		if (record.parentId !== void 0) {
			view.writeByte(2);
			view.writeUint32(record.parentId);
		}
		if (record.index !== void 0) {
			view.writeByte(3);
			view.writeUint32(record.index);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return PatchMove(PatchMove.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.id = view.readUint32();
				break;
			case 2:
				message.parentId = view.readUint32();
				break;
			case 3:
				message.index = view.readUint32();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var PatchDelete = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return PatchDelete.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		PatchDelete.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.id !== void 0) {
			view.writeByte(1);
			view.writeUint32(record.id);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return PatchDelete(PatchDelete.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.id = view.readUint32();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var PatchOperationValue = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return PatchOperationValue.encode(this);
		}
	};
}, {
	fromPatchCreate(value) {
		return PatchOperationValue({
			tag: 1,
			value
		});
	},
	fromPatchUpdate(value) {
		return PatchOperationValue({
			tag: 2,
			value
		});
	},
	fromPatchMove(value) {
		return PatchOperationValue({
			tag: 3,
			value
		});
	},
	fromPatchDelete(value) {
		return PatchOperationValue({
			tag: 4,
			value
		});
	},
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		PatchOperationValue.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length + 1;
		view.writeByte(record.tag);
		switch (record.tag) {
			case 1:
				PatchCreate.encodeInto(record.value, view);
				break;
			case 2:
				PatchUpdate.encodeInto(record.value, view);
				break;
			case 3:
				PatchMove.encodeInto(record.value, view);
				break;
			case 4: PatchDelete.encodeInto(record.value, view);
		}
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return PatchOperationValue(PatchOperationValue.readFrom(view));
	},
	readFrom(view) {
		const length = view.readMessageLength();
		const end = view.index + 1 + length;
		const tag = view.readByte();
		switch (tag) {
			case 1: return {
				tag: 1,
				value: PatchCreate.readFrom(view)
			};
			case 2: return {
				tag: 2,
				value: PatchUpdate.readFrom(view)
			};
			case 3: return {
				tag: 3,
				value: PatchMove.readFrom(view)
			};
			case 4: return {
				tag: 4,
				value: PatchDelete.readFrom(view)
			};
			default:
				view.index = end;
				throw new BebopRuntimeError(`Unknown union discriminator: ${tag}`);
		}
	}
}));
var PatchOperation = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return PatchOperation.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		PatchOperation.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.operation !== void 0) {
			view.writeByte(1);
			PatchOperationValue.encodeInto(record.operation, view);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return PatchOperation(PatchOperation.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.operation = PatchOperationValue.readFrom(view);
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var WindowOpenOptions = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return WindowOpenOptions.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		WindowOpenOptions.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.kind !== void 0) {
			view.writeByte(1);
			view.writeUint32(record.kind);
		}
		if (record.resizable !== void 0) {
			view.writeByte(2);
			view.writeByte(Number(record.resizable));
		}
		if (record.minWidth !== void 0) {
			view.writeByte(3);
			view.writeUint32(record.minWidth);
		}
		if (record.minHeight !== void 0) {
			view.writeByte(4);
			view.writeUint32(record.minHeight);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return WindowOpenOptions(WindowOpenOptions.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.kind = view.readUint32();
				break;
			case 2:
				message.resizable = !!view.readByte();
				break;
			case 3:
				message.minWidth = view.readUint32();
				break;
			case 4:
				message.minHeight = view.readUint32();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var NotificationActionDefinition = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return NotificationActionDefinition.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		NotificationActionDefinition.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.id !== void 0) {
			view.writeByte(1);
			view.writeString(record.id);
		}
		if (record.label !== void 0) {
			view.writeByte(2);
			view.writeString(record.label);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return NotificationActionDefinition(NotificationActionDefinition.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.id = view.readString();
				break;
			case 2:
				message.label = view.readString();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var MenuDefinition = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return MenuDefinition.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		MenuDefinition.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.title !== void 0) {
			view.writeByte(1);
			view.writeString(record.title);
		}
		if (record.items !== void 0) {
			view.writeByte(2);
			{
				const length0 = record.items.length;
				view.writeUint32(length0);
				for (let i0 = 0; i0 < length0; i0++) MenuItem.encodeInto(record.items[i0], view);
			}
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return MenuDefinition(MenuDefinition.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.title = view.readString();
				break;
			case 2:
				{
					const length0 = view.readUint32();
					message.items = [];
					for (let i0 = 0; i0 < length0; i0++) {
						let x0;
						x0 = MenuItem.readFrom(view);
						message.items[i0] = x0;
					}
				}
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var MenuSeparator = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return MenuSeparator.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		MenuSeparator.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return MenuSeparator(MenuSeparator.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			default:
				view.index = end;
				return message;
		}
	}
}));
var MenuAction = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return MenuAction.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		MenuAction.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.name !== void 0) {
			view.writeByte(1);
			view.writeString(record.name);
		}
		if (record.disabled !== void 0) {
			view.writeByte(2);
			view.writeByte(Number(record.disabled));
		}
		if (record.checked !== void 0) {
			view.writeByte(3);
			view.writeByte(Number(record.checked));
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return MenuAction(MenuAction.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.name = view.readString();
				break;
			case 2:
				message.disabled = !!view.readByte();
				break;
			case 3:
				message.checked = !!view.readByte();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var MenuSubmenu = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return MenuSubmenu.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		MenuSubmenu.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.menu !== void 0) {
			view.writeByte(1);
			MenuDefinition.encodeInto(record.menu, view);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return MenuSubmenu(MenuSubmenu.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.menu = MenuDefinition.readFrom(view);
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var MenuItemValue = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return MenuItemValue.encode(this);
		}
	};
}, {
	fromMenuSeparator(value) {
		return MenuItemValue({
			tag: 1,
			value
		});
	},
	fromMenuAction(value) {
		return MenuItemValue({
			tag: 2,
			value
		});
	},
	fromMenuSubmenu(value) {
		return MenuItemValue({
			tag: 3,
			value
		});
	},
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		MenuItemValue.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length + 1;
		view.writeByte(record.tag);
		switch (record.tag) {
			case 1:
				MenuSeparator.encodeInto(record.value, view);
				break;
			case 2:
				MenuAction.encodeInto(record.value, view);
				break;
			case 3: MenuSubmenu.encodeInto(record.value, view);
		}
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return MenuItemValue(MenuItemValue.readFrom(view));
	},
	readFrom(view) {
		const length = view.readMessageLength();
		const end = view.index + 1 + length;
		const tag = view.readByte();
		switch (tag) {
			case 1: return {
				tag: 1,
				value: MenuSeparator.readFrom(view)
			};
			case 2: return {
				tag: 2,
				value: MenuAction.readFrom(view)
			};
			case 3: return {
				tag: 3,
				value: MenuSubmenu.readFrom(view)
			};
			default:
				view.index = end;
				throw new BebopRuntimeError(`Unknown union discriminator: ${tag}`);
		}
	}
}));
var MenuItem = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return MenuItem.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		MenuItem.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.value !== void 0) {
			view.writeByte(1);
			MenuItemValue.encodeInto(record.value, view);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return MenuItem(MenuItem.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.value = MenuItemValue.readFrom(view);
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var KeybindingDefinition = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return KeybindingDefinition.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		KeybindingDefinition.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.keystrokes !== void 0) {
			view.writeByte(1);
			view.writeString(record.keystrokes);
		}
		if (record.actionName !== void 0) {
			view.writeByte(2);
			view.writeString(record.actionName);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return KeybindingDefinition(KeybindingDefinition.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.keystrokes = view.readString();
				break;
			case 2:
				message.actionName = view.readString();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var U32PairCommand = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return U32PairCommand.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		U32PairCommand.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.first !== void 0) {
			view.writeByte(1);
			view.writeUint32(record.first);
		}
		if (record.second !== void 0) {
			view.writeByte(2);
			view.writeUint32(record.second);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return U32PairCommand(U32PairCommand.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.first = view.readUint32();
				break;
			case 2:
				message.second = view.readUint32();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var FloatCommand = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return FloatCommand.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		FloatCommand.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.value !== void 0) {
			view.writeByte(1);
			view.writeFloat32(record.value);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return FloatCommand(FloatCommand.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.value = view.readFloat32();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var TextCommand = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return TextCommand.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		TextCommand.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.value !== void 0) {
			view.writeByte(1);
			view.writeString(record.value);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return TextCommand(TextCommand.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.value = view.readString();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var StringPairCommand = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return StringPairCommand.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		StringPairCommand.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.path !== void 0) {
			view.writeByte(1);
			view.writeString(record.path);
		}
		if (record.content !== void 0) {
			view.writeByte(2);
			view.writeString(record.content);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return StringPairCommand(StringPairCommand.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.path = view.readString();
				break;
			case 2:
				message.content = view.readString();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var OpenSurfaceCommand = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return OpenSurfaceCommand.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		OpenSurfaceCommand.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.title !== void 0) {
			view.writeByte(1);
			view.writeString(record.title);
		}
		if (record.width !== void 0) {
			view.writeByte(2);
			view.writeUint32(record.width);
		}
		if (record.height !== void 0) {
			view.writeByte(3);
			view.writeUint32(record.height);
		}
		if (record.options !== void 0) {
			view.writeByte(4);
			WindowOpenOptions.encodeInto(record.options, view);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return OpenSurfaceCommand(OpenSurfaceCommand.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.title = view.readString();
				break;
			case 2:
				message.width = view.readUint32();
				break;
			case 3:
				message.height = view.readUint32();
				break;
			case 4:
				message.options = WindowOpenOptions.readFrom(view);
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var FileDialogOpenCommand = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return FileDialogOpenCommand.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		FileDialogOpenCommand.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.title !== void 0) {
			view.writeByte(1);
			view.writeString(record.title);
		}
		if (record.directories !== void 0) {
			view.writeByte(2);
			view.writeByte(Number(record.directories));
		}
		if (record.multiple !== void 0) {
			view.writeByte(3);
			view.writeByte(Number(record.multiple));
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return FileDialogOpenCommand(FileDialogOpenCommand.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.title = view.readString();
				break;
			case 2:
				message.directories = !!view.readByte();
				break;
			case 3:
				message.multiple = !!view.readByte();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var NotificationCommand = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return NotificationCommand.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		NotificationCommand.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.title !== void 0) {
			view.writeByte(1);
			view.writeString(record.title);
		}
		if (record.body !== void 0) {
			view.writeByte(2);
			view.writeString(record.body);
		}
		if (record.actions !== void 0) {
			view.writeByte(3);
			{
				const length0 = record.actions.length;
				view.writeUint32(length0);
				for (let i0 = 0; i0 < length0; i0++) NotificationActionDefinition.encodeInto(record.actions[i0], view);
			}
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return NotificationCommand(NotificationCommand.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.title = view.readString();
				break;
			case 2:
				message.body = view.readString();
				break;
			case 3:
				{
					const length0 = view.readUint32();
					message.actions = [];
					for (let i0 = 0; i0 < length0; i0++) {
						let x0;
						x0 = NotificationActionDefinition.readFrom(view);
						message.actions[i0] = x0;
					}
				}
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var MenusCommand = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return MenusCommand.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		MenusCommand.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.menus !== void 0) {
			view.writeByte(1);
			{
				const length0 = record.menus.length;
				view.writeUint32(length0);
				for (let i0 = 0; i0 < length0; i0++) MenuDefinition.encodeInto(record.menus[i0], view);
			}
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return MenusCommand(MenusCommand.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				{
					const length0 = view.readUint32();
					message.menus = [];
					for (let i0 = 0; i0 < length0; i0++) {
						let x0;
						x0 = MenuDefinition.readFrom(view);
						message.menus[i0] = x0;
					}
				}
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var KeybindingsCommand = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return KeybindingsCommand.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		KeybindingsCommand.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.bindings !== void 0) {
			view.writeByte(1);
			{
				const length0 = record.bindings.length;
				view.writeUint32(length0);
				for (let i0 = 0; i0 < length0; i0++) KeybindingDefinition.encodeInto(record.bindings[i0], view);
			}
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return KeybindingsCommand(KeybindingsCommand.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				{
					const length0 = view.readUint32();
					message.bindings = [];
					for (let i0 = 0; i0 < length0; i0++) {
						let x0;
						x0 = KeybindingDefinition.readFrom(view);
						message.bindings[i0] = x0;
					}
				}
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var ClipboardImageCommand = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return ClipboardImageCommand.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		ClipboardImageCommand.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.format !== void 0) {
			view.writeByte(1);
			view.writeUint32(record.format);
		}
		if (record.bytes !== void 0) {
			view.writeByte(2);
			view.writeBytes(record.bytes);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return ClipboardImageCommand(ClipboardImageCommand.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.format = view.readUint32();
				break;
			case 2:
				message.bytes = view.readBytes();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var CloseResolutionCommand = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return CloseResolutionCommand.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		CloseResolutionCommand.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.requestId !== void 0) {
			view.writeByte(1);
			view.writeUint32(record.requestId);
		}
		if (record.allow !== void 0) {
			view.writeByte(2);
			view.writeByte(Number(record.allow));
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return CloseResolutionCommand(CloseResolutionCommand.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.requestId = view.readUint32();
				break;
			case 2:
				message.allow = !!view.readByte();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var InvokeNativeCommand = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return InvokeNativeCommand.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		InvokeNativeCommand.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.moduleId !== void 0) {
			view.writeByte(1);
			view.writeBytes(record.moduleId);
		}
		if (record.moduleDigest !== void 0) {
			view.writeByte(2);
			view.writeBytes(record.moduleDigest);
		}
		if (record.functionId !== void 0) {
			view.writeByte(3);
			view.writeUint32(record.functionId);
		}
		if (record.args !== void 0) {
			view.writeByte(4);
			view.writeBytes(record.args);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return InvokeNativeCommand(InvokeNativeCommand.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.moduleId = view.readBytes();
				break;
			case 2:
				message.moduleDigest = view.readBytes();
				break;
			case 3:
				message.functionId = view.readUint32();
				break;
			case 4:
				message.args = view.readBytes();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var CancelNativeCommand = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return CancelNativeCommand.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		CancelNativeCommand.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.requestId !== void 0) {
			view.writeByte(1);
			view.writeUint32(record.requestId);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return CancelNativeCommand(CancelNativeCommand.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.requestId = view.readUint32();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var ConfigureApplicationCommand = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return ConfigureApplicationCommand.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		ConfigureApplicationCommand.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.keepAlive !== void 0) {
			view.writeByte(1);
			view.writeByte(Number(record.keepAlive));
		}
		if (record.quit !== void 0) {
			view.writeByte(2);
			view.writeByte(Number(record.quit));
		}
		if (record.acknowledgedSequence !== void 0) {
			view.writeByte(3);
			view.writeUint32(record.acknowledgedSequence);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return ConfigureApplicationCommand(ConfigureApplicationCommand.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.keepAlive = !!view.readByte();
				break;
			case 2:
				message.quit = !!view.readByte();
				break;
			case 3:
				message.acknowledgedSequence = view.readUint32();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var OpenPopupCommand = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return OpenPopupCommand.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		OpenPopupCommand.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.anchorNodeId !== void 0) {
			view.writeByte(1);
			view.writeUint32(record.anchorNodeId);
		}
		if (record.width !== void 0) {
			view.writeByte(2);
			view.writeUint32(record.width);
		}
		if (record.height !== void 0) {
			view.writeByte(3);
			view.writeUint32(record.height);
		}
		if (record.placement !== void 0) {
			view.writeByte(4);
			view.writeUint32(record.placement);
		}
		if (record.gap !== void 0) {
			view.writeByte(5);
			view.writeFloat32(record.gap);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return OpenPopupCommand(OpenPopupCommand.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.anchorNodeId = view.readUint32();
				break;
			case 2:
				message.width = view.readUint32();
				break;
			case 3:
				message.height = view.readUint32();
				break;
			case 4:
				message.placement = view.readUint32();
				break;
			case 5:
				message.gap = view.readFloat32();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var ClosePopupCommand = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return ClosePopupCommand.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		ClosePopupCommand.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.requestId !== void 0) {
			view.writeByte(1);
			view.writeUint32(record.requestId);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return ClosePopupCommand(ClosePopupCommand.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.requestId = view.readUint32();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var CommandPayload = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return CommandPayload.encode(this);
		}
	};
}, {
	fromU32PairCommand(value) {
		return CommandPayload({
			tag: 1,
			value
		});
	},
	fromFloatCommand(value) {
		return CommandPayload({
			tag: 2,
			value
		});
	},
	fromTextCommand(value) {
		return CommandPayload({
			tag: 3,
			value
		});
	},
	fromStringPairCommand(value) {
		return CommandPayload({
			tag: 4,
			value
		});
	},
	fromOpenSurfaceCommand(value) {
		return CommandPayload({
			tag: 5,
			value
		});
	},
	fromFileDialogOpenCommand(value) {
		return CommandPayload({
			tag: 6,
			value
		});
	},
	fromNotificationCommand(value) {
		return CommandPayload({
			tag: 7,
			value
		});
	},
	fromMenusCommand(value) {
		return CommandPayload({
			tag: 8,
			value
		});
	},
	fromKeybindingsCommand(value) {
		return CommandPayload({
			tag: 9,
			value
		});
	},
	fromClipboardImageCommand(value) {
		return CommandPayload({
			tag: 10,
			value
		});
	},
	fromCloseResolutionCommand(value) {
		return CommandPayload({
			tag: 11,
			value
		});
	},
	fromInvokeNativeCommand(value) {
		return CommandPayload({
			tag: 12,
			value
		});
	},
	fromCancelNativeCommand(value) {
		return CommandPayload({
			tag: 13,
			value
		});
	},
	fromConfigureApplicationCommand(value) {
		return CommandPayload({
			tag: 14,
			value
		});
	},
	fromOpenPopupCommand(value) {
		return CommandPayload({
			tag: 15,
			value
		});
	},
	fromClosePopupCommand(value) {
		return CommandPayload({
			tag: 16,
			value
		});
	},
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		CommandPayload.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length + 1;
		view.writeByte(record.tag);
		switch (record.tag) {
			case 1:
				U32PairCommand.encodeInto(record.value, view);
				break;
			case 2:
				FloatCommand.encodeInto(record.value, view);
				break;
			case 3:
				TextCommand.encodeInto(record.value, view);
				break;
			case 4:
				StringPairCommand.encodeInto(record.value, view);
				break;
			case 5:
				OpenSurfaceCommand.encodeInto(record.value, view);
				break;
			case 6:
				FileDialogOpenCommand.encodeInto(record.value, view);
				break;
			case 7:
				NotificationCommand.encodeInto(record.value, view);
				break;
			case 8:
				MenusCommand.encodeInto(record.value, view);
				break;
			case 9:
				KeybindingsCommand.encodeInto(record.value, view);
				break;
			case 10:
				ClipboardImageCommand.encodeInto(record.value, view);
				break;
			case 11:
				CloseResolutionCommand.encodeInto(record.value, view);
				break;
			case 12:
				InvokeNativeCommand.encodeInto(record.value, view);
				break;
			case 13:
				CancelNativeCommand.encodeInto(record.value, view);
				break;
			case 14:
				ConfigureApplicationCommand.encodeInto(record.value, view);
				break;
			case 15:
				OpenPopupCommand.encodeInto(record.value, view);
				break;
			case 16: ClosePopupCommand.encodeInto(record.value, view);
		}
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return CommandPayload(CommandPayload.readFrom(view));
	},
	readFrom(view) {
		const length = view.readMessageLength();
		const end = view.index + 1 + length;
		const tag = view.readByte();
		switch (tag) {
			case 1: return {
				tag: 1,
				value: U32PairCommand.readFrom(view)
			};
			case 2: return {
				tag: 2,
				value: FloatCommand.readFrom(view)
			};
			case 3: return {
				tag: 3,
				value: TextCommand.readFrom(view)
			};
			case 4: return {
				tag: 4,
				value: StringPairCommand.readFrom(view)
			};
			case 5: return {
				tag: 5,
				value: OpenSurfaceCommand.readFrom(view)
			};
			case 6: return {
				tag: 6,
				value: FileDialogOpenCommand.readFrom(view)
			};
			case 7: return {
				tag: 7,
				value: NotificationCommand.readFrom(view)
			};
			case 8: return {
				tag: 8,
				value: MenusCommand.readFrom(view)
			};
			case 9: return {
				tag: 9,
				value: KeybindingsCommand.readFrom(view)
			};
			case 10: return {
				tag: 10,
				value: ClipboardImageCommand.readFrom(view)
			};
			case 11: return {
				tag: 11,
				value: CloseResolutionCommand.readFrom(view)
			};
			case 12: return {
				tag: 12,
				value: InvokeNativeCommand.readFrom(view)
			};
			case 13: return {
				tag: 13,
				value: CancelNativeCommand.readFrom(view)
			};
			case 14: return {
				tag: 14,
				value: ConfigureApplicationCommand.readFrom(view)
			};
			case 15: return {
				tag: 15,
				value: OpenPopupCommand.readFrom(view)
			};
			case 16: return {
				tag: 16,
				value: ClosePopupCommand.readFrom(view)
			};
			default:
				view.index = end;
				throw new BebopRuntimeError(`Unknown union discriminator: ${tag}`);
		}
	}
}));
var NumberValue = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return NumberValue.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		NumberValue.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.value !== void 0) {
			view.writeByte(1);
			view.writeUint32(record.value);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return NumberValue(NumberValue.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.value = view.readUint32();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var PairValue = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return PairValue.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		PairValue.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.width !== void 0) {
			view.writeByte(1);
			view.writeFloat32(record.width);
		}
		if (record.height !== void 0) {
			view.writeByte(2);
			view.writeFloat32(record.height);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return PairValue(PairValue.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.width = view.readFloat32();
				break;
			case 2:
				message.height = view.readFloat32();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var BoolValue = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return BoolValue.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		BoolValue.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.value !== void 0) {
			view.writeByte(1);
			view.writeByte(Number(record.value));
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return BoolValue(BoolValue.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.value = !!view.readByte();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var TextValue = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return TextValue.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		TextValue.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.value !== void 0) {
			view.writeByte(1);
			view.writeString(record.value);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return TextValue(TextValue.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.value = view.readString();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var PathsValue = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return PathsValue.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		PathsValue.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.paths !== void 0) {
			view.writeByte(1);
			{
				const length0 = record.paths.length;
				view.writeUint32(length0);
				for (let i0 = 0; i0 < length0; i0++) view.writeString(record.paths[i0]);
			}
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return PathsValue(PathsValue.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				{
					const length0 = view.readUint32();
					message.paths = [];
					for (let i0 = 0; i0 < length0; i0++) {
						let x0;
						x0 = view.readString();
						message.paths[i0] = x0;
					}
				}
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var FileTextValue = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return FileTextValue.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		FileTextValue.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.value !== void 0) {
			view.writeByte(1);
			view.writeString(record.value);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return FileTextValue(FileTextValue.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.value = view.readString();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var ImageValue = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return ImageValue.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		ImageValue.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.format !== void 0) {
			view.writeByte(1);
			view.writeUint32(record.format);
		}
		if (record.bytes !== void 0) {
			view.writeByte(2);
			view.writeBytes(record.bytes);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return ImageValue(ImageValue.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.format = view.readUint32();
				break;
			case 2:
				message.bytes = view.readBytes();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var BoundsValue = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return BoundsValue.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		BoundsValue.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.x !== void 0) {
			view.writeByte(1);
			view.writeFloat32(record.x);
		}
		if (record.y !== void 0) {
			view.writeByte(2);
			view.writeFloat32(record.y);
		}
		if (record.width !== void 0) {
			view.writeByte(3);
			view.writeFloat32(record.width);
		}
		if (record.height !== void 0) {
			view.writeByte(4);
			view.writeFloat32(record.height);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return BoundsValue(BoundsValue.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.x = view.readFloat32();
				break;
			case 2:
				message.y = view.readFloat32();
				break;
			case 3:
				message.width = view.readFloat32();
				break;
			case 4:
				message.height = view.readFloat32();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var WindowStateValue = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return WindowStateValue.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		WindowStateValue.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.fullscreen !== void 0) {
			view.writeByte(1);
			view.writeByte(Number(record.fullscreen));
		}
		if (record.maximized !== void 0) {
			view.writeByte(2);
			view.writeByte(Number(record.maximized));
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return WindowStateValue(WindowStateValue.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.fullscreen = !!view.readByte();
				break;
			case 2:
				message.maximized = !!view.readByte();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var ScrollOffsetValue = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return ScrollOffsetValue.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		ScrollOffsetValue.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.value !== void 0) {
			view.writeByte(1);
			view.writeFloat32(record.value);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return ScrollOffsetValue(ScrollOffsetValue.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.value = view.readFloat32();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var BytesValue = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return BytesValue.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		BytesValue.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.value !== void 0) {
			view.writeByte(1);
			view.writeBytes(record.value);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return BytesValue(BytesValue.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.value = view.readBytes();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var CommandValue = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return CommandValue.encode(this);
		}
	};
}, {
	fromNumberValue(value) {
		return CommandValue({
			tag: 1,
			value
		});
	},
	fromPairValue(value) {
		return CommandValue({
			tag: 2,
			value
		});
	},
	fromBoolValue(value) {
		return CommandValue({
			tag: 3,
			value
		});
	},
	fromTextValue(value) {
		return CommandValue({
			tag: 4,
			value
		});
	},
	fromPathsValue(value) {
		return CommandValue({
			tag: 5,
			value
		});
	},
	fromFileTextValue(value) {
		return CommandValue({
			tag: 6,
			value
		});
	},
	fromImageValue(value) {
		return CommandValue({
			tag: 7,
			value
		});
	},
	fromBoundsValue(value) {
		return CommandValue({
			tag: 8,
			value
		});
	},
	fromWindowStateValue(value) {
		return CommandValue({
			tag: 9,
			value
		});
	},
	fromScrollOffsetValue(value) {
		return CommandValue({
			tag: 10,
			value
		});
	},
	fromBytesValue(value) {
		return CommandValue({
			tag: 11,
			value
		});
	},
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		CommandValue.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length + 1;
		view.writeByte(record.tag);
		switch (record.tag) {
			case 1:
				NumberValue.encodeInto(record.value, view);
				break;
			case 2:
				PairValue.encodeInto(record.value, view);
				break;
			case 3:
				BoolValue.encodeInto(record.value, view);
				break;
			case 4:
				TextValue.encodeInto(record.value, view);
				break;
			case 5:
				PathsValue.encodeInto(record.value, view);
				break;
			case 6:
				FileTextValue.encodeInto(record.value, view);
				break;
			case 7:
				ImageValue.encodeInto(record.value, view);
				break;
			case 8:
				BoundsValue.encodeInto(record.value, view);
				break;
			case 9:
				WindowStateValue.encodeInto(record.value, view);
				break;
			case 10:
				ScrollOffsetValue.encodeInto(record.value, view);
				break;
			case 11: BytesValue.encodeInto(record.value, view);
		}
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return CommandValue(CommandValue.readFrom(view));
	},
	readFrom(view) {
		const length = view.readMessageLength();
		const end = view.index + 1 + length;
		const tag = view.readByte();
		switch (tag) {
			case 1: return {
				tag: 1,
				value: NumberValue.readFrom(view)
			};
			case 2: return {
				tag: 2,
				value: PairValue.readFrom(view)
			};
			case 3: return {
				tag: 3,
				value: BoolValue.readFrom(view)
			};
			case 4: return {
				tag: 4,
				value: TextValue.readFrom(view)
			};
			case 5: return {
				tag: 5,
				value: PathsValue.readFrom(view)
			};
			case 6: return {
				tag: 6,
				value: FileTextValue.readFrom(view)
			};
			case 7: return {
				tag: 7,
				value: ImageValue.readFrom(view)
			};
			case 8: return {
				tag: 8,
				value: BoundsValue.readFrom(view)
			};
			case 9: return {
				tag: 9,
				value: WindowStateValue.readFrom(view)
			};
			case 10: return {
				tag: 10,
				value: ScrollOffsetValue.readFrom(view)
			};
			case 11: return {
				tag: 11,
				value: BytesValue.readFrom(view)
			};
			default:
				view.index = end;
				throw new BebopRuntimeError(`Unknown union discriminator: ${tag}`);
		}
	}
}));
var TextInputEventData = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return TextInputEventData.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		TextInputEventData.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.text !== void 0) {
			view.writeByte(1);
			view.writeString(record.text);
		}
		if (record.selectionStart !== void 0) {
			view.writeByte(2);
			view.writeUint32(record.selectionStart);
		}
		if (record.selectionEnd !== void 0) {
			view.writeByte(3);
			view.writeUint32(record.selectionEnd);
		}
		if (record.markedStart !== void 0) {
			view.writeByte(4);
			view.writeUint32(record.markedStart);
		}
		if (record.markedEnd !== void 0) {
			view.writeByte(5);
			view.writeUint32(record.markedEnd);
		}
		if (record.editSeq !== void 0) {
			view.writeByte(6);
			view.writeUint32(record.editSeq);
		}
		if (record.reversed !== void 0) {
			view.writeByte(7);
			view.writeByte(Number(record.reversed));
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return TextInputEventData(TextInputEventData.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.text = view.readString();
				break;
			case 2:
				message.selectionStart = view.readUint32();
				break;
			case 3:
				message.selectionEnd = view.readUint32();
				break;
			case 4:
				message.markedStart = view.readUint32();
				break;
			case 5:
				message.markedEnd = view.readUint32();
				break;
			case 6:
				message.editSeq = view.readUint32();
				break;
			case 7:
				message.reversed = !!view.readByte();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var CommandResult = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return CommandResult.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		CommandResult.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.requestId !== void 0) {
			view.writeByte(1);
			view.writeUint32(record.requestId);
		}
		if (record.command !== void 0) {
			view.writeByte(2);
			view.writeUint32(record.command);
		}
		if (record.nodeId !== void 0) {
			view.writeByte(3);
			view.writeUint32(record.nodeId);
		}
		if (record.success !== void 0) {
			view.writeByte(4);
			view.writeByte(Number(record.success));
		}
		if (record.error !== void 0) {
			view.writeByte(5);
			view.writeString(record.error);
		}
		if (record.value !== void 0) {
			view.writeByte(6);
			CommandValue.encodeInto(record.value, view);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return CommandResult(CommandResult.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.requestId = view.readUint32();
				break;
			case 2:
				message.command = view.readUint32();
				break;
			case 3:
				message.nodeId = view.readUint32();
				break;
			case 4:
				message.success = !!view.readByte();
				break;
			case 5:
				message.error = view.readString();
				break;
			case 6:
				message.value = CommandValue.readFrom(view);
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var VisibleRangeEvent = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return VisibleRangeEvent.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		VisibleRangeEvent.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.start !== void 0) {
			view.writeByte(1);
			view.writeUint32(record.start);
		}
		if (record.end !== void 0) {
			view.writeByte(2);
			view.writeUint32(record.end);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return VisibleRangeEvent(VisibleRangeEvent.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.start = view.readUint32();
				break;
			case 2:
				message.end = view.readUint32();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var AnimationCompleteEvent = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return AnimationCompleteEvent.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		AnimationCompleteEvent.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.generation !== void 0) {
			view.writeByte(1);
			view.writeUint32(record.generation);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return AnimationCompleteEvent(AnimationCompleteEvent.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.generation = view.readUint32();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var KeyEvent = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return KeyEvent.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		KeyEvent.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.key !== void 0) {
			view.writeByte(1);
			view.writeString(record.key);
		}
		if (record.modifiers !== void 0) {
			view.writeByte(2);
			{
				const length0 = record.modifiers.length;
				view.writeUint32(length0);
				for (let i0 = 0; i0 < length0; i0++) view.writeString(record.modifiers[i0]);
			}
		}
		if (record.action !== void 0) {
			view.writeByte(3);
			view.writeUint32(record.action);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return KeyEvent(KeyEvent.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.key = view.readString();
				break;
			case 2:
				{
					const length0 = view.readUint32();
					message.modifiers = [];
					for (let i0 = 0; i0 < length0; i0++) {
						let x0;
						x0 = view.readString();
						message.modifiers[i0] = x0;
					}
				}
				break;
			case 3:
				message.action = view.readUint32();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var PointerEvent = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return PointerEvent.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		PointerEvent.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.button !== void 0) {
			view.writeByte(1);
			view.writeUint32(record.button);
		}
		if (record.modifiers !== void 0) {
			view.writeByte(2);
			{
				const length0 = record.modifiers.length;
				view.writeUint32(length0);
				for (let i0 = 0; i0 < length0; i0++) view.writeString(record.modifiers[i0]);
			}
		}
		if (record.action !== void 0) {
			view.writeByte(3);
			view.writeUint32(record.action);
		}
		if (record.clickCount !== void 0) {
			view.writeByte(4);
			view.writeUint32(record.clickCount);
		}
		if (record.x !== void 0) {
			view.writeByte(5);
			view.writeFloat32(record.x);
		}
		if (record.y !== void 0) {
			view.writeByte(6);
			view.writeFloat32(record.y);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return PointerEvent(PointerEvent.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.button = view.readUint32();
				break;
			case 2:
				{
					const length0 = view.readUint32();
					message.modifiers = [];
					for (let i0 = 0; i0 < length0; i0++) {
						let x0;
						x0 = view.readString();
						message.modifiers[i0] = x0;
					}
				}
				break;
			case 3:
				message.action = view.readUint32();
				break;
			case 4:
				message.clickCount = view.readUint32();
				break;
			case 5:
				message.x = view.readFloat32();
				break;
			case 6:
				message.y = view.readFloat32();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var PointerMoveEvent = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return PointerMoveEvent.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		PointerMoveEvent.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.modifiers !== void 0) {
			view.writeByte(1);
			{
				const length0 = record.modifiers.length;
				view.writeUint32(length0);
				for (let i0 = 0; i0 < length0; i0++) view.writeString(record.modifiers[i0]);
			}
		}
		if (record.x !== void 0) {
			view.writeByte(2);
			view.writeFloat32(record.x);
		}
		if (record.y !== void 0) {
			view.writeByte(3);
			view.writeFloat32(record.y);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return PointerMoveEvent(PointerMoveEvent.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				{
					const length0 = view.readUint32();
					message.modifiers = [];
					for (let i0 = 0; i0 < length0; i0++) {
						let x0;
						x0 = view.readString();
						message.modifiers[i0] = x0;
					}
				}
				break;
			case 2:
				message.x = view.readFloat32();
				break;
			case 3:
				message.y = view.readFloat32();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var ScrollEvent = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return ScrollEvent.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		ScrollEvent.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.deltaKind !== void 0) {
			view.writeByte(1);
			view.writeUint32(record.deltaKind);
		}
		if (record.dx !== void 0) {
			view.writeByte(2);
			view.writeFloat32(record.dx);
		}
		if (record.dy !== void 0) {
			view.writeByte(3);
			view.writeFloat32(record.dy);
		}
		if (record.x !== void 0) {
			view.writeByte(4);
			view.writeFloat32(record.x);
		}
		if (record.y !== void 0) {
			view.writeByte(5);
			view.writeFloat32(record.y);
		}
		if (record.modifiers !== void 0) {
			view.writeByte(6);
			{
				const length0 = record.modifiers.length;
				view.writeUint32(length0);
				for (let i0 = 0; i0 < length0; i0++) view.writeString(record.modifiers[i0]);
			}
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return ScrollEvent(ScrollEvent.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.deltaKind = view.readUint32();
				break;
			case 2:
				message.dx = view.readFloat32();
				break;
			case 3:
				message.dy = view.readFloat32();
				break;
			case 4:
				message.x = view.readFloat32();
				break;
			case 5:
				message.y = view.readFloat32();
				break;
			case 6:
				{
					const length0 = view.readUint32();
					message.modifiers = [];
					for (let i0 = 0; i0 < length0; i0++) {
						let x0;
						x0 = view.readString();
						message.modifiers[i0] = x0;
					}
				}
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var SubmitEvent = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return SubmitEvent.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		SubmitEvent.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.text !== void 0) {
			view.writeByte(1);
			view.writeString(record.text);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return SubmitEvent(SubmitEvent.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.text = view.readString();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var WindowResizeEvent = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return WindowResizeEvent.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		WindowResizeEvent.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.width !== void 0) {
			view.writeByte(1);
			view.writeFloat32(record.width);
		}
		if (record.height !== void 0) {
			view.writeByte(2);
			view.writeFloat32(record.height);
		}
		if (record.scaleFactor !== void 0) {
			view.writeByte(3);
			view.writeFloat32(record.scaleFactor);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return WindowResizeEvent(WindowResizeEvent.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.width = view.readFloat32();
				break;
			case 2:
				message.height = view.readFloat32();
				break;
			case 3:
				message.scaleFactor = view.readFloat32();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var WindowActivationEvent = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return WindowActivationEvent.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		WindowActivationEvent.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.active !== void 0) {
			view.writeByte(1);
			view.writeByte(Number(record.active));
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return WindowActivationEvent(WindowActivationEvent.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.active = !!view.readByte();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var ActionEvent = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return ActionEvent.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		ActionEvent.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.action !== void 0) {
			view.writeByte(1);
			view.writeString(record.action);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return ActionEvent(ActionEvent.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.action = view.readString();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var WindowAppearanceEvent = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return WindowAppearanceEvent.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		WindowAppearanceEvent.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.appearance !== void 0) {
			view.writeByte(1);
			view.writeByte(record.appearance);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return WindowAppearanceEvent(WindowAppearanceEvent.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.appearance = view.readByte();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var LayoutEvent = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return LayoutEvent.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		LayoutEvent.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.x !== void 0) {
			view.writeByte(1);
			view.writeFloat32(record.x);
		}
		if (record.y !== void 0) {
			view.writeByte(2);
			view.writeFloat32(record.y);
		}
		if (record.width !== void 0) {
			view.writeByte(3);
			view.writeFloat32(record.width);
		}
		if (record.height !== void 0) {
			view.writeByte(4);
			view.writeFloat32(record.height);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return LayoutEvent(LayoutEvent.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.x = view.readFloat32();
				break;
			case 2:
				message.y = view.readFloat32();
				break;
			case 3:
				message.width = view.readFloat32();
				break;
			case 4:
				message.height = view.readFloat32();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var DragOverEvent = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return DragOverEvent.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		DragOverEvent.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.dragType !== void 0) {
			view.writeByte(1);
			view.writeString(record.dragType);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return DragOverEvent(DragOverEvent.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.dragType = view.readString();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var DragDropEvent = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return DragDropEvent.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		DragDropEvent.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.dragType !== void 0) {
			view.writeByte(1);
			view.writeString(record.dragType);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return DragDropEvent(DragDropEvent.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.dragType = view.readString();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var ExternalFileDropEvent = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return ExternalFileDropEvent.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		ExternalFileDropEvent.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.paths !== void 0) {
			view.writeByte(1);
			{
				const length0 = record.paths.length;
				view.writeUint32(length0);
				for (let i0 = 0; i0 < length0; i0++) view.writeString(record.paths[i0]);
			}
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return ExternalFileDropEvent(ExternalFileDropEvent.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				{
					const length0 = view.readUint32();
					message.paths = [];
					for (let i0 = 0; i0 < length0; i0++) {
						let x0;
						x0 = view.readString();
						message.paths[i0] = x0;
					}
				}
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var NotificationResponseEvent = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return NotificationResponseEvent.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		NotificationResponseEvent.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.tag !== void 0) {
			view.writeByte(1);
			view.writeString(record.tag);
		}
		if (record.actionId !== void 0) {
			view.writeByte(2);
			view.writeString(record.actionId);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return NotificationResponseEvent(NotificationResponseEvent.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.tag = view.readString();
				break;
			case 2:
				message.actionId = view.readString();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var PointerDownOutsideEvent = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return PointerDownOutsideEvent.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		PointerDownOutsideEvent.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.x !== void 0) {
			view.writeByte(1);
			view.writeFloat32(record.x);
		}
		if (record.y !== void 0) {
			view.writeByte(2);
			view.writeFloat32(record.y);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return PointerDownOutsideEvent(PointerDownOutsideEvent.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.x = view.readFloat32();
				break;
			case 2:
				message.y = view.readFloat32();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var CloseRequestedEvent = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return CloseRequestedEvent.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		CloseRequestedEvent.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.requestId !== void 0) {
			view.writeByte(1);
			view.writeUint32(record.requestId);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return CloseRequestedEvent(CloseRequestedEvent.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.requestId = view.readUint32();
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var ExtensionEvent = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return ExtensionEvent.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		ExtensionEvent.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.eventId !== void 0) {
			view.writeByte(1);
			view.writeUint32(record.eventId);
		}
		if (record.fields !== void 0) {
			view.writeByte(2);
			{
				const length0 = record.fields.length;
				view.writeUint32(length0);
				for (let i0 = 0; i0 < length0; i0++) ExtensionField.encodeInto(record.fields[i0], view);
			}
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return ExtensionEvent(ExtensionEvent.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.eventId = view.readUint32();
				break;
			case 2:
				{
					const length0 = view.readUint32();
					message.fields = [];
					for (let i0 = 0; i0 < length0; i0++) {
						let x0;
						x0 = ExtensionField.readFrom(view);
						message.fields[i0] = x0;
					}
				}
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var ApplicationActivationEvent = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return ApplicationActivationEvent.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		ApplicationActivationEvent.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.targetSurfaceId !== void 0) {
			view.writeByte(1);
			view.writeUint32(record.targetSurfaceId);
		}
		if (record.reason !== void 0) {
			view.writeByte(2);
			view.writeString(record.reason);
		}
		if (record.urls !== void 0) {
			view.writeByte(3);
			{
				const length0 = record.urls.length;
				view.writeUint32(length0);
				for (let i0 = 0; i0 < length0; i0++) view.writeString(record.urls[i0]);
			}
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return ApplicationActivationEvent(ApplicationActivationEvent.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.targetSurfaceId = view.readUint32();
				break;
			case 2:
				message.reason = view.readString();
				break;
			case 3:
				{
					const length0 = view.readUint32();
					message.urls = [];
					for (let i0 = 0; i0 < length0; i0++) {
						let x0;
						x0 = view.readString();
						message.urls[i0] = x0;
					}
				}
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
var EventPayload = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return EventPayload.encode(this);
		}
	};
}, {
	fromTextInputEventData(value) {
		return EventPayload({
			tag: 1,
			value
		});
	},
	fromCommandResult(value) {
		return EventPayload({
			tag: 2,
			value
		});
	},
	fromVisibleRangeEvent(value) {
		return EventPayload({
			tag: 3,
			value
		});
	},
	fromAnimationCompleteEvent(value) {
		return EventPayload({
			tag: 4,
			value
		});
	},
	fromKeyEvent(value) {
		return EventPayload({
			tag: 5,
			value
		});
	},
	fromPointerEvent(value) {
		return EventPayload({
			tag: 6,
			value
		});
	},
	fromPointerMoveEvent(value) {
		return EventPayload({
			tag: 7,
			value
		});
	},
	fromScrollEvent(value) {
		return EventPayload({
			tag: 8,
			value
		});
	},
	fromSubmitEvent(value) {
		return EventPayload({
			tag: 9,
			value
		});
	},
	fromWindowResizeEvent(value) {
		return EventPayload({
			tag: 10,
			value
		});
	},
	fromWindowActivationEvent(value) {
		return EventPayload({
			tag: 11,
			value
		});
	},
	fromActionEvent(value) {
		return EventPayload({
			tag: 12,
			value
		});
	},
	fromWindowAppearanceEvent(value) {
		return EventPayload({
			tag: 13,
			value
		});
	},
	fromLayoutEvent(value) {
		return EventPayload({
			tag: 14,
			value
		});
	},
	fromDragOverEvent(value) {
		return EventPayload({
			tag: 15,
			value
		});
	},
	fromDragDropEvent(value) {
		return EventPayload({
			tag: 16,
			value
		});
	},
	fromExternalFileDropEvent(value) {
		return EventPayload({
			tag: 17,
			value
		});
	},
	fromNotificationResponseEvent(value) {
		return EventPayload({
			tag: 18,
			value
		});
	},
	fromPointerDownOutsideEvent(value) {
		return EventPayload({
			tag: 19,
			value
		});
	},
	fromCloseRequestedEvent(value) {
		return EventPayload({
			tag: 20,
			value
		});
	},
	fromExtensionEvent(value) {
		return EventPayload({
			tag: 21,
			value
		});
	},
	fromApplicationActivationEvent(value) {
		return EventPayload({
			tag: 22,
			value
		});
	},
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		EventPayload.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length + 1;
		view.writeByte(record.tag);
		switch (record.tag) {
			case 1:
				TextInputEventData.encodeInto(record.value, view);
				break;
			case 2:
				CommandResult.encodeInto(record.value, view);
				break;
			case 3:
				VisibleRangeEvent.encodeInto(record.value, view);
				break;
			case 4:
				AnimationCompleteEvent.encodeInto(record.value, view);
				break;
			case 5:
				KeyEvent.encodeInto(record.value, view);
				break;
			case 6:
				PointerEvent.encodeInto(record.value, view);
				break;
			case 7:
				PointerMoveEvent.encodeInto(record.value, view);
				break;
			case 8:
				ScrollEvent.encodeInto(record.value, view);
				break;
			case 9:
				SubmitEvent.encodeInto(record.value, view);
				break;
			case 10:
				WindowResizeEvent.encodeInto(record.value, view);
				break;
			case 11:
				WindowActivationEvent.encodeInto(record.value, view);
				break;
			case 12:
				ActionEvent.encodeInto(record.value, view);
				break;
			case 13:
				WindowAppearanceEvent.encodeInto(record.value, view);
				break;
			case 14:
				LayoutEvent.encodeInto(record.value, view);
				break;
			case 15:
				DragOverEvent.encodeInto(record.value, view);
				break;
			case 16:
				DragDropEvent.encodeInto(record.value, view);
				break;
			case 17:
				ExternalFileDropEvent.encodeInto(record.value, view);
				break;
			case 18:
				NotificationResponseEvent.encodeInto(record.value, view);
				break;
			case 19:
				PointerDownOutsideEvent.encodeInto(record.value, view);
				break;
			case 20:
				CloseRequestedEvent.encodeInto(record.value, view);
				break;
			case 21:
				ExtensionEvent.encodeInto(record.value, view);
				break;
			case 22: ApplicationActivationEvent.encodeInto(record.value, view);
		}
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return EventPayload(EventPayload.readFrom(view));
	},
	readFrom(view) {
		const length = view.readMessageLength();
		const end = view.index + 1 + length;
		const tag = view.readByte();
		switch (tag) {
			case 1: return {
				tag: 1,
				value: TextInputEventData.readFrom(view)
			};
			case 2: return {
				tag: 2,
				value: CommandResult.readFrom(view)
			};
			case 3: return {
				tag: 3,
				value: VisibleRangeEvent.readFrom(view)
			};
			case 4: return {
				tag: 4,
				value: AnimationCompleteEvent.readFrom(view)
			};
			case 5: return {
				tag: 5,
				value: KeyEvent.readFrom(view)
			};
			case 6: return {
				tag: 6,
				value: PointerEvent.readFrom(view)
			};
			case 7: return {
				tag: 7,
				value: PointerMoveEvent.readFrom(view)
			};
			case 8: return {
				tag: 8,
				value: ScrollEvent.readFrom(view)
			};
			case 9: return {
				tag: 9,
				value: SubmitEvent.readFrom(view)
			};
			case 10: return {
				tag: 10,
				value: WindowResizeEvent.readFrom(view)
			};
			case 11: return {
				tag: 11,
				value: WindowActivationEvent.readFrom(view)
			};
			case 12: return {
				tag: 12,
				value: ActionEvent.readFrom(view)
			};
			case 13: return {
				tag: 13,
				value: WindowAppearanceEvent.readFrom(view)
			};
			case 14: return {
				tag: 14,
				value: LayoutEvent.readFrom(view)
			};
			case 15: return {
				tag: 15,
				value: DragOverEvent.readFrom(view)
			};
			case 16: return {
				tag: 16,
				value: DragDropEvent.readFrom(view)
			};
			case 17: return {
				tag: 17,
				value: ExternalFileDropEvent.readFrom(view)
			};
			case 18: return {
				tag: 18,
				value: NotificationResponseEvent.readFrom(view)
			};
			case 19: return {
				tag: 19,
				value: PointerDownOutsideEvent.readFrom(view)
			};
			case 20: return {
				tag: 20,
				value: CloseRequestedEvent.readFrom(view)
			};
			case 21: return {
				tag: 21,
				value: ExtensionEvent.readFrom(view)
			};
			case 22: return {
				tag: 22,
				value: ApplicationActivationEvent.readFrom(view)
			};
			default:
				view.index = end;
				throw new BebopRuntimeError(`Unknown union discriminator: ${tag}`);
		}
	}
}));
var Envelope = /*#__PURE__*/ Object.freeze(/*#__PURE__*/ Object.assign((data) => {
	return {
		...data,
		encode() {
			return Envelope.encode(this);
		}
	};
}, {
	encode(record) {
		const view = BebopView.getInstance();
		view.startWriting();
		Envelope.encodeInto(record, view);
		return view.toArray();
	},
	encodeInto(record, view) {
		const pos = view.reserveMessageLength();
		const start = view.length;
		if (record.protocolVersion !== void 0) {
			view.writeByte(1);
			view.writeUint32(record.protocolVersion);
		}
		if (record.body !== void 0) {
			view.writeByte(2);
			Body.encodeInto(record.body, view);
		}
		view.writeByte(0);
		const end = view.length;
		view.fillMessageLength(pos, end - start);
	},
	decode(buffer) {
		const view = BebopView.getInstance();
		view.startReading(buffer);
		return Envelope(Envelope.readFrom(view));
	},
	readFrom(view) {
		const message = {};
		const length = view.readMessageLength();
		const end = view.index + length;
		while (true) switch (view.readByte()) {
			case 0: return message;
			case 1:
				message.protocolVersion = view.readUint32();
				break;
			case 2:
				message.body = Body.readFrom(view);
				break;
			default:
				view.index = end;
				return message;
		}
	}
}));
//#endregion
//#region packages/solid-gpui/src/style.ts
var COLOR_PATTERN = /^#[0-9a-fA-F]{6}(?:[0-9a-fA-F]{2})?$/;
var STYLE_KEYS = {
	borderTopColor: true,
	borderRightColor: true,
	borderBottomColor: true,
	borderLeftColor: true,
	linearGradient: true,
	paddingTop: true,
	paddingRight: true,
	paddingBottom: true,
	paddingLeft: true,
	borderTopWidth: true,
	borderRightWidth: true,
	borderBottomWidth: true,
	borderLeftWidth: true,
	borderTopLeftRadius: true,
	borderTopRightRadius: true,
	borderBottomRightRadius: true,
	borderBottomLeftRadius: true,
	widthPercent: true,
	heightPercent: true,
	flexWrap: true,
	gridColumns: true,
	gridRows: true,
	gridColumnSpan: true,
	gridRowSpan: true,
	width: true,
	height: true,
	flexDirection: true,
	flexGrow: true,
	padding: true,
	gap: true,
	justifyContent: true,
	alignItems: true,
	borderRadius: true,
	borderWidth: true,
	borderColor: true,
	fontSize: true,
	fontWeight: true,
	overflow: true,
	lineClamp: true,
	textOverflow: true,
	marginTop: true,
	marginRight: true,
	marginBottom: true,
	marginLeft: true,
	fontStyle: true,
	textDecoration: true,
	lineHeight: true,
	minWidth: true,
	maxWidth: true,
	minHeight: true,
	maxHeight: true,
	flexShrink: true,
	alignSelf: true,
	position: true,
	boxShadow: true,
	fontFamily: true,
	left: true,
	top: true,
	textAlign: true,
	right: true,
	cursor: true,
	bottom: true,
	backgroundColor: true,
	color: true,
	opacity: true,
	transition: true
};
function assertNumber(name, value, nonNegative) {
	if (typeof value !== "number" || !Number.isFinite(value) || nonNegative && value < 0) throw new TypeError(`${name} must be a finite${nonNegative ? " non-negative" : ""} number`);
	if (!Number.isFinite(Math.fround(value))) throw new TypeError(`${name} must be representable as float32`);
}
var BOX_SHADOW_KEYS = {
	offsetX: true,
	offsetY: true,
	blurRadius: true,
	spreadRadius: true,
	color: true,
	inset: true
};
function assertF32Number(name, value, nonNegative) {
	assertNumber(name, value, nonNegative);
	if (!Number.isFinite(Math.fround(value))) throw new TypeError(`${name} must be representable as f32`);
}
function validateBoxShadow(value, index) {
	if (value === null || typeof value !== "object" || Array.isArray(value)) throw new TypeError(`boxShadow[${index}] must be an object`);
	for (const key of Object.keys(value)) if (!BOX_SHADOW_KEYS[key]) throw new TypeError(`Unsupported boxShadow field: ${key}`);
	const shadow = value;
	assertF32Number(`boxShadow[${index}].offsetX`, shadow.offsetX, false);
	assertF32Number(`boxShadow[${index}].offsetY`, shadow.offsetY, false);
	assertF32Number(`boxShadow[${index}].blurRadius`, shadow.blurRadius, true);
	assertF32Number(`boxShadow[${index}].spreadRadius`, shadow.spreadRadius, true);
	if (typeof shadow.color !== "string" || !COLOR_PATTERN.test(shadow.color)) throw new TypeError(`boxShadow[${index}].color must be #RRGGBB or #RRGGBBAA`);
	if (shadow.inset !== void 0 && typeof shadow.inset !== "boolean") throw new TypeError(`boxShadow[${index}].inset must be a boolean`);
	return shadow;
}
function validateBoxShadowInput(value) {
	if (Array.isArray(value)) {
		if (value.length !== 2) throw new TypeError("boxShadow must contain one or two shadows");
		validateBoxShadow(value[0], 0);
		validateBoxShadow(value[1], 1);
		return value;
	}
	return validateBoxShadow(value, 0);
}
function validateFontFamily(value) {
	if (typeof value !== "string" || value.length === 0 || [...value].length > 64 || [...value].some((character) => /\p{Cc}/u.test(character))) throw new TypeError("fontFamily must be a non-empty string of at most 64 characters");
}
function validateStyle(value) {
	if (value == null) return value;
	if (typeof value !== "object" || Array.isArray(value)) throw new TypeError("style must be an object, null, or undefined");
	for (const key of Object.keys(value)) if (!STYLE_KEYS[key]) throw new TypeError(`Unsupported style field: ${key}`);
	const style = value;
	if (style.linearGradient !== void 0) {
		const gradient = style.linearGradient;
		if (!gradient || typeof gradient !== "object" || Object.keys(gradient).some((key) => key !== "angle" && key !== "stops")) throw new TypeError("linearGradient is invalid");
		assertNumber("linearGradient.angle", gradient.angle, true);
		if (gradient.angle > 360 || !Array.isArray(gradient.stops) || gradient.stops.length !== 2) throw new TypeError("linearGradient requires an angle in [0, 360] and exactly two stops");
		for (const stop of gradient.stops) {
			if (!stop || typeof stop !== "object" || Object.keys(stop).some((key) => key !== "color" && key !== "position") || typeof stop.color !== "string" || !COLOR_PATTERN.test(stop.color)) throw new TypeError("linearGradient stop is invalid");
			assertNumber("linearGradient stop position", stop.position, true);
			if (stop.position > 1) throw new TypeError("linearGradient stop position must be in [0, 1]");
		}
		if (gradient.stops[0].position >= gradient.stops[1].position) throw new TypeError("linearGradient stops must be strictly increasing");
	}
	if (style.paddingTop !== void 0) assertNumber("paddingTop", style.paddingTop, true);
	if (style.paddingRight !== void 0) assertNumber("paddingRight", style.paddingRight, true);
	if (style.paddingBottom !== void 0) assertNumber("paddingBottom", style.paddingBottom, true);
	if (style.paddingLeft !== void 0) assertNumber("paddingLeft", style.paddingLeft, true);
	if (style.borderTopWidth !== void 0) assertNumber("borderTopWidth", style.borderTopWidth, true);
	if (style.borderRightWidth !== void 0) assertNumber("borderRightWidth", style.borderRightWidth, true);
	if (style.borderBottomWidth !== void 0) assertNumber("borderBottomWidth", style.borderBottomWidth, true);
	if (style.borderLeftWidth !== void 0) assertNumber("borderLeftWidth", style.borderLeftWidth, true);
	if (style.borderTopLeftRadius !== void 0) assertNumber("borderTopLeftRadius", style.borderTopLeftRadius, true);
	if (style.borderTopRightRadius !== void 0) assertNumber("borderTopRightRadius", style.borderTopRightRadius, true);
	if (style.borderBottomRightRadius !== void 0) assertNumber("borderBottomRightRadius", style.borderBottomRightRadius, true);
	if (style.borderBottomLeftRadius !== void 0) assertNumber("borderBottomLeftRadius", style.borderBottomLeftRadius, true);
	if (style.widthPercent !== void 0) assertNumber("widthPercent", style.widthPercent, true);
	if (style.heightPercent !== void 0) assertNumber("heightPercent", style.heightPercent, true);
	if (style.width !== void 0 && style.widthPercent !== void 0) throw new TypeError("width and widthPercent are mutually exclusive");
	if (style.height !== void 0 && style.heightPercent !== void 0) throw new TypeError("height and heightPercent are mutually exclusive");
	if (style.flexWrap !== void 0 && ![
		"nowrap",
		"wrap",
		"wrap-reverse"
	].includes(style.flexWrap)) throw new TypeError("flexWrap is invalid");
	if (style.width !== void 0) assertNumber("width", style.width, true);
	if (style.height !== void 0) assertNumber("height", style.height, true);
	if (style.position !== void 0 && style.position !== "relative" && style.position !== "absolute" && style.position !== "overlay") throw new TypeError("position must be relative, absolute, or overlay");
	if (style.position === "overlay" && (style.right !== void 0 || style.bottom !== void 0)) throw new TypeError("overlay position supports left and top offsets only");
	for (const [name, offset] of [
		["left", style.left],
		["top", style.top],
		["right", style.right],
		["bottom", style.bottom]
	]) if (offset !== void 0) assertNumber(name, offset, false);
	if (style.flexGrow !== void 0) assertNumber("flexGrow", style.flexGrow, true);
	if (style.padding !== void 0) assertNumber("padding", style.padding, true);
	if (style.gap !== void 0) assertNumber("gap", style.gap, true);
	for (const [name, amount] of [
		["marginTop", style.marginTop],
		["marginRight", style.marginRight],
		["marginBottom", style.marginBottom],
		["marginLeft", style.marginLeft],
		["lineHeight", style.lineHeight],
		["minWidth", style.minWidth],
		["maxWidth", style.maxWidth],
		["minHeight", style.minHeight],
		["maxHeight", style.maxHeight],
		["flexShrink", style.flexShrink]
	]) if (amount !== void 0) assertNumber(name, amount, true);
	if (style.justifyContent !== void 0 && ![
		"flex-start",
		"center",
		"flex-end",
		"space-between",
		"space-around",
		"space-evenly"
	].includes(style.justifyContent)) throw new TypeError("justifyContent is invalid");
	if (style.alignItems !== void 0 && ![
		"flex-start",
		"center",
		"flex-end",
		"stretch",
		"baseline"
	].includes(style.alignItems)) throw new TypeError("alignItems is invalid");
	if (style.borderRadius !== void 0) assertNumber("borderRadius", style.borderRadius, true);
	if (style.borderWidth !== void 0) assertNumber("borderWidth", style.borderWidth, true);
	if (style.fontSize !== void 0) {
		assertNumber("fontSize", style.fontSize, false);
		if (style.fontSize <= 0) throw new TypeError("fontSize must be positive");
	}
	if (style.fontWeight !== void 0 && ![
		"normal",
		"medium",
		"semibold",
		"bold",
		"heavy"
	].includes(style.fontWeight)) throw new TypeError("fontWeight is invalid");
	if (style.overflow !== void 0 && ![
		"visible",
		"hidden",
		"scroll"
	].includes(style.overflow)) throw new TypeError("overflow is invalid");
	if (style.lineClamp !== void 0) {
		assertNumber("lineClamp", style.lineClamp, true);
		if (!Number.isInteger(style.lineClamp) || style.lineClamp < 1 || style.lineClamp > 100) throw new TypeError("lineClamp must be an integer between 1 and 100");
	}
	if (style.textOverflow !== void 0 && style.textOverflow !== "clip" && style.textOverflow !== "ellipsis") throw new TypeError("textOverflow is invalid");
	if (style.fontStyle !== void 0 && style.fontStyle !== "normal" && style.fontStyle !== "italic") throw new TypeError("fontStyle is invalid");
	if (style.textDecoration !== void 0 && style.textDecoration !== "none" && style.textDecoration !== "underline" && style.textDecoration !== "lineThrough") throw new TypeError("textDecoration is invalid");
	if (style.alignSelf !== void 0 && ![
		"start",
		"end",
		"flex-start",
		"flex-end",
		"center",
		"baseline",
		"stretch"
	].includes(style.alignSelf)) throw new TypeError("alignSelf is invalid");
	if (style.cursor !== void 0 && ![
		"default",
		"text",
		"pointer",
		"grab",
		"grabbing",
		"not-allowed",
		"context-menu",
		"crosshair",
		"vertical-text",
		"alias",
		"copy",
		"no-drop",
		"move",
		"ew-resize",
		"ns-resize",
		"nesw-resize",
		"nwse-resize",
		"col-resize",
		"row-resize"
	].includes(style.cursor)) throw new TypeError("cursor is invalid");
	for (const key of [
		"gridColumns",
		"gridRows",
		"gridColumnSpan",
		"gridRowSpan"
	]) {
		const value = style[key];
		if (value !== void 0 && (!Number.isInteger(value) || value < 1 || value > 64)) throw new TypeError(`${key} must be an integer from 1 through 64`);
	}
	if ((style.gridColumns !== void 0 || style.gridRows !== void 0) && style.flexDirection !== void 0) throw new TypeError("grid tracks and flexDirection cannot be combined");
	if (style.opacity !== void 0) {
		assertNumber("opacity", style.opacity, true);
		if (style.opacity > 1) throw new TypeError("opacity must be between 0 and 1");
	}
	if (style.flexDirection !== void 0 && ![
		"row",
		"column",
		"row-reverse",
		"column-reverse"
	].includes(style.flexDirection)) throw new TypeError("flexDirection is invalid");
	if (style.textAlign !== void 0 && ![
		"left",
		"center",
		"right"
	].includes(style.textAlign)) throw new TypeError("textAlign is invalid");
	for (const key of [
		"borderColor",
		"backgroundColor",
		"color",
		"borderTopColor",
		"borderRightColor",
		"borderBottomColor",
		"borderLeftColor"
	]) {
		const color = style[key];
		if (color !== void 0 && (typeof color !== "string" || !COLOR_PATTERN.test(color))) throw new TypeError(`${key} must be #RRGGBB or #RRGGBBAA`);
	}
	if (style.transition !== void 0) {
		const transition = style.transition;
		if (transition === null || typeof transition !== "object" || Array.isArray(transition)) throw new TypeError("transition must be an object");
		assertNumber("transition.durationMs", transition.durationMs, true);
		if (!Number.isInteger(transition.durationMs) || transition.durationMs > 4294967295) throw new TypeError("transition.durationMs must be a u32");
		if (transition.delayMs !== void 0) {
			assertNumber("transition.delayMs", transition.delayMs, true);
			if (!Number.isInteger(transition.delayMs) || transition.delayMs > 4294967295) throw new TypeError("transition.delayMs must be a u32");
		}
		if (transition.easing !== void 0 && transition.easing !== "linear" && transition.easing !== "easeIn" && transition.easing !== "easeOut" && transition.easing !== "easeInOut") throw new TypeError("transition.easing is invalid");
		if (transition.onComplete !== void 0 && typeof transition.onComplete !== "function") throw new TypeError("transition.onComplete must be a function");
		if (transition.properties !== void 0) {
			if (!Array.isArray(transition.properties) || transition.properties.length === 0) throw new TypeError("transition.properties must not be empty");
			const seen = /* @__PURE__ */ new Set();
			for (const property of transition.properties) {
				if (property !== "opacity" && property !== "backgroundColor" && property !== "width" && property !== "height") throw new TypeError("transition.properties contains an unsupported property");
				if (seen.has(property)) throw new TypeError("transition.properties contains duplicates");
				seen.add(property);
			}
		}
	}
	if (style.boxShadow !== void 0) validateBoxShadowInput(style.boxShadow);
	if (style.fontFamily !== void 0) validateFontFamily(style.fontFamily);
	return style;
}
function encodeColor(color) {
	if (!COLOR_PATTERN.test(color)) throw new TypeError("color must be #RRGGBB or #RRGGBBAA");
	const hex = color.slice(1);
	const alpha = hex.length === 6 ? "ff" : hex.slice(6);
	return Number.parseInt(`${hex.slice(0, 6)}${alpha}`, 16) >>> 0;
}
//#endregion
//#region packages/solid-gpui/src/protocol/validate.ts
var UTF8_ENCODER$1 = new TextEncoder();
function utf8ByteLength(value) {
	return UTF8_ENCODER$1.encode(value).byteLength;
}
var ProtocolVersionMismatchError = class extends Error {
	receivedVersion;
	expectedVersion = 6;
	constructor(receivedVersion) {
		super(`protocol version mismatch: host binary speaks protocol v${receivedVersion}; this renderer package speaks protocol v6 — update @solid-gpui/core to a v${receivedVersion} release / pin the host binary to a v6 release`);
		this.name = "ProtocolVersionMismatchError";
		this.receivedVersion = receivedVersion;
	}
};
function isU32$1(value) {
	return Number.isInteger(value) && value >= 0 && value <= 4294967295;
}
function isFiniteF32(value) {
	return Number.isFinite(value) && Number.isFinite(Math.fround(value));
}
function validModifiers(value) {
	for (let index = 0; index < value.length; index += 1) {
		const modifier = value[index];
		if (!Object.hasOwn(KEY_MODIFIER_NAMES, modifier)) return false;
		for (let previous = 0; previous < index; previous += 1) if (value[previous] === modifier) return false;
	}
	return true;
}
function validText(value, maxBytes) {
	return utf8ByteLength(value) <= maxBytes;
}
function validResourcePaths(paths) {
	return paths.length > 0 && paths.every((path) => path.length > 0 && utf8ByteLength(path) <= 16776192);
}
function validExternalPaths(paths) {
	return paths.length > 0 && paths.every((path) => path.length > 0 && utf8ByteLength(path) <= 4096 && !/[\u0000-\u001f\u007f]/.test(path));
}
function validateTextInputData(value) {
	return validText(value.text, 1048576) && isU32$1(value.selectionStart) && isU32$1(value.selectionEnd) && value.selectionStart <= value.selectionEnd && isU32$1(value.editSeq) && (value.markedStart === null && value.markedEnd === null || value.markedStart !== null && value.markedEnd !== null && isU32$1(value.markedStart) && isU32$1(value.markedEnd) && value.markedStart <= value.markedEnd);
}
function validateImage(image) {
	return (image.format === "png" || image.format === "jpeg" || image.format === "gif" || image.format === "svg") && image.bytes.byteLength > 0 && image.bytes.byteLength <= 16776192;
}
function validateCommandValue(value) {
	switch (value.type) {
		case "bytes": return value.value instanceof Uint8Array && value.value.byteLength <= 1048576;
		case "number": return isU32$1(value.value);
		case "pair": return isFiniteF32(value.width) && isFiniteF32(value.height) && value.width >= 0 && value.height >= 0;
		case "boolean": return typeof value.value === "boolean";
		case "text": return validText(value.value, MAX_CLIPBOARD_TEXT_BYTES);
		case "paths": return validResourcePaths(value.paths);
		case "file-text": return validText(value.value, MAX_FILE_READ_BYTES);
		case "image": return validateImage(value.image);
		case "bounds": return [
			value.x,
			value.y,
			value.width,
			value.height
		].every(isFiniteF32) && value.width >= 0 && value.height >= 0;
		case "window-state": return typeof value.fullscreen === "boolean" && typeof value.maximized === "boolean";
		case "scroll-offset": return isFiniteF32(value.value) && value.value >= 0;
	}
}
function validateCommandResult(value) {
	return isU32$1(value.requestId) && isU32$1(value.command) && COMMAND_KINDS.includes(value.command) && isU32$1(value.nodeId) && typeof value.success === "boolean" && (value.value?.type !== "bytes" || value.command === COMMAND_INVOKE_NATIVE && value.success) && (value.command !== COMMAND_INVOKE_NATIVE || value.nodeId !== 0 && (value.success ? value.value?.type === "bytes" && value.error === null : value.value === null)) && (value.error === null || typeof value.error === "string" && validText(value.error, 1048576)) && (value.value === null || validateCommandValue(value.value));
}
function validateEventPayload(eventType, payload) {
	switch (payload.type) {
		case "application-activation": return eventType === EVENT_APPLICATION_ACTIVATION && isU32$1(payload.targetSurfaceId) && payload.targetSurfaceId > 0 && [
			"launch",
			"reopen",
			"open-urls"
		].includes(payload.reason) && payload.urls.length <= 64 && payload.urls.every((url) => typeof url === "string" && url.length > 0 && utf8ByteLength(url) <= 4096 && !/[\u0000-\u001f\u007f]/.test(url)) && (payload.reason === "open-urls" ? payload.urls.length > 0 : payload.urls.length === 0);
		case "press": return eventType === EVENT_PRESS;
		case "change": return eventType === EVENT_CHANGE && validateTextInputData(payload.data);
		case "selection": return eventType === EVENT_SELECTION && validateTextInputData(payload.data);
		case "focus": return eventType === EVENT_FOCUS && (payload.data === void 0 || validateTextInputData(payload.data));
		case "blur": return eventType === EVENT_BLUR && (payload.data === void 0 || validateTextInputData(payload.data));
		case "command-result": return eventType === EVENT_COMMAND_RESULT && validateCommandResult(payload.result);
		case "visible-range": return eventType === EVENT_VISIBLE_RANGE && isU32$1(payload.start) && isU32$1(payload.end) && payload.start <= payload.end;
		case "animation-complete": return eventType === 8 && isU32$1(payload.generation);
		case "key": return eventType === EVENT_KEY && payload.key.length > 0 && validText(payload.key, 1048576) && validModifiers(payload.modifiers) && [
			1,
			2,
			3
		].includes(payload.action);
		case "pointer": return eventType === EVENT_POINTER && isU32$1(payload.button) && payload.button >= 1 && payload.button <= 5 && validModifiers(payload.modifiers) && [1, 2].includes(payload.action) && isU32$1(payload.clickCount) && payload.clickCount > 0 && isFiniteF32(payload.x) && payload.x >= 0 && isFiniteF32(payload.y) && payload.y >= 0;
		case "pointer-move": return eventType === EVENT_POINTER && isFiniteF32(payload.x) && payload.x >= 0 && isFiniteF32(payload.y) && payload.y >= 0 && validModifiers(payload.modifiers);
		case "hover": return eventType === EVENT_HOVER;
		case "scroll": return eventType === EVENT_SCROLL && [1, 2].includes(payload.deltaKind) && [
			payload.dx,
			payload.dy,
			payload.x,
			payload.y
		].every(isFiniteF32) && validModifiers(payload.modifiers);
		case "submit": return eventType === EVENT_SUBMIT && validText(payload.text, 1048576);
		case "window-resize": return eventType === EVENT_WINDOW_RESIZE && isFiniteF32(payload.width) && payload.width >= 0 && isFiniteF32(payload.height) && payload.height >= 0 && isFiniteF32(payload.scaleFactor) && payload.scaleFactor > 0;
		case "window-activation": return eventType === EVENT_WINDOW_ACTIVATION && typeof payload.active === "boolean";
		case "surface-closed": return eventType === EVENT_SURFACE_CLOSED;
		case "action": return eventType === EVENT_ACTION && payload.action.length > 0 && validText(payload.action, 256);
		case "window-appearance": return eventType === EVENT_WINDOW_APPEARANCE && (payload.appearance === "light" || payload.appearance === "dark");
		case "layout": return eventType === EVENT_LAYOUT && [
			payload.x,
			payload.y,
			payload.width,
			payload.height
		].every(isFiniteF32);
		case "drag-over":
		case "drag-drop": return eventType === EVENT_DRAG && payload.dragType.length > 0 && validText(payload.dragType, 512) && !/[\u0000-\u001f\u007f]/.test(payload.dragType);
		case "external-file-drop": return eventType === EVENT_DRAG && validExternalPaths(payload.paths);
		case "notification-response": return eventType === EVENT_NOTIFICATION_RESPONSE && payload.tag.length > 0 && validText(payload.tag, 1024) && (payload.actionId === null || payload.actionId.length > 0 && validText(payload.actionId, 64));
		case "pointer-down-outside": return eventType === EVENT_POINTER_DOWN_OUTSIDE && isFiniteF32(payload.x) && isFiniteF32(payload.y);
		case "close-requested": return eventType === EVENT_CLOSE_REQUESTED && isU32$1(payload.requestId);
		case "extension": return eventType === EVENT_EXTENSION && isU32$1(payload.eventId) && payload.eventId !== 0 && validateExtensionFields(payload.fields);
	}
	return false;
}
function validateEvent(value) {
	if (value.payload.type === "application-activation" && (value.surfaceId !== 0 || value.nodeId !== 0 || value.listenerId !== 0 || value.revision !== 0 || value.epoch === 0 || value.sequence === 0)) return null;
	if (value.surfaceId < 0 || !isU32$1(value.surfaceId) || !isU32$1(value.epoch) || !isU32$1(value.revision) || !isU32$1(value.sequence) || !isU32$1(value.nodeId) || !isU32$1(value.listenerId)) return null;
	if (!validateEventPayload((() => {
		switch (value.payload.type) {
			case "application-activation": return EVENT_APPLICATION_ACTIVATION;
			case "press": return EVENT_PRESS;
			case "change": return EVENT_CHANGE;
			case "selection": return EVENT_SELECTION;
			case "focus": return EVENT_FOCUS;
			case "blur": return EVENT_BLUR;
			case "command-result": return EVENT_COMMAND_RESULT;
			case "visible-range": return EVENT_VISIBLE_RANGE;
			case "animation-complete": return 8;
			case "key": return EVENT_KEY;
			case "pointer":
			case "pointer-move": return EVENT_POINTER;
			case "hover": return EVENT_HOVER;
			case "scroll": return EVENT_SCROLL;
			case "submit": return EVENT_SUBMIT;
			case "window-resize": return EVENT_WINDOW_RESIZE;
			case "window-activation": return EVENT_WINDOW_ACTIVATION;
			case "surface-closed": return EVENT_SURFACE_CLOSED;
			case "action": return EVENT_ACTION;
			case "window-appearance": return EVENT_WINDOW_APPEARANCE;
			case "layout": return EVENT_LAYOUT;
			case "drag-over":
			case "drag-drop":
			case "external-file-drop": return EVENT_DRAG;
			case "notification-response": return EVENT_NOTIFICATION_RESPONSE;
			case "pointer-down-outside": return EVENT_POINTER_DOWN_OUTSIDE;
			case "close-requested": return EVENT_CLOSE_REQUESTED;
			case "extension": return EVENT_EXTENSION;
		}
	})(), value.payload)) return null;
	if (value.payload.type === "surface-closed" && (value.nodeId !== 0 || value.listenerId !== 0)) return null;
	if (value.payload.type === "command-result" && value.payload.result.command === COMMAND_INVOKE_NATIVE && (value.nodeId === 0 || value.nodeId !== value.payload.result.nodeId || value.listenerId !== 0)) return null;
	if ((value.payload.type === "focus" || value.payload.type === "blur") && value.payload.data === void 0 && (value.nodeId === 0 || value.listenerId === 0)) return null;
	if ((value.payload.type === "action" || value.payload.type === "window-appearance" || value.payload.type === "notification-response") && (value.nodeId !== 1 || value.listenerId !== 0)) return null;
	if ((value.payload.type === "drag-over" || value.payload.type === "drag-drop" || value.payload.type === "external-file-drop" || value.payload.type === "pointer-down-outside") && (value.nodeId === 0 || value.listenerId === 0)) return null;
	return value;
}
//#endregion
//#region packages/solid-gpui/src/protocol/style.ts
var ENUMS = {
	flexDirection: {
		1: "row",
		2: "column",
		3: "row-reverse",
		4: "column-reverse"
	},
	justifyContent: {
		1: "flex-start",
		2: "center",
		3: "flex-end",
		4: "space-between",
		5: "space-around",
		6: "space-evenly"
	},
	alignItems: {
		1: "flex-start",
		2: "center",
		3: "flex-end",
		4: "stretch",
		5: "baseline"
	},
	fontWeight: {
		400: "normal",
		500: "medium",
		600: "semibold",
		700: "bold",
		900: "heavy"
	},
	overflow: {
		1: "visible",
		2: "hidden",
		3: "scroll"
	},
	textOverflow: {
		1: "clip",
		2: "ellipsis"
	},
	fontStyle: {
		0: "normal",
		1: "italic"
	},
	textDecoration: {
		0: "none",
		1: "underline",
		2: "lineThrough"
	},
	alignSelf: {
		1: "start",
		2: "end",
		3: "flex-start",
		4: "flex-end",
		5: "center",
		6: "baseline",
		7: "stretch"
	},
	position: {
		0: "relative",
		1: "absolute",
		2: "overlay"
	},
	cursor: Object.fromEntries([
		"default",
		"text",
		"pointer",
		"grab",
		"grabbing",
		"not-allowed",
		"context-menu",
		"crosshair",
		"vertical-text",
		"alias",
		"copy",
		"no-drop",
		"move",
		"ew-resize",
		"ns-resize",
		"nesw-resize",
		"nwse-resize",
		"col-resize",
		"row-resize"
	].map((name, index) => [index, name])),
	textAlign: {
		1: "left",
		2: "center",
		3: "right"
	},
	flexWrap: {
		0: "nowrap",
		1: "wrap",
		2: "wrap-reverse"
	}
};
var CODES = Object.fromEntries(Object.entries(ENUMS).map(([key, values]) => [key, Object.fromEntries(Object.entries(values).map(([code, name]) => [name, Number(code)]))]));
function styleEnum(key, value) {
	return value === void 0 ? void 0 : CODES[key][value];
}
//#endregion
//#region packages/solid-gpui/src/protocol/codec-bebop.ts
var NODE_KIND_CODES = {
	View: NodeKind.View,
	Text: NodeKind.Text,
	Pressable: NodeKind.Pressable,
	RawText: NodeKind.RawText,
	TextInput: NodeKind.TextInput,
	VirtualList: NodeKind.VirtualList,
	Image: NodeKind.Image,
	Extension: NodeKind.Extension,
	Icon: NodeKind.Icon
};
var FORMAT_CODES = {
	png: 1,
	jpeg: 2,
	gif: 3,
	svg: 4
};
var FORMAT_NAMES = [
	"png",
	"jpeg",
	"gif",
	"svg"
];
var UTF8_ENCODER = new TextEncoder();
var MAX_REPEATED_ITEMS = 1 << 20;
function finite(value, name) {
	if (!Number.isFinite(value)) throw new TypeError(`${name} must be finite`);
	return value;
}
function f32(value, name) {
	if (!Number.isFinite(value)) throw new TypeError(`${name} must be finite`);
	const rounded = Math.fround(value);
	if (!Number.isFinite(rounded)) throw new RangeError(`${name} must be representable as float32`);
	return rounded;
}
function boundedString(value, maxBytes, name) {
	if (typeof value !== "string" || UTF8_ENCODER.encode(value).byteLength > maxBytes) throw new RangeError(`${name} exceeds its byte limit`);
	return value;
}
function boundedArray(value, name) {
	if (value === void 0) return void 0;
	if (value.length > MAX_REPEATED_ITEMS) throw new RangeError(`${name} exceeds the repeated-field limit`);
	return [...value];
}
function withPath(path, operation) {
	try {
		return operation();
	} catch (error) {
		const message = error instanceof Error ? error.message : String(error);
		throw new TypeError(`${path}: ${message}`, { cause: error });
	}
}
function wireExtensionValue(value) {
	if (!validateExtensionValue(value)) throw new TypeError("extension value is invalid");
	switch (value.type) {
		case "bool": return ExtensionValue.fromExtensionBoolValue({ value: value.value });
		case "i32": return ExtensionValue.fromExtensionInt32Value({ value: value.value });
		case "u32": return ExtensionValue.fromExtensionU32Value({ value: value.value });
		case "f32": return ExtensionValue.fromExtensionF32Value({ value: f32(value.value, "extension f32") });
		case "text": return ExtensionValue.fromExtensionTextValue({ value: boundedString(value.value, MAX_EXTENSION_TEXT_BYTES, "extension text") });
		case "bytes": return ExtensionValue.fromExtensionBytesValue({ value: value.value });
	}
}
function wireExtensionField(value) {
	if (!validateExtensionValue(value.value) || !Number.isInteger(value.id) || value.id < 0 || value.id > 4294967295) throw new TypeError("extension field is invalid");
	return {
		id: value.id,
		value: wireExtensionValue(value.value)
	};
}
function wireTransition(value) {
	const properties = value.properties;
	let propertyMask = 0;
	if (properties === void 0 || properties.includes("opacity")) propertyMask |= 1;
	if (properties === void 0 || properties.includes("backgroundColor")) propertyMask |= 2;
	if (properties === void 0 || properties.includes("width")) propertyMask |= 4;
	if (properties === void 0 || properties.includes("height")) propertyMask |= 8;
	const easing = value.easing === "linear" ? 0 : value.easing === "easeIn" ? 1 : value.easing === "easeOut" ? 2 : 3;
	return {
		durationMs: value.durationMs,
		delayMs: value.delayMs ?? 0,
		easing,
		propertyMask
	};
}
function wireShadow(value) {
	return { values: (Array.isArray(value) ? value : [value]).map((shadow) => ({
		offsetX: Math.fround(shadow.offsetX),
		offsetY: Math.fround(shadow.offsetY),
		blurRadius: Math.fround(shadow.blurRadius),
		spreadRadius: Math.fround(shadow.spreadRadius),
		color: encodeColor(shadow.color),
		inset: shadow.inset ?? false
	})) };
}
function wireStyle(value) {
	if (value == null) return void 0;
	validateStyle(value);
	return {
		width: value.width === void 0 ? void 0 : Math.fround(value.width),
		height: value.height === void 0 ? void 0 : Math.fround(value.height),
		gridColumns: value.gridColumns,
		gridRows: value.gridRows,
		gridColumnSpan: value.gridColumnSpan,
		gridRowSpan: value.gridRowSpan,
		flexDirection: styleEnum("flexDirection", value.flexDirection),
		flexGrow: value.flexGrow === void 0 ? void 0 : Math.fround(value.flexGrow),
		padding: value.padding === void 0 ? void 0 : Math.fround(value.padding),
		gap: value.gap === void 0 ? void 0 : Math.fround(value.gap),
		backgroundColor: value.backgroundColor === void 0 ? void 0 : encodeColor(value.backgroundColor),
		color: value.color === void 0 ? void 0 : encodeColor(value.color),
		opacity: value.opacity === void 0 ? void 0 : Math.fround(value.opacity),
		transition: value.transition === void 0 ? void 0 : wireTransition(value.transition),
		justifyContent: styleEnum("justifyContent", value.justifyContent),
		alignItems: styleEnum("alignItems", value.alignItems),
		borderRadius: value.borderRadius === void 0 ? void 0 : Math.fround(value.borderRadius),
		borderWidth: value.borderWidth === void 0 ? void 0 : Math.fround(value.borderWidth),
		borderColor: value.borderColor === void 0 ? void 0 : encodeColor(value.borderColor),
		fontSize: value.fontSize === void 0 ? void 0 : Math.fround(value.fontSize),
		fontWeight: styleEnum("fontWeight", value.fontWeight),
		overflow: styleEnum("overflow", value.overflow),
		lineClamp: value.lineClamp,
		textOverflow: styleEnum("textOverflow", value.textOverflow),
		marginTop: value.marginTop === void 0 ? void 0 : Math.fround(value.marginTop),
		marginRight: value.marginRight === void 0 ? void 0 : Math.fround(value.marginRight),
		marginBottom: value.marginBottom === void 0 ? void 0 : Math.fround(value.marginBottom),
		marginLeft: value.marginLeft === void 0 ? void 0 : Math.fround(value.marginLeft),
		fontStyle: styleEnum("fontStyle", value.fontStyle),
		textDecoration: styleEnum("textDecoration", value.textDecoration),
		lineHeight: value.lineHeight === void 0 ? void 0 : Math.fround(value.lineHeight),
		minWidth: value.minWidth === void 0 ? void 0 : Math.fround(value.minWidth),
		maxWidth: value.maxWidth === void 0 ? void 0 : Math.fround(value.maxWidth),
		minHeight: value.minHeight === void 0 ? void 0 : Math.fround(value.minHeight),
		maxHeight: value.maxHeight === void 0 ? void 0 : Math.fround(value.maxHeight),
		flexShrink: value.flexShrink === void 0 ? void 0 : Math.fround(value.flexShrink),
		alignSelf: styleEnum("alignSelf", value.alignSelf),
		position: styleEnum("position", value.position),
		left: value.left === void 0 ? void 0 : Math.fround(value.left),
		top: value.top === void 0 ? void 0 : Math.fround(value.top),
		right: value.right === void 0 ? void 0 : Math.fround(value.right),
		bottom: value.bottom === void 0 ? void 0 : Math.fround(value.bottom),
		cursor: styleEnum("cursor", value.cursor),
		textAlign: styleEnum("textAlign", value.textAlign),
		boxShadow: value.boxShadow === void 0 ? void 0 : wireShadow(value.boxShadow),
		linearGradient: value.linearGradient === void 0 ? void 0 : {
			angle: Math.fround(value.linearGradient.angle),
			startColor: encodeColor(value.linearGradient.stops[0].color),
			startPosition: Math.fround(value.linearGradient.stops[0].position),
			endColor: encodeColor(value.linearGradient.stops[1].color),
			endPosition: Math.fround(value.linearGradient.stops[1].position)
		},
		fontFamily: value.fontFamily,
		borderTopColor: value.borderTopColor === void 0 ? void 0 : encodeColor(value.borderTopColor),
		borderRightColor: value.borderRightColor === void 0 ? void 0 : encodeColor(value.borderRightColor),
		borderBottomColor: value.borderBottomColor === void 0 ? void 0 : encodeColor(value.borderBottomColor),
		borderLeftColor: value.borderLeftColor === void 0 ? void 0 : encodeColor(value.borderLeftColor),
		paddingTop: value.paddingTop === void 0 ? void 0 : Math.fround(value.paddingTop),
		paddingRight: value.paddingRight === void 0 ? void 0 : Math.fround(value.paddingRight),
		paddingBottom: value.paddingBottom === void 0 ? void 0 : Math.fround(value.paddingBottom),
		paddingLeft: value.paddingLeft === void 0 ? void 0 : Math.fround(value.paddingLeft),
		borderTopWidth: value.borderTopWidth === void 0 ? void 0 : Math.fround(value.borderTopWidth),
		borderRightWidth: value.borderRightWidth === void 0 ? void 0 : Math.fround(value.borderRightWidth),
		borderBottomWidth: value.borderBottomWidth === void 0 ? void 0 : Math.fround(value.borderBottomWidth),
		borderLeftWidth: value.borderLeftWidth === void 0 ? void 0 : Math.fround(value.borderLeftWidth),
		borderTopLeftRadius: value.borderTopLeftRadius === void 0 ? void 0 : Math.fround(value.borderTopLeftRadius),
		borderTopRightRadius: value.borderTopRightRadius === void 0 ? void 0 : Math.fround(value.borderTopRightRadius),
		borderBottomRightRadius: value.borderBottomRightRadius === void 0 ? void 0 : Math.fround(value.borderBottomRightRadius),
		borderBottomLeftRadius: value.borderBottomLeftRadius === void 0 ? void 0 : Math.fround(value.borderBottomLeftRadius),
		widthPercent: value.widthPercent === void 0 ? void 0 : Math.fround(value.widthPercent),
		heightPercent: value.heightPercent === void 0 ? void 0 : Math.fround(value.heightPercent),
		flexWrap: styleEnum("flexWrap", value.flexWrap)
	};
}
function wireAccessibility(value) {
	if (value === null) return void 0;
	return {
		role: value.role,
		label: value.label ?? void 0,
		description: value.description ?? void 0,
		disabled: value.disabled,
		checked: value.checked ?? void 0,
		selected: value.selected ?? void 0,
		value: value.value ?? void 0,
		expanded: value.expanded ?? void 0,
		level: value.level ?? void 0,
		live: value.live ?? void 0
	};
}
function wireHost(value) {
	if (value === null) return void 0;
	if (value.type === "text-input") return HostProperties.fromTextInputProperties({
		value: value.value.value,
		placeholder: value.value.placeholder ?? void 0,
		multiline: value.value.multiline,
		disabled: value.value.disabled,
		controlled: value.value.controlled,
		ackEditSeq: value.value.ackEditSeq,
		selectionStart: value.value.selectionStart,
		selectionEnd: value.value.selectionEnd,
		markedStart: value.value.markedStart ?? void 0,
		markedEnd: value.value.markedEnd ?? void 0,
		maxLength: value.value.maxLength ?? void 0,
		selectionReversed: value.value.selectionReversed
	});
	if (value.type === "virtual-list") {
		const { dataRevision, dataEdit } = value.value;
		if (!isU32(dataRevision) || dataEdit !== null && (!isU32(dataEdit.baseRevision) || dataEdit.baseRevision >= dataRevision || !isU32(dataEdit.start) || !isU32(dataEdit.oldCount) || !isU32(dataEdit.newCount) || dataEdit.start + dataEdit.newCount > value.value.itemCount)) throw new TypeError("VirtualList data edit must identify a valid data revision and replacement range");
		return HostProperties.fromVirtualListProperties({
			itemCount: value.value.itemCount,
			rangeStart: value.value.rangeStart,
			rangeEnd: value.value.rangeEnd,
			estimatedItemSize: f32(value.value.estimatedItemSize, "estimated item size"),
			overscan: value.value.overscan,
			dataRevision,
			dataEdit: dataEdit ?? void 0
		});
	}
	if (value.type === "image") return HostProperties.fromImageProperties({
		source: value.value.source,
		objectFit: value.value.objectFit,
		fallbackSource: value.value.fallbackSource ?? void 0,
		sources: value.value.sources.map((candidate) => ({
			source: candidate.source,
			width: candidate.width,
			height: candidate.height
		}))
	});
	if (value.type === "icon") return HostProperties.fromIconProperties({
		name: value.value.name,
		size: f32(value.value.size, "icon size"),
		color: value.value.color ?? void 0
	});
	if (value.type === "drag") return HostProperties.fromDragProperties({
		dragType: value.value.dragType ?? void 0,
		exportFiles: boundedArray(value.value.exportFiles ?? void 0, "exportFiles"),
		acceptsDragOver: value.value.acceptsDragOver,
		acceptsDrop: value.value.acceptsDrop
	});
	if (!validateExtensionProperties(value.value)) throw new TypeError("extension properties are invalid");
	return HostProperties.fromExtensionProperties({
		providerId: value.value.providerId,
		catalogDigest: value.value.catalogDigest,
		entryId: value.value.entryId,
		entryVersion: value.value.entryVersion,
		fields: value.value.fields.map((field) => wireExtensionField(field)),
		eventIds: [...value.value.eventIds]
	});
}
function wireNode(value) {
	return {
		id: value.id,
		parentId: value.parentId,
		index: value.index,
		kind: NODE_KIND_CODES[value.kind],
		style: withPath(`node ${value.id}.style`, () => wireStyle(value.style)),
		text: value.text ?? void 0,
		listenerId: value.listenerId,
		hostProperties: withPath(`node ${value.id}.hostProperties`, () => wireHost(value.hostProperties)),
		accessibility: withPath(`node ${value.id}.accessibility`, () => wireAccessibility(value.accessibility)),
		focusable: value.focusable,
		selectable: value.selectable,
		tooltip: value.tooltip ?? void 0,
		acceptsPointerMove: value.acceptsPointerMove,
		observesLayout: value.observesLayout
	};
}
function wireSnapshot(value) {
	return {
		surfaceId: value.surfaceId,
		epoch: value.epoch,
		baseRevision: value.baseRevision,
		revision: value.revision,
		nodes: value.nodes.map((node, index) => withPath(`body.snapshot.nodes[${index}]`, () => wireNode(node)))
	};
}
function wirePatchOperation(value) {
	if (value.type === "create") return { operation: PatchOperationValue.fromPatchCreate({ node: withPath(`patch node ${value.node.id}`, () => wireNode(value.node)) }) };
	if (value.type === "move") return { operation: PatchOperationValue.fromPatchMove({
		id: value.id,
		parentId: value.parentId,
		index: value.index
	}) };
	if (value.type === "delete") return { operation: PatchOperationValue.fromPatchDelete({ id: value.id }) };
	const clearsStyle = value.style == null;
	const style = value.mask & 1 && !clearsStyle ? withPath(`patch node ${value.id}.style`, () => wireStyle(value.style)) : void 0;
	const clearStyle = value.mask & 1 && clearsStyle ? {} : void 0;
	return { operation: PatchOperationValue.fromPatchUpdate({
		id: value.id,
		mask: value.mask,
		style,
		clearStyle,
		text: value.mask & 2 && value.text !== null ? value.text : void 0,
		listenerId: value.mask & 4 ? value.listenerId : void 0,
		hostProperties: value.mask & 8 ? withPath(`patch node ${value.id}.hostProperties`, () => wireHost(value.hostProperties)) : void 0,
		accessibility: value.mask & 16 ? withPath(`patch node ${value.id}.accessibility`, () => wireAccessibility(value.accessibility)) : void 0,
		focusable: value.mask & 32 ? value.focusable : void 0,
		selectable: value.mask & 64 ? value.selectable : void 0,
		tooltip: value.mask & 128 ? value.tooltip ?? void 0 : void 0,
		acceptsPointerMove: value.mask & 256 ? value.acceptsPointerMove : void 0,
		observesLayout: value.mask & 512 ? value.observesLayout : void 0
	}) };
}
function wirePatch(value) {
	return {
		surfaceId: value.surfaceId,
		epoch: value.epoch,
		baseRevision: value.baseRevision,
		revision: value.revision,
		operations: value.operations.map((operation, index) => withPath(`body.patch.operations[${index}]`, () => wirePatchOperation(operation)))
	};
}
function wireMenuItem(value) {
	return { value: value.type === "separator" ? MenuItemValue.fromMenuSeparator({}) : value.type === "action" ? MenuItemValue.fromMenuAction({
		name: value.name,
		disabled: value.disabled ?? false,
		checked: value.checked ?? false
	}) : MenuItemValue.fromMenuSubmenu({ menu: wireMenu({
		title: value.title,
		items: value.items
	}) }) };
}
function wireMenu(value) {
	return {
		title: value.title,
		items: value.items.map(wireMenuItem)
	};
}
function wireNotificationActions(value) {
	return boundedArray(value, "notification actions")?.map((action) => ({
		id: action.id,
		label: action.label
	}));
}
function wireKeybindings(value) {
	return value.map((binding) => ({
		keystrokes: binding.keystrokes,
		actionName: binding.actionName
	}));
}
function wireWindowOptions(value) {
	if (value === void 0) return void 0;
	return {
		kind: value.kind ?? void 0,
		resizable: value.resizable ?? void 0,
		minWidth: value.minWidth ?? void 0,
		minHeight: value.minHeight ?? void 0
	};
}
function wireImage(value) {
	return {
		format: FORMAT_CODES[value.format],
		bytes: value.bytes
	};
}
function wireCommandPayload(value) {
	if (value === null) return void 0;
	if (value.type === "open-popup") return CommandPayload.fromOpenPopupCommand({
		anchorNodeId: value.anchorNodeId,
		width: value.width,
		height: value.height,
		placement: value.placement,
		gap: value.gap
	});
	if (value.type === "close-popup") return CommandPayload.fromClosePopupCommand({ requestId: value.requestId });
	if (value.type === "configure-application") return CommandPayload.fromConfigureApplicationCommand({
		keepAlive: value.keepAlive,
		quit: value.quit,
		acknowledgedSequence: value.acknowledgedSequence
	});
	if (value.type === "cancel-native") return CommandPayload.fromCancelNativeCommand({ requestId: value.requestId });
	if (value.type === "invoke-native") return CommandPayload.fromInvokeNativeCommand({
		moduleId: value.moduleId,
		moduleDigest: value.moduleDigest,
		functionId: value.functionId,
		args: value.args
	});
	if (value.type === "selection") return CommandPayload.fromU32PairCommand({
		first: value.start,
		second: value.end
	});
	if (value.type === "scroll-index") return CommandPayload.fromU32PairCommand({
		first: value.index,
		second: value.alignment
	});
	if (value.type === "window-size") return CommandPayload.fromU32PairCommand({
		first: value.width,
		second: value.height
	});
	if (value.type === "open-surface") return CommandPayload.fromOpenSurfaceCommand({
		title: value.title,
		width: value.width,
		height: value.height,
		options: wireWindowOptions(value.options)
	});
	if (value.type === "file-dialog-open") return CommandPayload.fromFileDialogOpenCommand({
		title: value.title,
		directories: value.directories,
		multiple: value.multiple
	});
	if (value.type === "notification") return CommandPayload.fromNotificationCommand({
		title: value.title,
		body: value.body,
		actions: wireNotificationActions(value.actions)
	});
	if (value.type === "menus") return CommandPayload.fromMenusCommand({ menus: value.menus.map(wireMenu) });
	if (value.type === "keybindings") return CommandPayload.fromKeybindingsCommand({ bindings: wireKeybindings(value.bindings) });
	if (value.type === "clipboard-image") return CommandPayload.fromClipboardImageCommand(wireImage(value.image));
	if (value.type === "file-write") return CommandPayload.fromStringPairCommand({
		path: value.path,
		content: value.content
	});
	if (value.type === "close-resolution") return CommandPayload.fromCloseResolutionCommand({
		requestId: value.requestId,
		allow: value.allow
	});
	if (value.type === "text") return CommandPayload.fromTextCommand({ value: value.value });
	return CommandPayload.fromFloatCommand({ value: value.value });
}
function isU32(value) {
	return typeof value === "number" && Number.isInteger(value) && value >= 0 && value <= 4294967295;
}
function validWindowSize(width, height) {
	return isU32(width) && isU32(height) && (width === 0 && height === 0 || width > 0 && width <= 16384 && height > 0 && height <= 16384);
}
function requirePayloadType(payload, type) {
	if (payload === null || payload.type !== type) throw new TypeError(`command payload must be ${type}`);
	return payload;
}
function isRecord(value) {
	return value !== null && typeof value === "object" && !Array.isArray(value);
}
function validProtocolText(value, maxBytes, nonEmpty = false, maxCodePoints) {
	return typeof value === "string" && (!nonEmpty || value.length > 0) && (maxCodePoints === void 0 || [...value].length <= maxCodePoints) && UTF8_ENCODER.encode(value).byteLength <= maxBytes && !/[\u0000-\u001f\u007f]/.test(value);
}
function validMenuItems(items, depth) {
	if (depth > 16 || items.length > 1024) return false;
	for (const item of items) {
		if (!isRecord(item)) return false;
		if (item.type === "separator") continue;
		if (item.type === "action" && validProtocolText(item.name, 1024, true, 256) && (item.disabled === void 0 || typeof item.disabled === "boolean") && (item.checked === void 0 || typeof item.checked === "boolean")) continue;
		if (item.type === "submenu" && validProtocolText(item.title, 1024, true, 256) && Array.isArray(item.items) && validMenuItems(item.items, depth + 1)) continue;
		return false;
	}
	return true;
}
function validMenu(value, depth = 0) {
	return isRecord(value) && depth <= 16 && validProtocolText(value.title, 1024, true, 256) && Array.isArray(value.items) && validMenuItems(value.items, depth);
}
function validKeybinding(value) {
	if (!isRecord(value)) return false;
	return validProtocolText(value.keystrokes, 64, true) && validProtocolText(value.actionName, 64, true, 64);
}
function validateCommand(value) {
	if (!isU32(value.surfaceId) || !isU32(value.epoch) || !isU32(value.afterRevision) || !isU32(value.requestId) || !isU32(value.nodeId) || !COMMAND_KINDS.includes(value.command)) throw new TypeError("command header is invalid");
	if (value.command === COMMAND_INVOKE_NATIVE && value.nodeId === 0) throw new TypeError("native invocation requires a live root or component node");
	const payload = value.payload;
	switch (value.command) {
		case COMMAND_CONFIGURE_APPLICATION: {
			const control = requirePayloadType(payload, "configure-application");
			if (!isU32(control.acknowledgedSequence) || value.surfaceId !== 0 || value.nodeId !== 0 || value.afterRevision !== 0 || value.epoch === 0 || value.requestId === 0 || typeof control.keepAlive !== "boolean" || typeof control.quit !== "boolean" || control.quit && control.keepAlive) throw new TypeError("application control header or policy is invalid");
			return;
		}
		case COMMAND_CANCEL_NATIVE: {
			const target = requirePayloadType(payload, "cancel-native");
			if (value.nodeId !== 1 || !isU32(target.requestId) || target.requestId === 0) throw new TypeError("native cancellation requires a root and positive request ID");
			return;
		}
		case COMMAND_INVOKE_NATIVE: {
			const value = requirePayloadType(payload, "invoke-native");
			if (!(value.moduleId instanceof Uint8Array) || value.moduleId.byteLength !== 16 || !(value.moduleDigest instanceof Uint8Array) || value.moduleDigest.byteLength !== 32 || !isU32(value.functionId) || value.functionId === 0 || !(value.args instanceof Uint8Array) || value.args.byteLength > 1048576) throw new TypeError("native invocation payload is invalid");
			return;
		}
		case COMMAND_FOCUS:
		case COMMAND_BLUR:
		case COMMAND_SCROLL_TO_END:
		case COMMAND_ZOOM_WINDOW:
		case COMMAND_TOGGLE_FULLSCREEN:
		case COMMAND_FOCUS_NEXT:
		case COMMAND_FOCUS_PREV:
		case COMMAND_GET_WINDOW_SIZE:
		case COMMAND_GET_FOCUS:
		case COMMAND_CLIPBOARD_READ:
		case COMMAND_CLIPBOARD_READ_IMAGE:
		case COMMAND_MINIMIZE_WINDOW:
		case COMMAND_GET_WINDOW_BOUNDS:
		case COMMAND_GET_WINDOW_STATE:
		case COMMAND_ACTIVATE_WINDOW:
		case COMMAND_GET_SCROLL_OFFSET:
			if (payload !== null) throw new TypeError("command does not accept a payload");
			return;
		case COMMAND_SET_SELECTION: {
			const value = requirePayloadType(payload, "selection");
			if (!isU32(value.start) || !isU32(value.end) || value.start > value.end) throw new TypeError("selection payload is invalid");
			return;
		}
		case COMMAND_SCROLL_TO_INDEX: {
			const value = requirePayloadType(payload, "scroll-index");
			if (!isU32(value.index) || !isU32(value.alignment)) throw new TypeError("scroll index payload is invalid");
			return;
		}
		case COMMAND_RESIZE_WINDOW: {
			const value = requirePayloadType(payload, "window-size");
			if (!validWindowSize(value.width, value.height)) throw new TypeError("window size payload is invalid");
			return;
		}
		case COMMAND_SCROLL_TO_OFFSET: {
			const value = requirePayloadType(payload, "number");
			if (!Number.isFinite(value.value) || value.value < 0 || !Number.isFinite(Math.fround(value.value))) throw new TypeError("scroll offset payload is invalid");
			return;
		}
		case COMMAND_OPEN_POPUP: {
			if (value.nodeId !== 1) throw new TypeError("popup creation requires the Surface root");
			const options = requirePayloadType(payload, "open-popup");
			if (!isU32(options.anchorNodeId) || options.anchorNodeId < 2 || !validWindowSize(options.width, options.height) || options.width === 0 || options.height === 0 || !isU32(options.placement) || options.placement > 11 || !Number.isFinite(options.gap) || options.gap < 0 || options.gap > 1024) throw new TypeError("popup options are invalid");
			return;
		}
		case COMMAND_CLOSE_POPUP: {
			if (value.nodeId !== 1) throw new TypeError("popup cancellation requires the Surface root");
			const options = requirePayloadType(payload, "close-popup");
			if (!isU32(options.requestId) || options.requestId === 0) throw new TypeError("popup request is invalid");
			return;
		}
		case COMMAND_OPEN_SURFACE: {
			const value = requirePayloadType(payload, "open-surface");
			if (typeof value.title !== "string" || [...value.title].length > 256 || !validWindowSize(value.width, value.height) || value.options !== void 0 && (value.options.kind !== null && ![
				0,
				1,
				2
			].includes(value.options.kind) || value.options.resizable !== null && typeof value.options.resizable !== "boolean" || value.options.minWidth === null !== (value.options.minHeight === null) || value.options.minWidth !== null && (!isU32(value.options.minWidth) || !isU32(value.options.minHeight) || value.options.minWidth === 0 || value.options.minHeight === 0 || value.options.minWidth > 16384 || value.options.minHeight > 16384))) throw new TypeError("open surface payload is invalid");
			return;
		}
		case COMMAND_FILE_DIALOG_OPEN: {
			const value = requirePayloadType(payload, "file-dialog-open");
			if (typeof value.title !== "string" || [...value.title].length > 256 || typeof value.directories !== "boolean" || typeof value.multiple !== "boolean") throw new TypeError("file dialog payload is invalid");
			return;
		}
		case COMMAND_SHOW_NOTIFICATION: {
			const value = requirePayloadType(payload, "notification");
			if (typeof value.title !== "string" || UTF8_ENCODER.encode(value.title).byteLength > 256 || typeof value.body !== "string" || UTF8_ENCODER.encode(value.body).byteLength > 1024 || value.actions !== void 0 && (!Array.isArray(value.actions) || value.actions.length > 3 || value.actions.some((action) => !isRecord(action) || !validProtocolText(action.id, 64, true) || !validProtocolText(action.label, 256, true, 256)))) throw new TypeError("notification payload is invalid");
			return;
		}
		case COMMAND_SET_MENUS: {
			const value = requirePayloadType(payload, "menus");
			if (!Array.isArray(value.menus) || value.menus.length > 64 || value.menus.some((menu) => !validMenu(menu))) throw new TypeError("menus payload is invalid");
			return;
		}
		case COMMAND_SET_KEYBINDINGS: {
			const value = requirePayloadType(payload, "keybindings");
			if (!Array.isArray(value.bindings) || value.bindings.length > 64 || value.bindings.some((binding) => !validKeybinding(binding))) throw new TypeError("keybindings payload is invalid");
			return;
		}
		case COMMAND_CLIPBOARD_WRITE_IMAGE: {
			const value = requirePayloadType(payload, "clipboard-image");
			if (!isRecord(value.image) || ![
				"png",
				"jpeg",
				"gif",
				"svg"
			].includes(value.image.format) || !(value.image.bytes instanceof Uint8Array) || value.image.bytes.byteLength === 0 || value.image.bytes.byteLength > 16776192) throw new TypeError("clipboard image payload is invalid");
			return;
		}
		case COMMAND_WRITE_TEXT_FILE: {
			const value = requirePayloadType(payload, "file-write");
			if (typeof value.path !== "string" || value.path.length === 0 || UTF8_ENCODER.encode(value.path).byteLength > 1024 || /[\u0000-\u001f\u007f]/.test(value.path) || !value.path.startsWith("/") || typeof value.content !== "string" || UTF8_ENCODER.encode(value.content).byteLength > 16776192) throw new TypeError("file write payload is invalid");
			return;
		}
		case COMMAND_RESOLVE_CLOSE_REQUEST: {
			const value = requirePayloadType(payload, "close-resolution");
			if (!isU32(value.requestId) || typeof value.allow !== "boolean") throw new TypeError("close resolution payload is invalid");
			return;
		}
		case COMMAND_SET_TITLE: {
			const value = requirePayloadType(payload, "text");
			if (typeof value.value !== "string" || [...value.value].length > 256 || value.value.length === 0) throw new TypeError("title command payload is invalid");
			return;
		}
		case COMMAND_FILE_DIALOG_SAVE: {
			const value = requirePayloadType(payload, "text");
			if (typeof value.value !== "string" || [...value.value].length > 256) throw new TypeError("file dialog save payload is invalid");
			return;
		}
		case COMMAND_OPEN_URL: {
			const value = requirePayloadType(payload, "text");
			if (typeof value.value !== "string" || value.value.length === 0 || UTF8_ENCODER.encode(value.value).byteLength > 2048 || /\s/.test(value.value) || !value.value.startsWith("http://") && !value.value.startsWith("https://") || value.value.slice(value.value.indexOf("://") + 3).length === 0) throw new TypeError("URL command payload is invalid");
			return;
		}
		case COMMAND_CLIPBOARD_WRITE: {
			const value = requirePayloadType(payload, "text");
			if (typeof value.value !== "string" || UTF8_ENCODER.encode(value.value).byteLength > 1048576) throw new TypeError("clipboard text payload is invalid");
			return;
		}
		case COMMAND_SET_CLOSE_POLICY: {
			const value = requirePayloadType(payload, "text");
			if (value.value !== "allow" && value.value !== "require-confirmation") throw new TypeError("close policy payload is invalid");
			return;
		}
		case COMMAND_READ_TEXT_FILE:
		case COMMAND_LOAD_FONT: {
			const value = requirePayloadType(payload, "text");
			if (typeof value.value !== "string" || value.value.length === 0 || UTF8_ENCODER.encode(value.value).byteLength > 1024 || /[\u0000-\u001f\u007f]/.test(value.value) || !value.value.startsWith("/")) throw new TypeError("file path command payload is invalid");
			return;
		}
		default: throw new TypeError("unknown command");
	}
}
function wireCommand(value) {
	return withPath(`body.command(kind=${value.command})`, () => {
		validateCommand(value);
		return {
			surfaceId: value.surfaceId,
			epoch: value.epoch,
			afterRevision: value.afterRevision,
			requestId: value.requestId,
			nodeId: value.nodeId,
			kind: value.command,
			payload: wireCommandPayload(value.payload)
		};
	});
}
function wireCommandValue(value) {
	if (value.type === "bytes") {
		if (!(value.value instanceof Uint8Array) || value.value.byteLength > 1048576) throw new TypeError("native byte result is invalid");
		return CommandValue.fromBytesValue({ value: value.value });
	}
	if (value.type === "number") {
		if (!isU32(value.value)) throw new TypeError("command number must be a u32");
		return CommandValue.fromNumberValue({ value: value.value });
	}
	if (value.type === "pair") return CommandValue.fromPairValue({
		width: f32(value.width, "command pair width"),
		height: f32(value.height, "command pair height")
	});
	if (value.type === "boolean") return CommandValue.fromBoolValue({ value: value.value });
	if (value.type === "text") return CommandValue.fromTextValue({ value: value.value });
	if (value.type === "paths") return CommandValue.fromPathsValue({ paths: [...value.paths] });
	if (value.type === "file-text") return CommandValue.fromFileTextValue({ value: value.value });
	if (value.type === "image") return CommandValue.fromImageValue(wireImage(value.image));
	if (value.type === "bounds") return CommandValue.fromBoundsValue({
		x: f32(value.x, "bounds x"),
		y: f32(value.y, "bounds y"),
		width: f32(value.width, "bounds width"),
		height: f32(value.height, "bounds height")
	});
	if (value.type === "window-state") return CommandValue.fromWindowStateValue({
		fullscreen: value.fullscreen,
		maximized: value.maximized
	});
	if (value.type === "scroll-offset") return CommandValue.fromScrollOffsetValue({ value: f32(value.value, "scroll offset") });
	throw new TypeError("unknown command value");
}
function wireTextInput(value) {
	return {
		text: value.text,
		selectionStart: value.selectionStart,
		selectionEnd: value.selectionEnd,
		markedStart: value.markedStart ?? void 0,
		markedEnd: value.markedEnd ?? void 0,
		editSeq: value.editSeq,
		reversed: value.reversed
	};
}
function eventType(value) {
	switch (value.type) {
		case "application-activation": return EVENT_APPLICATION_ACTIVATION;
		case "press": return EVENT_PRESS;
		case "change": return EVENT_CHANGE;
		case "selection": return EVENT_SELECTION;
		case "focus": return EVENT_FOCUS;
		case "blur": return EVENT_BLUR;
		case "command-result": return EVENT_COMMAND_RESULT;
		case "visible-range": return EVENT_VISIBLE_RANGE;
		case "animation-complete": return EVENT_ANIMATION_COMPLETE;
		case "key": return EVENT_KEY;
		case "pointer":
		case "pointer-move": return EVENT_POINTER;
		case "hover": return EVENT_HOVER;
		case "scroll": return EVENT_SCROLL;
		case "submit": return EVENT_SUBMIT;
		case "window-resize": return EVENT_WINDOW_RESIZE;
		case "window-activation": return EVENT_WINDOW_ACTIVATION;
		case "surface-closed": return EVENT_SURFACE_CLOSED;
		case "action": return EVENT_ACTION;
		case "window-appearance": return EVENT_WINDOW_APPEARANCE;
		case "layout": return EVENT_LAYOUT;
		case "drag-over":
		case "drag-drop":
		case "external-file-drop": return EVENT_DRAG;
		case "notification-response": return EVENT_NOTIFICATION_RESPONSE;
		case "pointer-down-outside": return EVENT_POINTER_DOWN_OUTSIDE;
		case "close-requested": return EVENT_CLOSE_REQUESTED;
	}
	return EVENT_EXTENSION;
}
function wireEventPayload(value) {
	switch (value.type) {
		case "application-activation": return EventPayload.fromApplicationActivationEvent({
			targetSurfaceId: value.targetSurfaceId,
			reason: value.reason,
			urls: [...value.urls]
		});
		case "press":
		case "hover":
		case "surface-closed": return;
		case "change":
		case "selection":
		case "focus":
		case "blur": return value.data === void 0 ? void 0 : EventPayload.fromTextInputEventData(wireTextInput(value.data));
		case "command-result": {
			const result = value.result;
			return EventPayload.fromCommandResult({
				requestId: result.requestId,
				command: result.command,
				nodeId: result.nodeId,
				success: result.success,
				error: result.error ?? void 0,
				value: result.value === null ? void 0 : wireCommandValue(result.value)
			});
		}
		case "visible-range": return EventPayload.fromVisibleRangeEvent({
			start: value.start,
			end: value.end
		});
		case "animation-complete": return EventPayload.fromAnimationCompleteEvent({ generation: value.generation });
		case "key": return EventPayload.fromKeyEvent({
			key: value.key,
			modifiers: [...value.modifiers],
			action: value.action
		});
		case "pointer": return EventPayload.fromPointerEvent({
			button: value.button,
			modifiers: [...value.modifiers],
			action: value.action,
			clickCount: value.clickCount,
			x: f32(value.x, "pointer x"),
			y: f32(value.y, "pointer y")
		});
		case "pointer-move": return EventPayload.fromPointerMoveEvent({
			modifiers: [...value.modifiers],
			x: f32(value.x, "pointer move x"),
			y: f32(value.y, "pointer move y")
		});
		case "scroll": return EventPayload.fromScrollEvent({
			deltaKind: value.deltaKind,
			dx: f32(value.dx, "scroll dx"),
			dy: f32(value.dy, "scroll dy"),
			x: f32(value.x, "scroll x"),
			y: f32(value.y, "scroll y"),
			modifiers: [...value.modifiers]
		});
		case "submit": return EventPayload.fromSubmitEvent({ text: value.text });
		case "window-resize": return EventPayload.fromWindowResizeEvent({
			width: f32(value.width, "window width"),
			height: f32(value.height, "window height"),
			scaleFactor: f32(value.scaleFactor, "window scale factor")
		});
		case "window-activation": return EventPayload.fromWindowActivationEvent({ active: value.active });
		case "action": return EventPayload.fromActionEvent({ action: value.action });
		case "window-appearance": return EventPayload.fromWindowAppearanceEvent({ appearance: value.appearance === "light" ? WindowAppearance.Light : WindowAppearance.Dark });
		case "layout": return EventPayload.fromLayoutEvent({
			x: f32(value.x, "layout x"),
			y: f32(value.y, "layout y"),
			width: f32(value.width, "layout width"),
			height: f32(value.height, "layout height")
		});
		case "drag-over": return EventPayload.fromDragOverEvent({ dragType: value.dragType });
		case "drag-drop": return EventPayload.fromDragDropEvent({ dragType: value.dragType });
		case "external-file-drop": return EventPayload.fromExternalFileDropEvent({ paths: [...value.paths] });
		case "notification-response": return EventPayload.fromNotificationResponseEvent({
			tag: value.tag,
			actionId: value.actionId ?? void 0
		});
		case "pointer-down-outside": return EventPayload.fromPointerDownOutsideEvent({
			x: f32(value.x, "pointer-down-outside x"),
			y: f32(value.y, "pointer-down-outside y")
		});
		case "close-requested": return EventPayload.fromCloseRequestedEvent({ requestId: value.requestId });
		case "extension": return EventPayload.fromExtensionEvent({
			eventId: value.eventId,
			fields: value.fields.map((field) => wireExtensionField(field))
		});
	}
}
function wireEvent(value) {
	return withPath(`body.event(sequence=${value.sequence})`, () => {
		if (validateEvent(value) === null) throw new TypeError("event is invalid");
		return {
			surfaceId: value.surfaceId,
			epoch: value.epoch,
			revision: value.revision,
			sequence: value.sequence,
			nodeId: value.nodeId,
			listenerId: value.listenerId,
			eventType: eventType(value.payload),
			payload: wireEventPayload(value.payload)
		};
	});
}
function wireEnvelope(value) {
	return {
		protocolVersion: 6,
		body: value.type === "snapshot" ? Body.fromSnapshot(wireSnapshot(value)) : value.type === "patch" ? Body.fromPatch(wirePatch(value)) : value.type === "command" ? Body.fromCommand(wireCommand(value)) : Body.fromEvent(wireEvent(value))
	};
}
function encodeInto(value, view) {
	view.startWriting();
	Envelope.encodeInto(wireEnvelope(value), view);
	return view.toArray();
}
function encodeFrame(value) {
	const payload = encodeInto(value, BebopView.getInstance());
	if (payload.byteLength > 16777216) throw new RangeError("Bebop frame exceeds maximum size");
	const frame = new Uint8Array(payload.byteLength + 4);
	new DataView(frame.buffer).setUint32(0, payload.byteLength, true);
	frame.set(payload, 4);
	return frame;
}
function decodeEnvelope(payload) {
	if (payload.byteLength > 16777216) throw new RangeError("Bebop payload exceeds maximum size");
	boundedBebopDecode(payload);
	const value = Envelope.decode(payload);
	if (value.protocolVersion !== 6) throw new ProtocolVersionMismatchError(value.protocolVersion ?? 0);
	if (value.body === void 0) throw new TypeError("Bebop envelope body is missing");
	return value;
}
function decodeImage(value) {
	if (value.format === void 0 || value.bytes === void 0 || !Number.isInteger(value.format) || value.format < 1 || value.format > 4 || value.bytes.byteLength === 0 || value.bytes.byteLength > 16776192) return null;
	return {
		format: FORMAT_NAMES[value.format - 1],
		bytes: value.bytes.slice()
	};
}
function semanticCommandValue(value) {
	switch (value.tag) {
		case 1: return value.value.value === void 0 || !isU32(value.value.value) ? null : {
			type: "number",
			value: value.value.value
		};
		case 2: return value.value.width === void 0 || value.value.height === void 0 ? null : {
			type: "pair",
			width: finite(value.value.width, "command pair width"),
			height: finite(value.value.height, "command pair height")
		};
		case 3: return value.value.value === void 0 ? null : {
			type: "boolean",
			value: value.value.value
		};
		case 4: return value.value.value === void 0 ? null : {
			type: "text",
			value: boundedString(value.value.value, MAX_CLIPBOARD_TEXT_BYTES, "command text")
		};
		case 5: return value.value.paths === void 0 || value.value.paths.length === 0 ? null : {
			type: "paths",
			paths: value.value.paths.map((path) => boundedString(path, MAX_FILE_READ_BYTES, "command path"))
		};
		case 6: return value.value.value === void 0 ? null : {
			type: "file-text",
			value: boundedString(value.value.value, MAX_FILE_READ_BYTES, "file text")
		};
		case 7: {
			const image = decodeImage(value.value);
			return image === null ? null : {
				type: "image",
				image
			};
		}
		case 8: return value.value.x === void 0 || value.value.y === void 0 || value.value.width === void 0 || value.value.height === void 0 ? null : {
			type: "bounds",
			x: finite(value.value.x, "bounds x"),
			y: finite(value.value.y, "bounds y"),
			width: finite(value.value.width, "bounds width"),
			height: finite(value.value.height, "bounds height")
		};
		case 9: return value.value.fullscreen === void 0 || value.value.maximized === void 0 ? null : {
			type: "window-state",
			fullscreen: value.value.fullscreen,
			maximized: value.value.maximized
		};
		case 10: return value.value.value === void 0 ? null : {
			type: "scroll-offset",
			value: finite(value.value.value, "scroll offset")
		};
		case 11: return value.value.value instanceof Uint8Array && value.value.value.byteLength <= 1048576 ? {
			type: "bytes",
			value: value.value.value
		} : null;
		default: return null;
	}
}
function semanticCommandResult(value) {
	if (value.requestId === void 0 || value.command === void 0 || value.nodeId === void 0 || value.success === void 0 || !COMMAND_KINDS.includes(value.command)) return null;
	const commandValue = value.value === void 0 ? null : semanticCommandValue(value.value);
	if (value.value !== void 0 && commandValue === null) return null;
	return {
		requestId: value.requestId,
		command: value.command,
		nodeId: value.nodeId,
		success: value.success,
		error: value.error ?? null,
		value: commandValue
	};
}
function semanticExtensionValue(value) {
	let candidate;
	switch (value.tag) {
		case 1:
			candidate = value.value.value === void 0 ? null : {
				type: "bool",
				value: value.value.value
			};
			break;
		case 2:
			candidate = value.value.value === void 0 ? null : {
				type: "i32",
				value: value.value.value
			};
			break;
		case 3:
			candidate = value.value.value === void 0 ? null : {
				type: "u32",
				value: value.value.value
			};
			break;
		case 4:
			candidate = value.value.value === void 0 ? null : {
				type: "f32",
				value: value.value.value
			};
			break;
		case 5:
			candidate = value.value.value === void 0 ? null : {
				type: "text",
				value: value.value.value
			};
			break;
		case 6:
			candidate = value.value.value === void 0 ? null : {
				type: "bytes",
				value: value.value.value
			};
			break;
		default: return null;
	}
	return candidate !== null && validateExtensionValue(candidate) ? candidate : null;
}
function semanticExtensionFields(fields) {
	if (fields === void 0 || fields.length > 256) return null;
	const result = [];
	for (const field of fields) {
		if (field.id === void 0 || field.value === void 0) return null;
		const value = semanticExtensionValue(field.value);
		if (value === null) return null;
		result.push({
			id: field.id,
			value
		});
	}
	return validateExtensionFields(result) ? result : null;
}
function semanticTextInput(value) {
	if (value.text === void 0 || value.selectionStart === void 0 || value.selectionEnd === void 0 || value.editSeq === void 0 || value.reversed === void 0 || value.selectionStart > value.selectionEnd || value.markedStart === void 0 !== (value.markedEnd === void 0) || value.markedStart !== void 0 && value.markedEnd !== void 0 && value.markedStart > value.markedEnd) return null;
	return {
		text: value.text,
		selectionStart: value.selectionStart,
		selectionEnd: value.selectionEnd,
		markedStart: value.markedStart ?? null,
		markedEnd: value.markedEnd ?? null,
		editSeq: value.editSeq,
		reversed: value.reversed
	};
}
function semanticEventPayload(eventType, value) {
	if (eventType === EVENT_PRESS) return value === void 0 ? { type: "press" } : null;
	if (eventType === EVENT_HOVER) return value === void 0 ? { type: "hover" } : null;
	if (eventType === EVENT_SURFACE_CLOSED) return value === void 0 ? { type: "surface-closed" } : null;
	if (eventType === EVENT_FOCUS || eventType === EVENT_BLUR) {
		if (value === void 0) return eventType === EVENT_FOCUS ? { type: "focus" } : { type: "blur" };
		if (value.tag !== 1) return null;
		const data = semanticTextInput(value.value);
		return data === null ? null : eventType === EVENT_FOCUS ? {
			type: "focus",
			data
		} : {
			type: "blur",
			data
		};
	}
	if (value === void 0) return null;
	switch (eventType) {
		case EVENT_APPLICATION_ACTIVATION:
			if (value.tag !== 22 || value.value.targetSurfaceId === void 0 || value.value.reason !== "launch" && value.value.reason !== "reopen" && value.value.reason !== "open-urls" || value.value.urls === void 0) return null;
			return {
				type: "application-activation",
				targetSurfaceId: value.value.targetSurfaceId,
				reason: value.value.reason,
				urls: value.value.urls
			};
		case EVENT_CHANGE:
		case EVENT_SELECTION: {
			if (value.tag !== 1) return null;
			const data = semanticTextInput(value.value);
			return data === null ? null : eventType === EVENT_CHANGE ? {
				type: "change",
				data
			} : {
				type: "selection",
				data
			};
		}
		case EVENT_COMMAND_RESULT: {
			if (value.tag !== 2) return null;
			const result = semanticCommandResult(value.value);
			return result === null ? null : {
				type: "command-result",
				result
			};
		}
		case EVENT_VISIBLE_RANGE: return value.tag === 3 && value.value.start !== void 0 && value.value.end !== void 0 && value.value.start <= value.value.end ? {
			type: "visible-range",
			start: value.value.start,
			end: value.value.end
		} : null;
		case EVENT_ANIMATION_COMPLETE: return value.tag === 4 && value.value.generation !== void 0 ? {
			type: "animation-complete",
			generation: value.value.generation
		} : null;
		case EVENT_KEY: return value.tag === 5 && value.value.key !== void 0 && value.value.key.length > 0 && value.value.modifiers !== void 0 && value.value.action !== void 0 && [
			1,
			2,
			3
		].includes(value.value.action) ? {
			type: "key",
			key: value.value.key,
			modifiers: value.value.modifiers,
			action: value.value.action
		} : null;
		case EVENT_POINTER:
			if (value.tag === 6) return value.value.button !== void 0 && value.value.modifiers !== void 0 && value.value.action !== void 0 && value.value.clickCount !== void 0 && value.value.clickCount > 0 && value.value.x !== void 0 && value.value.y !== void 0 ? {
				type: "pointer",
				button: value.value.button,
				modifiers: value.value.modifiers,
				action: value.value.action,
				clickCount: value.value.clickCount,
				x: value.value.x,
				y: value.value.y
			} : null;
			return value.tag === 7 && value.value.modifiers !== void 0 && value.value.x !== void 0 && value.value.y !== void 0 ? {
				type: "pointer-move",
				x: value.value.x,
				y: value.value.y,
				modifiers: value.value.modifiers
			} : null;
		case EVENT_SCROLL: return value.tag === 8 && value.value.deltaKind !== void 0 && [1, 2].includes(value.value.deltaKind) && value.value.dx !== void 0 && value.value.dy !== void 0 && value.value.x !== void 0 && value.value.y !== void 0 && value.value.modifiers !== void 0 ? {
			type: "scroll",
			deltaKind: value.value.deltaKind,
			dx: value.value.dx,
			dy: value.value.dy,
			x: value.value.x,
			y: value.value.y,
			modifiers: value.value.modifiers
		} : null;
		case EVENT_SUBMIT: return value.tag === 9 && value.value.text !== void 0 ? {
			type: "submit",
			text: value.value.text
		} : null;
		case EVENT_WINDOW_RESIZE: return value.tag === 10 && value.value.width !== void 0 && value.value.height !== void 0 && value.value.scaleFactor !== void 0 && value.value.width >= 0 && value.value.height >= 0 && value.value.scaleFactor > 0 ? {
			type: "window-resize",
			width: value.value.width,
			height: value.value.height,
			scaleFactor: value.value.scaleFactor
		} : null;
		case EVENT_WINDOW_ACTIVATION: return value.tag === 11 && value.value.active !== void 0 ? {
			type: "window-activation",
			active: value.value.active
		} : null;
		case EVENT_ACTION: return value.tag === 12 && value.value.action !== void 0 && value.value.action.length > 0 ? {
			type: "action",
			action: value.value.action
		} : null;
		case EVENT_WINDOW_APPEARANCE: return value.tag === 13 && value.value.appearance !== void 0 && (value.value.appearance === WindowAppearance.Light || value.value.appearance === WindowAppearance.Dark) ? {
			type: "window-appearance",
			appearance: value.value.appearance === WindowAppearance.Light ? "light" : "dark"
		} : null;
		case EVENT_LAYOUT: return value.tag === 14 && value.value.x !== void 0 && value.value.y !== void 0 && value.value.width !== void 0 && value.value.height !== void 0 ? {
			type: "layout",
			x: value.value.x,
			y: value.value.y,
			width: value.value.width,
			height: value.value.height
		} : null;
		case EVENT_DRAG:
			if (value.tag === 15 && value.value.dragType !== void 0) return {
				type: "drag-over",
				dragType: value.value.dragType
			};
			if (value.tag === 16 && value.value.dragType !== void 0) return {
				type: "drag-drop",
				dragType: value.value.dragType
			};
			return value.tag === 17 && value.value.paths !== void 0 && value.value.paths.length > 0 ? {
				type: "external-file-drop",
				paths: value.value.paths
			} : null;
		case EVENT_EXTENSION:
			if (value.tag !== 21 || value.value.eventId === void 0) return null;
			{
				const fields = semanticExtensionFields(value.value.fields);
				return fields === null ? null : {
					type: "extension",
					eventId: value.value.eventId,
					fields
				};
			}
		case EVENT_NOTIFICATION_RESPONSE: return value.tag === 18 && value.value.tag !== void 0 ? {
			type: "notification-response",
			tag: value.value.tag,
			actionId: value.value.actionId ?? null
		} : null;
		case EVENT_POINTER_DOWN_OUTSIDE: return value.tag === 19 && value.value.x !== void 0 && value.value.y !== void 0 ? {
			type: "pointer-down-outside",
			x: value.value.x,
			y: value.value.y
		} : null;
		case EVENT_CLOSE_REQUESTED: return value.tag === 20 && value.value.requestId !== void 0 ? {
			type: "close-requested",
			requestId: value.value.requestId
		} : null;
		default: return null;
	}
}
function semanticEvent(value) {
	if (value.surfaceId === void 0 || value.epoch === void 0 || value.revision === void 0 || value.sequence === void 0 || value.nodeId === void 0 || value.listenerId === void 0 || value.eventType === void 0) return null;
	const payload = semanticEventPayload(value.eventType, value.payload);
	if (payload === null) return null;
	return validateEvent({
		type: "event",
		surfaceId: value.surfaceId,
		epoch: value.epoch,
		revision: value.revision,
		sequence: value.sequence,
		nodeId: value.nodeId,
		listenerId: value.listenerId,
		payload
	});
}
function decodeEvent(payload) {
	const envelope = decodeEnvelope(payload);
	if (envelope.body?.tag !== 2) return null;
	return semanticEvent(envelope.body.value);
}
function classifyPayload(payload) {
	try {
		const body = decodeEnvelope(payload).body;
		if (body === void 0) return { kind: "unknown" };
		if (body.tag === 1) return { kind: "snapshot" };
		if (body.tag === 3) return { kind: "patch" };
		if (body.tag === 2) {
			const event = body.value;
			if (event.eventType === void 0) return { kind: "unknown" };
			const result = event.payload?.tag === 2 ? event.payload.value : void 0;
			return {
				kind: "event",
				event_type: event.eventType,
				...result?.requestId === void 0 ? {} : { request_id: result.requestId },
				...result?.success === void 0 ? {} : { success: result.success }
			};
		}
		if (body.tag === 4) return {
			kind: "command",
			command_kind: body.value.kind,
			request_id: body.value.requestId
		};
		return { kind: "unknown" };
	} catch {
		return { kind: "unknown" };
	}
}
//#endregion
//#region packages/solid-gpui/src/protocol-tap.ts
var PROTOCOL_TAP_CAPACITY_BYTES = 67108864;
var TAP_STOP_RESERVE_BYTES = 256;
/** Process-local JSONL observer for framed protocol traffic. */
var ProtocolTap = class ProtocolTap {
	capacityBytes;
	file;
	started = performance.now();
	bytesWritten = 0;
	sequence = 1;
	stopped = false;
	closed = false;
	inboundBuffer = /* @__PURE__ */ new Uint8Array(0);
	constructor(capacityBytes, path) {
		this.capacityBytes = capacityBytes;
		try {
			this.file = openSync(path, "w", 384);
		} catch (error) {
			console.error(`solid-gpui: protocol tap disabled; cannot open ${path}: ${String(error)}`);
			throw error;
		}
	}
	static fromEnv() {
		const path = process.env.SOLID_GPUI_TAP;
		if (path === void 0 || path === "") return void 0;
		try {
			return new ProtocolTap(PROTOCOL_TAP_CAPACITY_BYTES, path);
		} catch {
			return;
		}
	}
	static openForTest(path, capacityBytes) {
		if (!Number.isSafeInteger(capacityBytes) || capacityBytes < 1) throw new RangeError("protocol tap capacity must be a positive integer");
		return new ProtocolTap(capacityBytes, path);
	}
	get enabled() {
		return !this.stopped;
	}
	recordOutboundFrame(frame) {
		if (this.stopped) return;
		this.record("out", "host", frame.subarray(4), frame.byteLength);
	}
	observeInbound(chunk) {
		if (this.stopped || chunk.byteLength === 0) return;
		const combined = new Uint8Array(this.inboundBuffer.byteLength + chunk.byteLength);
		combined.set(this.inboundBuffer);
		combined.set(chunk, this.inboundBuffer.byteLength);
		this.inboundBuffer = combined;
		while (this.inboundBuffer.byteLength >= 4 && !this.stopped) {
			const size = new DataView(this.inboundBuffer.buffer, this.inboundBuffer.byteOffset, 4).getUint32(0, true);
			if (size > 16777216) {
				this.record("in", "host", /* @__PURE__ */ new Uint8Array(0), 4, { kind: "unknown" });
				this.inboundBuffer = /* @__PURE__ */ new Uint8Array(0);
				return;
			}
			const frameSize = size + 4;
			if (this.inboundBuffer.byteLength < frameSize) return;
			const frame = this.inboundBuffer.subarray(0, frameSize);
			this.record("in", "host", frame.subarray(4), frame.byteLength);
			this.inboundBuffer = frameSize === this.inboundBuffer.byteLength ? /* @__PURE__ */ new Uint8Array(0) : this.inboundBuffer.subarray(frameSize);
		}
	}
	dispose() {
		if (this.closed) return;
		closeSync(this.file);
		this.closed = true;
		this.stopped = true;
		this.inboundBuffer = /* @__PURE__ */ new Uint8Array(0);
	}
	record(direction, peer, payload, frameBytes, override) {
		if (this.stopped) return;
		const classification = override ?? classify(payload);
		const timestamp = Math.floor(performance.now() - this.started);
		const line = JSON.stringify({
			t: timestamp,
			dir: direction,
			peer,
			kind: classification.kind,
			bytes: frameBytes,
			seq: this.sequence,
			...classification.event_type === void 0 ? {} : { event_type: classification.event_type },
			...classification.command_kind === void 0 ? {} : { command_kind: classification.command_kind },
			...classification.request_id === void 0 ? {} : { request_id: classification.request_id },
			...classification.success === void 0 ? {} : { success: classification.success }
		}) + "\n";
		if (this.bytesWritten + line.length > this.capacityBytes - TAP_STOP_RESERVE_BYTES) {
			this.writeStopped(direction, peer, frameBytes, timestamp);
			return;
		}
		try {
			writeSync(this.file, line, void 0, "utf8");
			this.bytesWritten += line.length;
			this.sequence += 1;
		} catch {
			this.stopped = true;
		}
	}
	writeStopped(direction, peer, frameBytes, timestamp) {
		const line = JSON.stringify({
			t: timestamp,
			dir: direction,
			peer,
			kind: "tap_stopped",
			bytes: frameBytes,
			seq: this.sequence,
			reason: "capacity"
		}) + "\n";
		if (this.bytesWritten + line.length <= this.capacityBytes) try {
			writeSync(this.file, line, void 0, "utf8");
			this.bytesWritten += line.length;
			this.sequence += 1;
		} catch {}
		this.stopped = true;
	}
};
function classify(payload) {
	return classifyPayload(payload);
}
//#endregion
//#region packages/solid-gpui/src/transport.ts
function isTransportTerminationCause(value) {
	if (value === null || typeof value !== "object") return false;
	const candidate = value;
	if (candidate.kind === "shutdown" || candidate.kind === "eof") return true;
	if (candidate.kind === "exit") return typeof candidate.code === "number" && Number.isInteger(candidate.code);
	return (candidate.kind === "protocol" || candidate.kind === "io") && typeof candidate.detail === "string" && candidate.detail.length > 0;
}
var TransportTerminatedError = class extends Error {
	cause;
	exitCode;
	stderrTail;
	crashReportPath;
	constructor(message, cause, details = {}) {
		super(message);
		this.name = "TransportTerminatedError";
		this.cause = cause === void 0 ? void 0 : isTransportTerminationCause(cause) ? cause : {
			kind: "io",
			detail: describeError$1(cause)
		};
		this.exitCode = details.exitCode;
		this.stderrTail = details.stderrTail;
		this.crashReportPath = details.crashReportPath;
	}
};
function describeError$1(error) {
	return error instanceof Error ? error.message : String(error);
}
//#endregion
//#region packages/solid-gpui/src/stdio.ts
function asBytes(chunk) {
	return chunk instanceof Uint8Array ? chunk : new Uint8Array(chunk);
}
function describeError(error) {
	return error instanceof Error ? error.message : String(error);
}
var CRASH_REPORT_PREFIX = "solid-gpui-host: crash report: ";
function crashReportPathFromStderr(stderrTail) {
	if (stderrTail === void 0) return void 0;
	let path;
	for (const line of stderrTail.split(/\r?\n/)) if (line.startsWith(CRASH_REPORT_PREFIX) && line.length > 31) path = line.slice(31);
	return path;
}
function detailsFromCause(cause) {
	if (cause === null || typeof cause !== "object") return {};
	const value = cause;
	const stderrTail = typeof value.stderrTail === "string" ? value.stderrTail.split(/\r?\n/).slice(-50).join("\n") : void 0;
	return {
		exitCode: typeof value.exitCode === "number" && Number.isInteger(value.exitCode) ? value.exitCode : void 0,
		stderrTail,
		crashReportPath: crashReportPathFromStderr(stderrTail)
	};
}
function terminatedError(context, cause, terminationCause) {
	if (cause instanceof TransportTerminatedError) return cause;
	const details = detailsFromCause(cause);
	const message = cause === void 0 ? context : `${context}: ${describeError(cause)}`;
	const diagnostics = [
		details.exitCode === void 0 ? void 0 : `host exit code: ${details.exitCode}`,
		details.stderrTail === void 0 ? void 0 : `host stderr tail:\n${details.stderrTail}`,
		details.crashReportPath === void 0 ? void 0 : `host crash report: ${details.crashReportPath}`
	].filter((value) => value !== void 0);
	const typedCause = terminationCause ?? (details.exitCode === void 0 ? {
		kind: "io",
		detail: describeError(cause ?? context)
	} : {
		kind: "exit",
		code: details.exitCode
	});
	return new TransportTerminatedError(diagnostics.length === 0 ? message : `${message}\n${diagnostics.join("\n")}`, typedCause, details);
}
function defaultProcessExit(code) {
	const { exit } = processGlobal();
	if (exit) {
		exit(code);
		return;
	}
	throw new Error(`solid-gpui: transport terminated with exit code ${code}`);
}
/**
* Framed stdio connection to the host. The connection that reads the process's
* own stdin also owns the renderer's lifetime: closing the host pipe ends this
* process instead of leaving an orphan that application timers keep alive.
* Intentionally disposed connections and connections over supplied streams
* never end the process; that is the embedder's decision, not the host's.
*/
var StdioTransport = class {
	listeners = /* @__PURE__ */ new Set();
	terminationListeners = /* @__PURE__ */ new Set();
	inputListener;
	inputEndListener;
	inputCloseListener;
	inputErrorListener;
	drainListener;
	outputCloseListener;
	outputErrorListener;
	drainListeners = /* @__PURE__ */ new Set();
	disposed = false;
	terminated = false;
	terminationError;
	input;
	output;
	exitOnHostClose;
	exit;
	tap;
	constructor(options = {}) {
		const stdio = processGlobal();
		const input = options.input ?? stdio.stdin;
		const output = options.output ?? stdio.stdout;
		if (!input) throw new Error("StdioTransport requires a readable stdin");
		if (!output) throw new Error("StdioTransport requires a writable stdout");
		this.input = input;
		this.output = output;
		this.exit = options.exit ?? defaultProcessExit;
		this.exitOnHostClose = options.exitOnHostClose ?? input === stdio.stdin;
		this.tap = ProtocolTap.fromEnv();
		this.inputListener = (chunk) => {
			if (this.disposed || this.terminated) return;
			const bytes = asBytes(chunk);
			this.tap?.observeInbound(bytes);
			for (const listener of this.listeners) listener(bytes);
		};
		this.inputEndListener = () => this.terminate(terminatedError("StdioTransport input ended", void 0, { kind: "eof" }));
		this.inputCloseListener = () => this.terminate(terminatedError("StdioTransport input closed", void 0, { kind: "eof" }));
		this.inputErrorListener = (error) => this.terminate(terminatedError("StdioTransport input failed", error));
		this.drainListener = () => {
			for (const listener of this.drainListeners) listener();
		};
		this.outputCloseListener = () => this.terminate(terminatedError("StdioTransport output closed", void 0, { kind: "eof" }));
		this.outputErrorListener = (error) => this.terminate(terminatedError("StdioTransport output failed", error));
		input.on("data", this.inputListener);
		input.on("end", this.inputEndListener);
		input.on("close", this.inputCloseListener);
		input.on("error", this.inputErrorListener);
		output.on("drain", this.drainListener);
		output.on("close", this.outputCloseListener);
		output.on("error", this.outputErrorListener);
	}
	submit(frame) {
		if (this.terminated) throw this.terminationError;
		if (this.disposed) throw new Error("StdioTransport is disposed");
		try {
			const writable = this.output.write(frame);
			this.tap?.recordOutboundFrame(frame);
			return writable;
		} catch (error) {
			const failure = terminatedError("StdioTransport output write failed", error);
			this.terminate(failure);
			throw failure;
		}
	}
	onDrain(listener) {
		this.drainListeners.add(listener);
		return () => this.drainListeners.delete(listener);
	}
	onData(listener) {
		if (this.terminated) throw this.terminationError;
		if (this.disposed) throw new Error("StdioTransport is disposed");
		this.listeners.add(listener);
		return () => this.listeners.delete(listener);
	}
	onTermination(listener) {
		if (this.terminated) {
			listener(this.terminationError);
			return () => void 0;
		}
		if (this.disposed) return () => void 0;
		this.terminationListeners.add(listener);
		return () => this.terminationListeners.delete(listener);
	}
	dispose() {
		if (this.disposed) return;
		this.disposed = true;
		this.detachListeners();
		this.listeners.clear();
		this.terminationListeners.clear();
		this.drainListeners.clear();
		this.tap?.dispose();
	}
	terminate(error) {
		if (this.disposed || this.terminated) return;
		this.terminated = true;
		this.terminationError = error;
		this.detachListeners();
		this.listeners.clear();
		this.drainListeners.clear();
		this.tap?.dispose();
		const listeners = [...this.terminationListeners];
		this.terminationListeners.clear();
		try {
			for (const listener of listeners) listener(error);
		} finally {
			this.endHostLifetime(error);
		}
	}
	/**
	* Terminating listeners have already observed the reason; end the process
	* afterwards so a closed host pipe cannot leave an orphan renderer whose
	* timers keep the event loop alive. Nothing here touches application-owned
	* children: detached services are separate processes and outlive the renderer.
	*/
	endHostLifetime(error) {
		if (!this.exitOnHostClose) return;
		if (error.cause?.kind === "eof") {
			this.exit(0);
			return;
		}
		console.error(`solid-gpui: transport terminated: ${error.message}`);
		this.exit(1);
	}
	detachListeners() {
		detachInputListener(this.input, "data", this.inputListener);
		detachInputListener(this.input, "end", this.inputEndListener);
		detachInputListener(this.input, "close", this.inputCloseListener);
		detachInputListener(this.input, "error", this.inputErrorListener);
		detachOutputListener(this.output, "drain", this.drainListener);
		detachOutputListener(this.output, "close", this.outputCloseListener);
		detachOutputListener(this.output, "error", this.outputErrorListener);
	}
};
function detachInputListener(input, event, listener) {
	if (input.off) input.off(event, listener);
	else input.removeListener?.(event, listener);
}
function detachOutputListener(output, event, listener) {
	if (output.off) output.off(event, listener);
	else output.removeListener?.(event, listener);
}
/** Bun and Node expose stdio and process exit here; other runtimes reach the host through a bridge. */
function processGlobal() {
	return globalThis.process ?? {};
}
//#endregion
//#region node_modules/.bun/solid-js@1.9.15/node_modules/solid-js/dist/solid.js
var sharedConfig = {
	context: void 0,
	registry: void 0,
	effects: void 0,
	done: false,
	getContextId() {
		return getContextId(this.context.count);
	},
	getNextContextId() {
		return getContextId(this.context.count++);
	}
};
function getContextId(count) {
	const num = String(count), len = num.length - 1;
	return sharedConfig.context.id + (len ? String.fromCharCode(96 + len) : "") + num;
}
function setHydrateContext(context) {
	sharedConfig.context = context;
}
function nextHydrateContext() {
	return {
		...sharedConfig.context,
		id: sharedConfig.getNextContextId(),
		count: 0
	};
}
var equalFn = (a, b) => a === b;
var $PROXY = Symbol("solid-proxy");
var SUPPORTS_PROXY = typeof Proxy === "function";
var $TRACK = Symbol("solid-track");
var signalOptions = { equals: equalFn };
var ERROR$1 = null;
var runEffects = runQueue;
var STALE = 1;
var PENDING = 2;
var UNOWNED = {
	owned: null,
	cleanups: null,
	context: null,
	owner: null
};
var Owner = null;
var Transition = null;
var Scheduler = null;
var ExternalSourceConfig = null;
var Listener = null;
var Updates = null;
var Effects = null;
var ExecCount = 0;
function createRoot$1(fn, detachedOwner) {
	const listener = Listener, owner = Owner, unowned = fn.length === 0, current = detachedOwner === void 0 ? owner : detachedOwner, root = unowned ? UNOWNED : {
		owned: null,
		cleanups: null,
		context: current ? current.context : null,
		owner: current
	}, updateFn = unowned ? fn : () => fn(() => untrack(() => cleanNode(root)));
	Owner = root;
	Listener = null;
	try {
		return runUpdates(updateFn, true);
	} finally {
		Listener = listener;
		Owner = owner;
	}
}
function createSignal(value, options) {
	options = options ? Object.assign({}, signalOptions, options) : signalOptions;
	const s = {
		value,
		observers: null,
		observerSlots: null,
		comparator: options.equals || void 0
	};
	const setter = (value) => {
		if (typeof value === "function") {
			if (Transition && Transition.running && Transition.sources.has(s)) value = value(s.tValue);
			else value = value(s.value);
		}
		return writeSignal(s, value);
	};
	return [readSignal.bind(s), setter];
}
function createComputed(fn, value, options) {
	const c = createComputation(fn, value, true, STALE);
	if (Scheduler && Transition && Transition.running) Updates.push(c);
	else updateComputation(c);
}
function createRenderEffect(fn, value, options) {
	const c = createComputation(fn, value, false, STALE);
	if (Scheduler && Transition && Transition.running) Updates.push(c);
	else updateComputation(c);
}
function createEffect(fn, value, options) {
	runEffects = runUserEffects;
	const c = createComputation(fn, value, false, STALE), s = SuspenseContext && useContext(SuspenseContext);
	if (s) c.suspense = s;
	if (!options || !options.render) c.user = true;
	Effects ? Effects.push(c) : updateComputation(c);
}
function createMemo(fn, value, options) {
	options = options ? Object.assign({}, signalOptions, options) : signalOptions;
	const c = createComputation(fn, value, true, 0);
	c.observers = null;
	c.observerSlots = null;
	c.comparator = options.equals || void 0;
	if (Scheduler && Transition && Transition.running) {
		c.tState = STALE;
		Updates.push(c);
	} else updateComputation(c);
	return readSignal.bind(c);
}
function batch(fn) {
	return runUpdates(fn, false);
}
function untrack(fn) {
	if (!ExternalSourceConfig && Listener === null) return fn();
	const listener = Listener;
	Listener = null;
	try {
		if (ExternalSourceConfig) return ExternalSourceConfig.untrack(fn);
		return fn();
	} finally {
		Listener = listener;
	}
}
function onMount(fn) {
	createEffect(() => untrack(fn));
}
function onCleanup(fn) {
	if (Owner === null);
	else if (Owner.cleanups === null) Owner.cleanups = [fn];
	else Owner.cleanups.push(fn);
	return fn;
}
function getOwner() {
	return Owner;
}
function startTransition(fn) {
	if (Transition && Transition.running) {
		fn();
		return Transition.done;
	}
	const l = Listener;
	const o = Owner;
	return Promise.resolve().then(() => {
		Listener = l;
		Owner = o;
		let t;
		if (Scheduler || SuspenseContext) {
			t = Transition || (Transition = {
				sources: /* @__PURE__ */ new Set(),
				effects: [],
				promises: /* @__PURE__ */ new Set(),
				disposed: /* @__PURE__ */ new Set(),
				queue: /* @__PURE__ */ new Set(),
				running: true
			});
			t.done || (t.done = new Promise((res) => t.resolve = res));
			t.running = true;
		}
		runUpdates(fn, false);
		Listener = Owner = null;
		return t ? t.done : void 0;
	});
}
var [transPending, setTransPending] = /*@__PURE__*/ createSignal(false);
function createContext(defaultValue, options) {
	const id = Symbol("context");
	return {
		id,
		Provider: createProvider(id),
		defaultValue
	};
}
function useContext(context) {
	let value;
	return Owner && Owner.context && (value = Owner.context[context.id]) !== void 0 ? value : context.defaultValue;
}
function children(fn) {
	const children = createMemo(fn);
	const memo = createMemo(() => resolveChildren(children()));
	memo.toArray = () => {
		const c = memo();
		return Array.isArray(c) ? c : c != null ? [c] : [];
	};
	return memo;
}
var SuspenseContext;
function readSignal() {
	const runningTransition = Transition && Transition.running;
	if (this.sources && (runningTransition ? this.tState : this.state)) {
		if ((runningTransition ? this.tState : this.state) === STALE) updateComputation(this);
		else {
			const updates = Updates;
			Updates = null;
			runUpdates(() => lookUpstream(this), false);
			Updates = updates;
		}
	}
	if (Listener) {
		const observers = this.observers;
		if (!observers || observers[observers.length - 1] !== Listener) {
			const sSlot = observers ? observers.length : 0;
			if (!Listener.sources) {
				Listener.sources = [this];
				Listener.sourceSlots = [sSlot];
			} else {
				Listener.sources.push(this);
				Listener.sourceSlots.push(sSlot);
			}
			if (!observers) {
				this.observers = [Listener];
				this.observerSlots = [Listener.sources.length - 1];
			} else {
				observers.push(Listener);
				this.observerSlots.push(Listener.sources.length - 1);
			}
		}
	}
	if (runningTransition && Transition.sources.has(this)) return this.tValue;
	return this.value;
}
function writeSignal(node, value, isComp) {
	let current = Transition && Transition.running && Transition.sources.has(node) ? node.tValue : node.value;
	if (!node.comparator || !node.comparator(current, value)) {
		if (Transition) {
			const TransitionRunning = Transition.running;
			if (TransitionRunning || !isComp && Transition.sources.has(node)) {
				Transition.sources.add(node);
				node.tValue = value;
			}
			if (!TransitionRunning) node.value = value;
		} else node.value = value;
		if (node.observers && node.observers.length) runUpdates(() => {
			for (let i = 0; i < node.observers.length; i += 1) {
				const o = node.observers[i];
				const TransitionRunning = Transition && Transition.running;
				if (TransitionRunning && Transition.disposed.has(o)) continue;
				if (TransitionRunning ? !o.tState : !o.state) {
					if (o.pure) Updates.push(o);
					else Effects.push(o);
					if (o.observers) markDownstream(o);
				}
				if (!TransitionRunning) o.state = STALE;
				else o.tState = STALE;
			}
			if (Updates.length > 1e6) {
				Updates = [];
				throw new Error();
			}
		}, false);
	}
	return value;
}
function updateComputation(node) {
	if (!node.fn) return;
	cleanNode(node);
	const time = ExecCount;
	runComputation(node, Transition && Transition.running && Transition.sources.has(node) ? node.tValue : node.value, time);
	if (Transition && !Transition.running && Transition.sources.has(node)) queueMicrotask(() => {
		runUpdates(() => {
			Transition && (Transition.running = true);
			Listener = Owner = node;
			runComputation(node, node.tValue, time);
			Listener = Owner = null;
		}, false);
	});
}
function runComputation(node, value, time) {
	let nextValue;
	const owner = Owner, listener = Listener;
	Listener = Owner = node;
	try {
		nextValue = node.fn(value);
	} catch (err) {
		if (node.pure) {
			if (Transition && Transition.running) {
				node.tState = STALE;
				node.tOwned && node.tOwned.forEach(cleanNode);
				node.tOwned = void 0;
			} else {
				node.state = STALE;
				node.owned && node.owned.forEach(cleanNode);
				node.owned = null;
			}
		}
		node.updatedAt = time + 1;
		return handleError(err);
	} finally {
		Listener = listener;
		Owner = owner;
	}
	if (!node.updatedAt || node.updatedAt <= time) {
		if (node.updatedAt != null && "observers" in node) writeSignal(node, nextValue, true);
		else if (Transition && Transition.running && node.pure) {
			if (!Transition.sources.has(node)) node.value = nextValue;
			Transition.sources.add(node);
			node.tValue = nextValue;
		} else node.value = nextValue;
		node.updatedAt = time;
	}
}
function createComputation(fn, init, pure, state = STALE, options) {
	const c = {
		fn,
		state,
		updatedAt: null,
		owned: null,
		sources: null,
		sourceSlots: null,
		cleanups: null,
		value: init,
		owner: Owner,
		context: Owner ? Owner.context : null,
		pure
	};
	if (Transition && Transition.running) {
		c.state = 0;
		c.tState = state;
	}
	if (Owner === null);
	else if (Owner !== UNOWNED) {
		if (Transition && Transition.running && Owner.pure) {
			if (!Owner.tOwned) Owner.tOwned = [c];
			else Owner.tOwned.push(c);
		} else if (!Owner.owned) Owner.owned = [c];
		else Owner.owned.push(c);
	}
	if (ExternalSourceConfig && c.fn) {
		const sourceFn = c.fn;
		const [track, trigger] = createSignal(void 0, { equals: false });
		const ordinary = ExternalSourceConfig.factory(sourceFn, trigger);
		onCleanup(() => ordinary.dispose());
		let inTransition;
		let trackedOrdinary = false;
		const triggerInTransition = () => startTransition(trigger).then(() => {
			if (inTransition) {
				inTransition.dispose();
				inTransition = void 0;
				if (!trackedOrdinary) trigger();
			}
		});
		c.fn = (x) => {
			track();
			if (Transition && Transition.running) {
				if (!inTransition) inTransition = ExternalSourceConfig.factory(sourceFn, triggerInTransition);
				return inTransition.track(x);
			}
			trackedOrdinary = true;
			return ordinary.track(x);
		};
	}
	return c;
}
function runTop(node) {
	const runningTransition = Transition && Transition.running;
	if ((runningTransition ? node.tState : node.state) === 0) return;
	if ((runningTransition ? node.tState : node.state) === PENDING) return lookUpstream(node);
	if (node.suspense && untrack(node.suspense.inFallback)) return node.suspense.effects.push(node);
	const ancestors = [node];
	while ((node = node.owner) && (!node.updatedAt || node.updatedAt < ExecCount)) {
		if (runningTransition && Transition.disposed.has(node)) return;
		if (runningTransition ? node.tState : node.state) ancestors.push(node);
	}
	for (let i = ancestors.length - 1; i >= 0; i--) {
		node = ancestors[i];
		if (runningTransition) {
			let top = node, prev = ancestors[i + 1];
			while ((top = top.owner) && top !== prev) if (Transition.disposed.has(top)) return;
		}
		if ((runningTransition ? node.tState : node.state) === STALE) updateComputation(node);
		else if ((runningTransition ? node.tState : node.state) === PENDING) {
			const updates = Updates;
			Updates = null;
			runUpdates(() => lookUpstream(node, ancestors[0]), false);
			Updates = updates;
		}
	}
}
function runUpdates(fn, init) {
	if (Updates) return fn();
	let wait = false;
	if (!init) Updates = [];
	if (Effects) wait = true;
	else Effects = [];
	ExecCount++;
	try {
		const res = fn();
		completeUpdates(wait);
		return res;
	} catch (err) {
		if (!wait) Effects = null;
		Updates = null;
		handleError(err);
	}
}
function completeUpdates(wait) {
	if (Updates) {
		if (Scheduler && Transition && Transition.running) scheduleQueue(Updates);
		else runQueue(Updates);
		Updates = null;
	}
	if (wait) return;
	let res;
	if (Transition) {
		if (!Transition.promises.size && !Transition.queue.size) {
			const sources = Transition.sources;
			const disposed = Transition.disposed;
			Effects.push.apply(Effects, Transition.effects);
			res = Transition.resolve;
			for (const e of Effects) {
				"tState" in e && (e.state = e.tState);
				delete e.tState;
			}
			Transition = null;
			runUpdates(() => {
				for (const d of disposed) cleanNode(d);
				for (const v of sources) {
					v.value = v.tValue;
					if (v.owned) for (let i = 0, len = v.owned.length; i < len; i++) cleanNode(v.owned[i]);
					if (v.tOwned) v.owned = v.tOwned;
					delete v.tValue;
					delete v.tOwned;
					v.tState = 0;
				}
				setTransPending(false);
			}, false);
		} else if (Transition.running) {
			Transition.running = false;
			Transition.effects.push.apply(Transition.effects, Effects);
			Effects = null;
			setTransPending(true);
			return;
		}
	}
	const e = Effects;
	Effects = null;
	if (e.length) runUpdates(() => runEffects(e), false);
	if (res) res();
}
function runQueue(queue) {
	for (let i = 0; i < queue.length; i++) runTop(queue[i]);
}
function scheduleQueue(queue) {
	for (let i = 0; i < queue.length; i++) {
		const item = queue[i];
		const tasks = Transition.queue;
		if (!tasks.has(item)) {
			tasks.add(item);
			Scheduler(() => {
				tasks.delete(item);
				runUpdates(() => {
					Transition.running = true;
					runTop(item);
				}, false);
				Transition && (Transition.running = false);
			});
		}
	}
}
function runUserEffects(queue) {
	let i, userLength = 0;
	for (i = 0; i < queue.length; i++) {
		const e = queue[i];
		if (!e.user) runTop(e);
		else queue[userLength++] = e;
	}
	if (sharedConfig.context) {
		if (sharedConfig.count) {
			sharedConfig.effects || (sharedConfig.effects = []);
			sharedConfig.effects.push(...queue.slice(0, userLength));
			return;
		}
		setHydrateContext();
	}
	if (sharedConfig.effects && (sharedConfig.done || !sharedConfig.count)) {
		queue = [...sharedConfig.effects, ...queue];
		userLength += sharedConfig.effects.length;
		delete sharedConfig.effects;
	}
	for (i = 0; i < userLength; i++) runTop(queue[i]);
}
function lookUpstream(node, ignore) {
	const runningTransition = Transition && Transition.running;
	if (runningTransition) node.tState = 0;
	else node.state = 0;
	for (let i = 0; i < node.sources.length; i += 1) {
		const source = node.sources[i];
		if (source.sources) {
			const state = runningTransition ? source.tState : source.state;
			if (state === STALE) {
				if (source !== ignore && (!source.updatedAt || source.updatedAt < ExecCount)) runTop(source);
			} else if (state === PENDING) lookUpstream(source, ignore);
		}
	}
}
function markDownstream(node) {
	const runningTransition = Transition && Transition.running;
	for (let i = 0; i < node.observers.length; i += 1) {
		const o = node.observers[i];
		if (runningTransition ? !o.tState : !o.state) {
			if (runningTransition) o.tState = PENDING;
			else o.state = PENDING;
			if (o.pure) Updates.push(o);
			else Effects.push(o);
			o.observers && markDownstream(o);
		}
	}
}
function cleanNode(node) {
	let i;
	if (node.sources) while (node.sources.length) {
		const source = node.sources.pop(), index = node.sourceSlots.pop(), obs = source.observers;
		if (obs && obs.length) {
			const n = obs.pop(), s = source.observerSlots.pop();
			if (index < obs.length) {
				n.sourceSlots[s] = index;
				obs[index] = n;
				source.observerSlots[index] = s;
			}
		}
	}
	if (node.tOwned) {
		for (i = node.tOwned.length - 1; i >= 0; i--) cleanNode(node.tOwned[i]);
		delete node.tOwned;
	}
	if (Transition && Transition.running && node.pure) reset(node, true);
	else if (node.owned) {
		for (i = node.owned.length - 1; i >= 0; i--) cleanNode(node.owned[i]);
		node.owned = null;
	}
	if (node.cleanups) {
		for (i = node.cleanups.length - 1; i >= 0; i--) node.cleanups[i]();
		node.cleanups = null;
	}
	if (Transition && Transition.running) node.tState = 0;
	else node.state = 0;
}
function reset(node, top) {
	if (!top) {
		node.tState = 0;
		Transition.disposed.add(node);
	}
	if (node.owned) for (let i = 0; i < node.owned.length; i++) reset(node.owned[i]);
}
function castError(err) {
	if (err instanceof Error) return err;
	return new Error(typeof err === "string" ? err : "Unknown error", { cause: err });
}
function runErrors(err, fns, owner) {
	try {
		for (const f of fns) f(err);
	} catch (e) {
		handleError(e, owner && owner.owner || null);
	}
}
function handleError(err, owner = Owner) {
	const fns = ERROR$1 && owner && owner.context && owner.context[ERROR$1];
	const error = castError(err);
	if (!fns) throw error;
	if (Effects) Effects.push({
		fn() {
			runErrors(error, fns, owner);
		},
		state: STALE
	});
	else runErrors(error, fns, owner);
}
function resolveChildren(children) {
	if (typeof children === "function" && !children.length) return resolveChildren(children());
	if (Array.isArray(children)) {
		const results = [];
		for (let i = 0; i < children.length; i++) {
			const result = resolveChildren(children[i]);
			if (Array.isArray(result)) {
				if (result.length < 32768) results.push.apply(results, result);
				else for (let j = 0; j < result.length; j++) results.push(result[j]);
			} else results.push(result);
		}
		return results;
	}
	return children;
}
function createProvider(id, options) {
	return function provider(props) {
		let res;
		createRenderEffect(() => res = untrack(() => {
			Owner.context = {
				...Owner.context,
				[id]: props.value
			};
			return children(() => props.children);
		}), void 0);
		return res;
	};
}
var FALLBACK = Symbol("fallback");
function dispose(d) {
	for (let i = 0; i < d.length; i++) d[i]();
}
function mapArray(list, mapFn, options = {}) {
	let items = [], mapped = [], disposers = [], len = 0, indexes = mapFn.length > 1 ? [] : null;
	onCleanup(() => dispose(disposers));
	return () => {
		let newItems = list() || [], newLen = newItems.length, i, j;
		newItems[$TRACK];
		return untrack(() => {
			let newIndices, newIndicesNext, temp, tempdisposers, tempIndexes, start, end, newEnd, item;
			if (newLen === 0) {
				if (len !== 0) {
					dispose(disposers);
					disposers = [];
					items = [];
					mapped = [];
					len = 0;
					indexes && (indexes = []);
				}
				if (options.fallback) {
					items = [FALLBACK];
					mapped[0] = createRoot$1((disposer) => {
						disposers[0] = disposer;
						return options.fallback();
					});
					len = 1;
				}
			} else if (len === 0) {
				mapped = new Array(newLen);
				for (j = 0; j < newLen; j++) {
					items[j] = newItems[j];
					mapped[j] = createRoot$1(mapper);
				}
				len = newLen;
			} else {
				temp = new Array(newLen);
				tempdisposers = new Array(newLen);
				indexes && (tempIndexes = new Array(newLen));
				for (start = 0, end = Math.min(len, newLen); start < end && items[start] === newItems[start]; start++);
				for (end = len - 1, newEnd = newLen - 1; end >= start && newEnd >= start && items[end] === newItems[newEnd]; end--, newEnd--) {
					temp[newEnd] = mapped[end];
					tempdisposers[newEnd] = disposers[end];
					indexes && (tempIndexes[newEnd] = indexes[end]);
				}
				newIndices = /* @__PURE__ */ new Map();
				newIndicesNext = new Array(newEnd + 1);
				for (j = newEnd; j >= start; j--) {
					item = newItems[j];
					i = newIndices.get(item);
					newIndicesNext[j] = i === void 0 ? -1 : i;
					newIndices.set(item, j);
				}
				for (i = start; i <= end; i++) {
					item = items[i];
					j = newIndices.get(item);
					if (j !== void 0 && j !== -1) {
						temp[j] = mapped[i];
						tempdisposers[j] = disposers[i];
						indexes && (tempIndexes[j] = indexes[i]);
						j = newIndicesNext[j];
						newIndices.set(item, j);
					} else disposers[i]();
				}
				for (j = start; j < newLen; j++) if (j in temp) {
					mapped[j] = temp[j];
					disposers[j] = tempdisposers[j];
					if (indexes) {
						indexes[j] = tempIndexes[j];
						indexes[j](j);
					}
				} else mapped[j] = createRoot$1(mapper);
				mapped = mapped.slice(0, len = newLen);
				items = newItems.slice(0);
			}
			return mapped;
		});
		function mapper(disposer) {
			disposers[j] = disposer;
			if (indexes) {
				const [s, set] = createSignal(j);
				indexes[j] = set;
				return mapFn(newItems[j], s);
			}
			return mapFn(newItems[j]);
		}
	};
}
var hydrationEnabled = false;
function createComponent$1(Comp, props) {
	if (hydrationEnabled) {
		if (sharedConfig.context) {
			const c = sharedConfig.context;
			setHydrateContext(nextHydrateContext());
			const r = untrack(() => Comp(props || {}));
			setHydrateContext(c);
			return r;
		}
	}
	return untrack(() => Comp(props || {}));
}
function trueFn() {
	return true;
}
var propTraps = {
	get(_, property, receiver) {
		if (property === $PROXY) return receiver;
		return _.get(property);
	},
	has(_, property) {
		if (property === $PROXY) return true;
		return _.has(property);
	},
	set: trueFn,
	deleteProperty: trueFn,
	getOwnPropertyDescriptor(_, property) {
		return {
			configurable: true,
			enumerable: true,
			get() {
				return _.get(property);
			},
			set: trueFn,
			deleteProperty: trueFn
		};
	},
	ownKeys(_) {
		return _.keys();
	}
};
function resolveSource(s) {
	return !(s = typeof s === "function" ? s() : s) ? {} : s;
}
function resolveSources() {
	for (let i = 0, length = this.length; i < length; ++i) {
		const v = this[i]();
		if (v !== void 0) return v;
	}
}
function mergeProps$1(...sources) {
	let proxy = false;
	for (let i = 0; i < sources.length; i++) {
		const s = sources[i];
		proxy = proxy || !!s && $PROXY in s;
		sources[i] = typeof s === "function" ? (proxy = true, createMemo(s)) : s;
	}
	if (SUPPORTS_PROXY && proxy) return new Proxy({
		get(property) {
			for (let i = sources.length - 1; i >= 0; i--) {
				const v = resolveSource(sources[i])[property];
				if (v !== void 0) return v;
			}
		},
		has(property) {
			for (let i = sources.length - 1; i >= 0; i--) if (property in resolveSource(sources[i])) return true;
			return false;
		},
		keys() {
			const keys = [];
			for (let i = 0; i < sources.length; i++) keys.push(...Object.keys(resolveSource(sources[i])));
			return [...new Set(keys)];
		}
	}, propTraps);
	const sourcesMap = {};
	const defined = Object.create(null);
	for (let i = sources.length - 1; i >= 0; i--) {
		const source = sources[i];
		if (!source) continue;
		const sourceKeys = Object.getOwnPropertyNames(source);
		for (let i = sourceKeys.length - 1; i >= 0; i--) {
			const key = sourceKeys[i];
			if (key === "__proto__" || key === "constructor") continue;
			const desc = Object.getOwnPropertyDescriptor(source, key);
			if (!defined[key]) defined[key] = desc.get ? {
				enumerable: true,
				configurable: true,
				get: resolveSources.bind(sourcesMap[key] = [desc.get.bind(source)])
			} : desc.value !== void 0 ? desc : void 0;
			else {
				const sources = sourcesMap[key];
				if (sources) {
					if (desc.get) sources.push(desc.get.bind(source));
					else if (desc.value !== void 0) sources.push(() => desc.value);
				}
			}
		}
	}
	const target = {};
	const definedKeys = Object.keys(defined);
	for (let i = definedKeys.length - 1; i >= 0; i--) {
		const key = definedKeys[i], desc = defined[key];
		if (desc && desc.get) Object.defineProperty(target, key, desc);
		else target[key] = desc ? desc.value : void 0;
	}
	return target;
}
function For$1(props) {
	const fallback = "fallback" in props && { fallback: () => props.fallback };
	return createMemo(mapArray(() => props.each, props.children, fallback || void 0));
}
//#endregion
//#region packages/solid-gpui/src/runtime-guard.ts
/**
* Importing the SDK under a non-client resolution used to leave signals inert:
* Solid's server build never schedules the renderer, so a mounted application
* submits a snapshot and then nothing at all. A duplicated Solid link fails the
* same silent way, with signals on one side invisible to the other.
*
* Both are decided structurally, not by observing effects: Solid's server build
* aliases `createRenderEffect` to `createComputed`, and a second Solid copy
* brings its own functions. Neither check schedules work, so they hold whatever
* batching context a root is created in, and tools that import the SDK for its
* utilities or types are unaffected because the checks run only when a renderer
* root is created.
*/
var SERVER_SOLID = "solid-gpui: solid-js resolved to its server build, so signals never notify this renderer. Resolve Solid's client build (Bun: --conditions=browser, Vite: resolve.conditions) or run tests through @solid-gpui/vite/test, which does it for you.";
var DUPLICATE_SOLID = "solid-gpui: two copies of solid-js are loaded, so signals created by one copy never notify the other. Deduplicate solid-js (Vite: resolve.dedupe: [\"solid-js\"]) so the application and the SDK share one reactive graph.";
var REGISTRY = Symbol.for("@solid-gpui/core/solid");
var solid = {
	createSignal,
	createRenderEffect,
	createComputed
};
var registry = globalThis;
var previous = registry[REGISTRY];
if (!previous) registry[REGISTRY] = solid;
var verified = false;
/** Refuse to start a renderer that cannot observe the application's signals. */
function assertSolidRuntime() {
	if (verified) return;
	if (createRenderEffect === createComputed) throw new Error(SERVER_SOLID);
	if (previous && previous.createSignal !== createSignal) throw new Error(DUPLICATE_SOLID);
	verified = true;
}
//#endregion
//#region packages/solid-gpui/src/generation.ts
function generationHost() {
	return globalThis.__solidGpuiGeneration;
}
//#endregion
//#region packages/solid-gpui/src/refs.ts
/** Compiler ref ABI: preserve the owner, suppress tracking, and ignore callback return values. */
function applyRef(ref, value) {
	untrack(() => {
		if (Array.isArray(ref)) for (const callback of ref) applyRef(callback, value);
		else if (ref) ref(value);
	});
}
//#endregion
//#region node_modules/.bun/solid-js@1.9.15/node_modules/solid-js/universal/dist/universal.js
var memo$1 = (fn) => createMemo(() => fn());
function createRenderer$1({ createElement, createTextNode, isTextNode, replaceText, insertNode, removeNode, setProperty, getParentNode, getFirstChild, getNextSibling }) {
	function insert(parent, accessor, marker, initial) {
		if (marker !== void 0 && !initial) initial = [];
		if (typeof accessor !== "function") return insertExpression(parent, accessor, initial, marker);
		createRenderEffect((current) => insertExpression(parent, accessor(), current, marker), initial);
	}
	function insertExpression(parent, value, current, marker, unwrapArray) {
		while (typeof current === "function") current = current();
		if (value === current) return current;
		const t = typeof value, multi = marker !== void 0;
		if (t === "string" || t === "number") {
			if (t === "number") value = value.toString();
			if (multi) {
				let node = current[0];
				if (node && isTextNode(node)) replaceText(node, value);
				else node = createTextNode(value);
				current = cleanChildren(parent, current, marker, node);
			} else if (current !== "" && typeof current === "string") replaceText(getFirstChild(parent), current = value);
			else {
				cleanChildren(parent, current, marker, createTextNode(value));
				current = value;
			}
		} else if (value == null || t === "boolean") current = cleanChildren(parent, current, marker);
		else if (t === "function") {
			createRenderEffect(() => {
				let v = value();
				while (typeof v === "function") v = v();
				current = insertExpression(parent, v, current, marker);
			});
			return () => current;
		} else if (Array.isArray(value)) {
			const array = [];
			if (normalizeIncomingArray(array, value, unwrapArray)) {
				createRenderEffect(() => current = insertExpression(parent, array, current, marker, true));
				return () => current;
			}
			if (array.length === 0) {
				const replacement = cleanChildren(parent, current, marker);
				if (multi) return current = replacement;
			} else if (Array.isArray(current)) {
				if (current.length === 0) appendNodes(parent, array, marker);
				else reconcileArrays(parent, current, array);
			} else if (current == null || current === "") appendNodes(parent, array);
			else reconcileArrays(parent, multi && current || [getFirstChild(parent)], array);
			current = array;
		} else {
			if (Array.isArray(current)) {
				if (multi) return current = cleanChildren(parent, current, marker, value);
				cleanChildren(parent, current, null, value);
			} else if (current == null || current === "" || !getFirstChild(parent)) insertNode(parent, value);
			else replaceNode(parent, value, getFirstChild(parent));
			current = value;
		}
		return current;
	}
	function normalizeIncomingArray(normalized, array, unwrap) {
		let dynamic = false;
		for (let i = 0, len = array.length; i < len; i++) {
			let item = array[i], t;
			if (item == null || item === true || item === false);
			else if (Array.isArray(item)) dynamic = normalizeIncomingArray(normalized, item) || dynamic;
			else if ((t = typeof item) === "string" || t === "number") normalized.push(createTextNode(item));
			else if (t === "function") {
				if (unwrap) {
					while (typeof item === "function") item = item();
					dynamic = normalizeIncomingArray(normalized, Array.isArray(item) ? item : [item]) || dynamic;
				} else {
					normalized.push(item);
					dynamic = true;
				}
			} else normalized.push(item);
		}
		return dynamic;
	}
	function reconcileArrays(parentNode, a, b) {
		let bLength = b.length, aEnd = a.length, bEnd = bLength, aStart = 0, bStart = 0, after = getNextSibling(a[aEnd - 1]), map = null;
		while (aStart < aEnd || bStart < bEnd) {
			if (a[aStart] === b[bStart]) {
				aStart++;
				bStart++;
				continue;
			}
			while (a[aEnd - 1] === b[bEnd - 1]) {
				aEnd--;
				bEnd--;
			}
			if (aEnd === aStart) {
				const node = bEnd < bLength ? bStart ? getNextSibling(b[bStart - 1]) : b[bEnd - bStart] : after;
				while (bStart < bEnd) insertNode(parentNode, b[bStart++], node);
			} else if (bEnd === bStart) while (aStart < aEnd) {
				if (!map || !map.has(a[aStart])) removeNode(parentNode, a[aStart]);
				aStart++;
			}
			else if (a[aStart] === b[bEnd - 1] && b[bStart] === a[aEnd - 1]) {
				const node = getNextSibling(a[--aEnd]);
				insertNode(parentNode, b[bStart++], getNextSibling(a[aStart++]));
				insertNode(parentNode, b[--bEnd], node);
				a[aEnd] = b[bEnd];
			} else {
				if (!map) {
					map = /* @__PURE__ */ new Map();
					let i = bStart;
					while (i < bEnd) map.set(b[i], i++);
				}
				const index = map.get(a[aStart]);
				if (index != null) {
					if (bStart < index && index < bEnd) {
						let i = aStart, sequence = 1, t;
						while (++i < aEnd && i < bEnd) {
							if ((t = map.get(a[i])) == null || t !== index + sequence) break;
							sequence++;
						}
						if (sequence > index - bStart) {
							const node = a[aStart];
							while (bStart < index) insertNode(parentNode, b[bStart++], node);
						} else replaceNode(parentNode, b[bStart++], a[aStart++]);
					} else aStart++;
				} else removeNode(parentNode, a[aStart++]);
			}
		}
	}
	function cleanChildren(parent, current, marker, replacement) {
		if (marker === void 0) {
			let removed;
			while (removed = getFirstChild(parent)) removeNode(parent, removed);
			replacement && insertNode(parent, replacement);
			return "";
		}
		const node = replacement || createTextNode("");
		if (current.length) {
			let inserted = false;
			for (let i = current.length - 1; i >= 0; i--) {
				const el = current[i];
				if (node !== el) {
					const isParent = getParentNode(el) === parent;
					if (!inserted && !i) isParent ? replaceNode(parent, node, el) : insertNode(parent, node, marker);
					else isParent && removeNode(parent, el);
				} else inserted = true;
			}
		} else insertNode(parent, node, marker);
		return [node];
	}
	function appendNodes(parent, array, marker) {
		for (let i = 0, len = array.length; i < len; i++) insertNode(parent, array[i], marker);
	}
	function replaceNode(parent, newNode, oldNode) {
		insertNode(parent, newNode, oldNode);
		removeNode(parent, oldNode);
	}
	function spreadExpression(node, props, prevProps = {}, skipChildren) {
		props || (props = {});
		if (!skipChildren) createRenderEffect(() => prevProps.children = insertExpression(node, props.children, prevProps.children));
		createRenderEffect(() => props.ref && props.ref(node));
		createRenderEffect(() => {
			for (const prop in props) {
				if (prop === "children" || prop === "ref") continue;
				const value = props[prop];
				if (value === prevProps[prop]) continue;
				setProperty(node, prop, value, prevProps[prop]);
				prevProps[prop] = value;
			}
		});
		return prevProps;
	}
	return {
		render(code, element) {
			let disposer;
			createRoot$1((dispose) => {
				disposer = dispose;
				insert(element, code());
			});
			return disposer;
		},
		insert,
		spread(node, accessor, skipChildren) {
			if (typeof accessor === "function") createRenderEffect((current) => spreadExpression(node, accessor(), current, skipChildren));
			else spreadExpression(node, accessor, void 0, skipChildren);
		},
		createElement,
		createTextNode,
		insertNode,
		setProp(node, name, value, prev) {
			setProperty(node, name, value, prev);
			return value;
		},
		mergeProps: mergeProps$1,
		effect: createRenderEffect,
		memo: memo$1,
		createComponent: createComponent$1,
		use(fn, element, arg) {
			return untrack(() => fn(element, arg));
		}
	};
}
function createRenderer(options) {
	const renderer = createRenderer$1(options);
	renderer.mergeProps = mergeProps$1;
	return renderer;
}
//#endregion
//#region packages/solid-gpui/src/renderer/facts.ts
var VALID_HOST_TYPES = {
	View: true,
	Text: true,
	Pressable: true,
	TextInput: true,
	VirtualList: true,
	Image: true,
	Extension: true,
	Icon: true
};
var ROLE_CODES = {
	generic: 1,
	button: 2,
	text: 3,
	textbox: 4,
	checkbox: 5,
	heading: 6,
	link: 7,
	status: 8,
	alert: 9,
	group: 10,
	list: 11,
	listitem: 12,
	dialog: 13
};
var PROP_GROUP_STYLE = 1;
var PROP_GROUP_ACCESSIBILITY = 2;
var PROP_GROUP_INTERACTION = 4;
var PROP_GROUP_HOST_PROPERTIES = 8;
var PROP_GROUP_TOOLTIP = 16;
var ACCESSIBILITY_PROPS = {
	accessibilityRole: true,
	accessibilityLabel: true,
	accessibilityDescription: true,
	accessibilityDisabled: true,
	accessibilityChecked: true,
	accessibilitySelected: true,
	accessibilityValue: true,
	accessibilityExpanded: true,
	accessibilityLevel: true,
	accessibilityLive: true
};
var BASE_PROPS = {
	style: true,
	children: true,
	ref: true,
	...ACCESSIBILITY_PROPS
};
var HOST_KIND_SPECS = {
	View: {
		allowedProps: {
			...BASE_PROPS,
			tooltip: true,
			onLayout: true,
			draggable: true,
			onDragOver: true,
			onDrop: true,
			onExternalFileDrop: true,
			focusable: true,
			onKeyDown: true,
			onPointerDown: true,
			onPointerUp: true,
			onPointerMove: true,
			onHoverChange: true,
			onScroll: true,
			onFocus: true,
			onBlur: true,
			onPointerDownOutside: true
		},
		projector: "drag",
		allowsLayout: true
	},
	Text: {
		allowedProps: {
			...BASE_PROPS,
			selectable: true,
			onPress: true,
			onFocus: true,
			onBlur: true,
			onLayout: true
		},
		projector: "none",
		allowsLayout: true
	},
	Pressable: {
		allowedProps: {
			...BASE_PROPS,
			tooltip: true,
			onLayout: true,
			onPress: true,
			disabled: true,
			draggable: true,
			onDragOver: true,
			onDrop: true,
			onExternalFileDrop: true,
			focusable: true,
			onKeyDown: true,
			onPointerDown: true,
			onPointerUp: true,
			onPointerMove: true,
			onHoverChange: true,
			onFocus: true,
			onBlur: true
		},
		defaultAccessibilityRole: "button",
		projector: "drag",
		allowsLayout: true
	},
	TextInput: {
		allowedProps: {
			...BASE_PROPS,
			value: true,
			defaultValue: true,
			placeholder: true,
			onChangeText: true,
			onSelectionChange: true,
			onFocus: true,
			onBlur: true,
			onSubmitEditing: true,
			onKeyDown: true,
			multiline: true,
			disabled: true,
			maxLength: true
		},
		defaultAccessibilityRole: "textbox",
		projector: "input",
		allowsLayout: false
	},
	VirtualList: {
		allowedProps: {
			...BASE_PROPS,
			__itemCount: true,
			__data: true,
			__rangeStart: true,
			__rangeEnd: true,
			__estimatedItemSize: true,
			__overscan: true,
			__onVisibleRange: true,
			__onAnimationComplete: true
		},
		projector: "virtualList",
		allowsLayout: false
	},
	Image: {
		allowedProps: {
			...BASE_PROPS,
			source: true,
			sourceSet: true,
			fallbackSource: true,
			objectFit: true,
			onLayout: true
		},
		projector: "image",
		allowsLayout: true
	},
	Icon: {
		allowedProps: {
			...BASE_PROPS,
			name: true,
			size: true,
			color: true,
			onLayout: true
		},
		projector: "icon",
		allowsLayout: true
	},
	RawText: {
		allowedProps: {
			children: true,
			ref: true
		},
		projector: "none",
		allowsLayout: false
	},
	Extension: {
		allowedProps: {
			...BASE_PROPS,
			onExtensionEvent: true,
			onLayout: true,
			__extensionDescriptor: true
		},
		projector: "none",
		allowsLayout: true
	}
};
function propGroup(kind, name) {
	if (name === "style") return PROP_GROUP_STYLE;
	if (name.startsWith("accessibility")) return PROP_GROUP_ACCESSIBILITY;
	if (name === "onExtensionEvent") return PROP_GROUP_INTERACTION;
	if (kind === "Extension") {
		if (name === "children" || name === "ref" || name === "onLayout") return PROP_GROUP_INTERACTION;
		return PROP_GROUP_HOST_PROPERTIES;
	}
	if (name === "tooltip") return PROP_GROUP_TOOLTIP;
	if (name === "onPress" || name === "focusable" || name === "selectable" || name === "disabled" || name === "onKeyDown" || name === "onPointerDown" || name === "onPointerUp" || name === "onPointerMove" || name === "onHoverChange" || name === "onFocus" || name === "onBlur" || name === "onPointerDownOutside" || name === "onScroll" || name === "onLayout" || name === "draggable" || name === "onDragOver" || name === "onDrop" || name === "onExternalFileDrop") return PROP_GROUP_INTERACTION;
	if (kind === "TextInput" || kind === "VirtualList" || kind === "Image" || kind === "Icon") return PROP_GROUP_HOST_PROPERTIES;
	return PROP_GROUP_INTERACTION;
}
function isAccessibilityProp(name) {
	return Object.hasOwn(ACCESSIBILITY_PROPS, name);
}
function hostKindSpec(kind) {
	return HOST_KIND_SPECS[kind];
}
function hasAllowedProp(kind, name) {
	return Object.hasOwn(HOST_KIND_SPECS[kind].allowedProps, name);
}
function defaultAccessibilityRole(kind) {
	return HOST_KIND_SPECS[kind].defaultAccessibilityRole;
}
function assertChildKind(parentKind, childKind, textDepth = 0) {
	if (parentKind === null) return;
	if (parentKind === "Text" && childKind !== "RawText" && childKind !== "Text") throw new TypeError("Text children must be raw text or one-level nested Text runs");
	if (parentKind === "Text" && childKind === "Text" && textDepth > 1) throw new TypeError("Nested Text may contain only raw text; deeper Text nesting is unsupported");
	if (parentKind === "Image" || parentKind === "Icon") throw new TypeError(`${parentKind} nodes cannot contain children`);
	if (childKind === "RawText" && parentKind !== "Text") throw new TypeError("Raw text is only valid directly under Text");
}
function validateHostKindValues(kind, props) {
	if (kind === "Pressable" && props.onPress !== void 0 && typeof props.onPress !== "function") throw new TypeError("Pressable onPress must be a function");
	if (kind !== "TextInput") return;
	const input = props;
	for (const [name, callback] of [
		["onChangeText", input.onChangeText],
		["onSelectionChange", input.onSelectionChange],
		["onFocus", input.onFocus],
		["onBlur", input.onBlur],
		["onSubmitEditing", input.onSubmitEditing],
		["onKeyDown", input.onKeyDown]
	]) if (callback !== void 0 && typeof callback !== "function") throw new TypeError(`TextInput ${name} must be a function`);
	for (const [name, value] of [
		["value", input.value],
		["defaultValue", input.defaultValue],
		["placeholder", input.placeholder]
	]) if (value !== void 0 && typeof value !== "string") throw new TypeError(`TextInput ${name} must be a string`);
	if (input.multiline !== void 0 && typeof input.multiline !== "boolean") throw new TypeError("TextInput multiline must be a boolean");
	if (input.disabled !== void 0 && typeof input.disabled !== "boolean") throw new TypeError("TextInput disabled must be a boolean");
}
//#endregion
//#region packages/solid-gpui/src/renderer/props.ts
var OBJECT_FIT_CODES = {
	fill: 1,
	contain: 2,
	cover: 3,
	scaleDown: 4,
	none: 5
};
function assertAccessibilityText(name, value, maxBytes) {
	if (value !== void 0 && (typeof value !== "string" || utf8ByteLength(value) > maxBytes || /[\u0000-\u001f\u007f]/.test(value))) throw new TypeError(`${name} must be a safe string of at most ${maxBytes} UTF-8 bytes`);
}
function accessibilityFor(kind, props) {
	const roleValue = Object.hasOwn(props, "accessibilityRole") ? props.accessibilityRole : void 0;
	if (roleValue !== void 0 && !Object.hasOwn(ROLE_CODES, roleValue)) throw new TypeError("accessibilityRole is invalid");
	const role = roleValue ?? defaultAccessibilityRole(kind);
	const label = Object.hasOwn(props, "accessibilityLabel") ? props.accessibilityLabel : void 0;
	const description = Object.hasOwn(props, "accessibilityDescription") ? props.accessibilityDescription : void 0;
	const disabled = Object.hasOwn(props, "accessibilityDisabled") ? props.accessibilityDisabled : void 0;
	const checked = Object.hasOwn(props, "accessibilityChecked") ? props.accessibilityChecked : void 0;
	const selected = Object.hasOwn(props, "accessibilitySelected") ? props.accessibilitySelected : void 0;
	const value = Object.hasOwn(props, "accessibilityValue") ? props.accessibilityValue : void 0;
	const expanded = Object.hasOwn(props, "accessibilityExpanded") ? props.accessibilityExpanded : void 0;
	const level = Object.hasOwn(props, "accessibilityLevel") ? props.accessibilityLevel : void 0;
	const live = props.accessibilityLive;
	const liveCodes = {
		off: 0,
		polite: 1,
		assertive: 2
	};
	if (live !== void 0 && !Object.hasOwn(liveCodes, live)) throw new TypeError("accessibilityLive is invalid");
	if (live !== void 0 && live !== "off" && (role === void 0 || role === "generic" || value === void 0)) throw new TypeError("A live region requires a semantic role and accessibilityValue");
	const inputDisabled = kind === "TextInput" && Object.hasOwn(props, "disabled") ? props.disabled : void 0;
	assertAccessibilityText("accessibilityLabel", label, 1024);
	assertAccessibilityText("accessibilityDescription", description, MAX_CLIPBOARD_TEXT_BYTES);
	assertAccessibilityText("accessibilityValue", value, MAX_CLIPBOARD_TEXT_BYTES);
	if (disabled !== void 0 && typeof disabled !== "boolean") throw new TypeError("accessibilityDisabled must be a boolean");
	if (checked !== void 0 && typeof checked !== "boolean") throw new TypeError("accessibilityChecked must be a boolean");
	if (selected !== void 0 && typeof selected !== "boolean") throw new TypeError("accessibilitySelected must be a boolean");
	if (expanded !== void 0 && typeof expanded !== "boolean") throw new TypeError("accessibilityExpanded must be a boolean");
	if (level !== void 0 && (!Number.isInteger(level) || level < 1 || level > 4294967295)) throw new TypeError("accessibilityLevel must be a positive u32");
	if (checked !== void 0 && role !== "checkbox") throw new TypeError("accessibilityChecked requires accessibilityRole=checkbox");
	if (level !== void 0 && role !== "heading") throw new TypeError("accessibilityLevel requires accessibilityRole=heading");
	if (role === void 0 && !Object.keys(props).some(isAccessibilityProp)) return null;
	return {
		role: role === void 0 ? 0 : ROLE_CODES[role],
		label: label ?? null,
		description: description ?? null,
		disabled: disabled ?? inputDisabled ?? false,
		checked: checked ?? null,
		selected: selected ?? null,
		value: value ?? null,
		expanded: expanded ?? null,
		level: level ?? null,
		live: live === void 0 ? null : liveCodes[live]
	};
}
function inputFor(node, props) {
	if (node.kind !== "TextInput") return null;
	const input = props;
	const previous = node.hostProperties;
	const previousInput = previous?.type === "text-input" ? previous.value : null;
	const maxLength = input.maxLength ?? null;
	return {
		type: "text-input",
		value: {
			value: truncateUtf16(input.value ?? node.latestNativeText ?? previousInput?.value ?? input.defaultValue ?? "", maxLength),
			placeholder: input.placeholder ?? null,
			multiline: input.multiline ?? false,
			disabled: input.disabled ?? false,
			controlled: input.value !== void 0,
			ackEditSeq: node.latestNativeEditSeq || previousInput?.ackEditSeq || 0,
			selectionStart: node.latestNativeSelection?.start ?? previousInput?.selectionStart ?? 0,
			selectionEnd: node.latestNativeSelection?.end ?? previousInput?.selectionEnd ?? 0,
			markedStart: node.latestNativeSelection?.markedStart ?? previousInput?.markedStart ?? null,
			markedEnd: node.latestNativeSelection?.markedEnd ?? previousInput?.markedEnd ?? null,
			selectionReversed: node.latestNativeSelection?.reversed ?? previousInput?.selectionReversed ?? false,
			maxLength
		}
	};
}
function assertImageSource(name, value) {
	if (typeof value !== "string" || value.length === 0 || utf8ByteLength(value) > 1048576 || /[\u0000-\u001f\u007f]/.test(value)) throw new TypeError(`Image ${name} must be a non-empty source of at most ${MAX_IMAGE_SOURCE_BYTES} UTF-8 bytes`);
}
function imageFor(node, props) {
	if (node.kind !== "Image") return null;
	const source = props.source;
	assertImageSource("source", source);
	const fallbackSource = props.fallbackSource;
	if (fallbackSource !== void 0) assertImageSource("fallbackSource", fallbackSource);
	const sourceSet = props.sourceSet ?? [];
	if (!Array.isArray(sourceSet) || sourceSet.length > 32) throw new TypeError("Image sourceSet must contain at most 32 candidates");
	let sourceBytes = utf8ByteLength(source) + (fallbackSource === void 0 ? 0 : utf8ByteLength(fallbackSource));
	const sources = sourceSet.map((candidate, index) => {
		if (candidate === null || typeof candidate !== "object") throw new TypeError(`Image sourceSet[${index}] must be a source candidate`);
		assertImageSource(`sourceSet[${index}].source`, candidate.source);
		for (const dimension of ["width", "height"]) {
			const value = candidate[dimension];
			if (!Number.isInteger(value) || value <= 0 || value > 4294967295) throw new TypeError(`Image sourceSet[${index}].${dimension} must be a positive uint32`);
		}
		sourceBytes += utf8ByteLength(candidate.source);
		return {
			source: candidate.source,
			width: candidate.width,
			height: candidate.height
		};
	});
	if (sourceBytes > 16776192) throw new RangeError("Image sources exceed the aggregate frame byte limit");
	const objectFit = props.objectFit ?? "contain";
	if (![
		"fill",
		"contain",
		"cover",
		"scaleDown",
		"none"
	].includes(objectFit)) throw new TypeError("Image objectFit is invalid");
	return {
		type: "image",
		value: {
			source,
			objectFit: OBJECT_FIT_CODES[objectFit],
			fallbackSource: fallbackSource ?? null,
			sources
		}
	};
}
function iconFor(node, props) {
	if (node.kind !== "Icon") return null;
	const name = props.name;
	if (!isIconName(name)) throw new TypeError(`Unknown Icon name ${typeof name === "string" ? JSON.stringify(name) : `(${typeof name})`}. Use ICON_NAMES from @solid-gpui/core for built-in names, or register the SVG with solid_gpui::icons::register_icons and read its name from the generated applicationIcons catalog (for example applicationIcons["prefix:name"]). Iconify names are not downloaded at runtime.`);
	const size = props.size ?? 16;
	if (typeof size !== "number" || !Number.isFinite(size) || size <= 0 || !Number.isFinite(Math.fround(size))) throw new RangeError("Icon size must be a positive float32");
	const color = props.color;
	if (color !== void 0 && typeof color !== "string") throw new TypeError("Icon color must be a CSS color");
	return {
		type: "icon",
		value: {
			name,
			size: Math.fround(size),
			color: color === void 0 ? null : encodeColor(color)
		}
	};
}
function committedVirtualListData(previous) {
	if (previous === null) return void 0;
	return previous.committedData;
}
/**
* Compute the replacement span between the previously published data and the
* next published data. Positions outside the span hold identical items (by
* reference), so the native list can retain their measured heights; rows inside
* the span must be treated as replaced.
*/
function diffDataSpan(previous, current) {
	const previousLength = previous.length;
	const currentLength = current.length;
	const limit = Math.min(previousLength, currentLength);
	let start = 0;
	while (start < limit && Object.is(previous[start], current[start])) start += 1;
	let suffix = 0;
	while (suffix < limit - start && Object.is(previous[previousLength - 1 - suffix], current[currentLength - 1 - suffix])) suffix += 1;
	return {
		start,
		oldCount: previousLength - start - suffix,
		newCount: currentLength - start - suffix
	};
}
function virtualListFor(node, props) {
	if (node.kind !== "VirtualList") return null;
	const itemCount = props.__itemCount;
	const rangeStart = props.__rangeStart;
	const rangeEnd = props.__rangeEnd;
	const estimatedItemSize = props.__estimatedItemSize;
	const overscan = props.__overscan ?? 0;
	if (itemCount === void 0 || rangeStart === void 0 || rangeEnd === void 0 || estimatedItemSize === void 0) throw new TypeError("VirtualList host properties are incomplete");
	if (![
		itemCount,
		rangeStart,
		rangeEnd,
		overscan
	].every(Number.isInteger) || itemCount < 0 || rangeStart < 0 || rangeEnd < rangeStart || rangeEnd > itemCount || overscan < 0 || estimatedItemSize <= 0 || !Number.isFinite(estimatedItemSize) || !Number.isFinite(Math.fround(estimatedItemSize))) throw new RangeError("VirtualList host properties are invalid");
	assertU32Option("VirtualList itemCount", itemCount);
	assertU32Option("VirtualList rangeStart", rangeStart);
	assertU32Option("VirtualList rangeEnd", rangeEnd);
	assertU32Option("VirtualList overscan", overscan);
	const data = props.__data;
	if (!Array.isArray(data)) throw new TypeError("VirtualList requires a data array");
	if (data.length !== itemCount) throw new RangeError("VirtualList itemCount must equal the data length");
	const previous = node.hostProperties !== null && node.hostProperties.type === "virtual-list" ? node.hostProperties.value : null;
	let dataRevision = previous?.dataRevision ?? 0;
	let dataEdit = previous?.dataEdit ?? null;
	const committed = committedVirtualListData(previous);
	if (committed === void 0 || previous === null) {
		dataRevision = 0;
		dataEdit = null;
	} else {
		const span = committed === data ? null : diffDataSpan(committed, data);
		if (span === null || span.oldCount === 0 && span.newCount === 0) {
			dataRevision = previous.dataRevision;
			dataEdit = previous.dataEdit;
		} else {
			dataRevision = nextU32(previous.dataRevision, "VirtualList dataRevision");
			dataEdit = {
				baseRevision: previous.dataRevision,
				...span
			};
		}
	}
	const value = {
		itemCount,
		rangeStart,
		rangeEnd,
		estimatedItemSize,
		overscan,
		dataRevision,
		dataEdit
	};
	value.committedData = data;
	return {
		type: "virtual-list",
		value
	};
}
function exportFilesFor(value) {
	if (value === void 0) return null;
	if (!Array.isArray(value) || value.length === 0 || value.length > 8) throw new RangeError("draggable.exportFiles must contain 1..8 paths");
	for (const [index, path] of value.entries()) if (typeof path !== "string" || path.length === 0 || utf8ByteLength(path) > 1024 || /[\u0000-\u001f\u007f]/.test(path)) throw new TypeError(`draggable.exportFiles[${index}] must be a non-empty path of at most 1024 UTF-8 bytes`);
	return value;
}
function dragFor(node, props) {
	if (node.kind === "Pressable" && props.disabled === true) return null;
	if (props.draggable === void 0 && props.onDragOver === void 0 && props.onDrop === void 0 && props.onExternalFileDrop === void 0) return null;
	return {
		type: "drag",
		value: {
			dragType: props.draggable?.type ?? null,
			exportFiles: exportFilesFor(props.draggable?.exportFiles),
			acceptsDragOver: props.onDragOver !== void 0,
			acceptsDrop: props.onDrop !== void 0
		}
	};
}
function nextU32(value, name) {
	if (value >= 4294967295) throw new RangeError(`${name} exhausted u32 range`);
	return value + 1;
}
function assertU32Option(name, value) {
	if (!Number.isInteger(value) || value < 0 || value > 4294967295) throw new RangeError(`${name} must be a u32`);
	return value;
}
function truncateUtf16(value, maxLength) {
	if (maxLength === null) return value;
	let units = 0;
	let end = 0;
	for (const character of value) {
		const nextUnits = units + character.length;
		if (nextUnits > maxLength) break;
		units = nextUnits;
		end += character.length;
	}
	return end === value.length ? value : value.slice(0, end);
}
function validateProps(kind, props) {
	validateHostKindValues(kind, props);
	if (kind === "Extension") {
		if (props.__extensionDescriptor === void 0) throw new TypeError("Extension descriptor is required");
		if (typeof props.__extensionDescriptor !== "object") throw new TypeError("Extension descriptor is invalid");
		if (props.onExtensionEvent !== void 0 && typeof props.onExtensionEvent !== "function") throw new TypeError("Extension onExtensionEvent must be a function");
		if (props.onLayout !== void 0 && typeof props.onLayout !== "function") throw new TypeError("Extension onLayout must be a function");
		if (props.style !== void 0) validateStyle(props.style);
		accessibilityFor(kind, props);
		return;
	}
	for (const key of Object.keys(props)) if (!hasAllowedProp(kind, key)) throw new TypeError(`Unsupported ${kind} prop: ${key}`);
	if ((kind === "View" || kind === "Text" || kind === "Pressable" || kind === "Image") && props.onLayout !== void 0 && typeof props.onLayout !== "function") throw new TypeError(`${kind} onLayout must be a function`);
	if (kind !== "RawText") validateStyle(props.style);
	if (kind === "Text" && props.onPress !== void 0 && typeof props.onPress !== "function") throw new TypeError("Text onPress must be a function");
	if (kind === "Text" && props.selectable !== void 0 && typeof props.selectable !== "boolean") throw new TypeError("Text selectable must be a boolean");
	if (kind === "Text") {
		if (props.onFocus !== void 0 && typeof props.onFocus !== "function") throw new TypeError("Text onFocus must be a function");
		if (props.onBlur !== void 0 && typeof props.onBlur !== "function") throw new TypeError("Text onBlur must be a function");
		if (props.onFocus !== void 0 || props.onBlur !== void 0) {
			if (props.onPress === void 0) throw new TypeError("Text onFocus/onBlur requires onPress");
		}
	}
	if ((kind === "View" || kind === "Pressable") && props.tooltip !== void 0) {
		if (typeof props.tooltip !== "string" || props.tooltip.length === 0 || utf8ByteLength(props.tooltip) > 256 || /[\u0000-\u001f\u007f-\u009f]/.test(props.tooltip)) throw new TypeError(`${kind} tooltip must be a non-empty safe string of at most 256 UTF-8 bytes`);
	}
	if (kind === "View" || kind === "Pressable") {
		if (props.onPointerDown !== void 0 && typeof props.onPointerDown !== "function") throw new TypeError(`${kind} onPointerDown must be a function`);
		if (props.onPointerMove !== void 0 && typeof props.onPointerMove !== "function") throw new TypeError(`${kind} onPointerMove must be a function`);
		if (props.onPointerUp !== void 0 && typeof props.onPointerUp !== "function") throw new TypeError(`${kind} onPointerUp must be a function`);
		if (props.onHoverChange !== void 0 && typeof props.onHoverChange !== "function") throw new TypeError(`${kind} onHoverChange must be a function`);
		if (props.onFocus !== void 0 && typeof props.onFocus !== "function") throw new TypeError(`${kind} onFocus must be a function`);
		if (props.onBlur !== void 0 && typeof props.onBlur !== "function") throw new TypeError(`${kind} onBlur must be a function`);
		if ((props.onFocus !== void 0 || props.onBlur !== void 0) && props.focusable !== true) throw new TypeError(`${kind} onFocus/onBlur requires focusable=true`);
		if (kind === "View" && props.onPointerDownOutside !== void 0 && typeof props.onPointerDownOutside !== "function") throw new TypeError(`${kind} onPointerDownOutside must be a function`);
		if (props.draggable !== void 0) {
			const draggable = props.draggable;
			if (draggable === null || typeof draggable !== "object" || typeof draggable.type !== "string" || draggable.type.length === 0 || [...draggable.type].length > 128 || /[\u0000-\u001f\u007f]/.test(draggable.type)) throw new TypeError(`${kind} draggable.type must be a non-empty safe string`);
		}
		if (props.onDragOver !== void 0 && typeof props.onDragOver !== "function") throw new TypeError(`${kind} onDragOver must be a function`);
		if (props.onDrop !== void 0 && typeof props.onDrop !== "function") throw new TypeError(`${kind} onDrop must be a function`);
		if (props.onExternalFileDrop !== void 0 && typeof props.onExternalFileDrop !== "function") throw new TypeError(`${kind} onExternalFileDrop must be a function`);
		if (props.focusable !== void 0 && typeof props.focusable !== "boolean") throw new TypeError("Pressable focusable must be a boolean");
		if (props.onKeyDown !== void 0 && typeof props.onKeyDown !== "function") throw new TypeError("Pressable onKeyDown must be a function");
		if (props.onKeyDown !== void 0 && props.focusable !== true) throw new TypeError("Pressable onKeyDown requires focusable=true");
		if (props.disabled !== void 0 && typeof props.disabled !== "boolean") throw new TypeError("Pressable disabled must be a boolean");
	}
	if (kind === "Image") imageFor({
		kind,
		hostProperties: null
	}, props);
	if (kind === "Icon") iconFor({
		kind,
		hostProperties: null
	}, props);
	if (kind === "TextInput") {
		const input = props;
		for (const [name, value] of [
			["value", input.value],
			["defaultValue", input.defaultValue],
			["placeholder", input.placeholder]
		]) if (value !== void 0 && (typeof value !== "string" || utf8ByteLength(value) > 1048576)) throw new TypeError(`TextInput ${name} must be a string of at most ${MAX_CLIPBOARD_TEXT_BYTES} UTF-8 bytes`);
		if (input.maxLength !== void 0 && (!Number.isInteger(input.maxLength) || input.maxLength < 0 || input.maxLength > 4294967295)) throw new TypeError("TextInput maxLength must be a non-negative u32");
	}
	if (kind === "View") {
		if (props.focusable !== void 0 && typeof props.focusable !== "boolean") throw new TypeError("View focusable must be a boolean");
		if (props.onKeyDown !== void 0 && typeof props.onKeyDown !== "function") throw new TypeError("View onKeyDown must be a function");
		if (props.onKeyDown !== void 0 && props.focusable !== true) throw new TypeError("View onKeyDown requires focusable=true");
		if (props.onScroll !== void 0 && typeof props.onScroll !== "function") throw new TypeError("View onScroll must be a function");
	}
	if (kind === "VirtualList") {
		virtualListFor({
			kind,
			hostProperties: null
		}, props);
		if (props.__onVisibleRange !== void 0 && typeof props.__onVisibleRange !== "function") throw new TypeError("VirtualList range callback must be a function");
	}
	accessibilityFor(kind, props);
}
//#endregion
//#region packages/solid-gpui/src/renderer/extension.ts
var EXTENSION_ENCODER = new TextEncoder();
function assertExtensionBytes(value, length, name) {
	if (!(value instanceof Uint8Array) || value.byteLength !== length) throw new RangeError(`${name} must be exactly ${length} bytes`);
}
function assertExtensionId(value, name) {
	if (!Number.isInteger(value) || value <= 0 || value > 4294967295) throw new RangeError(`${name} must be a non-zero u32`);
}
function assertExtensionU32(value, name) {
	if (!Number.isInteger(value) || value < 0 || value > 4294967295) throw new RangeError(`${name} must be a u32`);
}
function assertExtensionValue(value, name) {
	if (value === null || typeof value !== "object" || Array.isArray(value)) throw new TypeError(`${name} is invalid`);
	switch (value.type) {
		case "bool":
			if (typeof value.value !== "boolean") throw new TypeError(`${name} must be a bool`);
			return;
		case "i32":
			if (!Number.isInteger(value.value) || value.value < -2147483648 || value.value > 2147483647) throw new TypeError(`${name} must be an i32`);
			return;
		case "u32":
			assertExtensionU32(value.value, `${name} value`);
			return;
		case "f32":
			if (!Number.isFinite(value.value) || !Number.isFinite(Math.fround(value.value))) throw new TypeError(`${name} must be a finite f32`);
			return;
		case "text":
			if (typeof value.value !== "string") throw new TypeError(`${name} must be text`);
			if (EXTENSION_ENCODER.encode(value.value).byteLength > 1048576) throw new RangeError(`${name} exceeds its byte limit`);
			return;
		case "bytes":
			if (!(value.value instanceof Uint8Array)) throw new TypeError(`${name} must be bytes`);
			if (value.value.byteLength > 1048576) throw new RangeError(`${name} exceeds its byte limit`);
			return;
		default: throw new TypeError(`${name} has an unknown type`);
	}
}
function normalizeExtensionProperties(descriptor, props, eventIds = descriptor.eventIds) {
	if (descriptor === null || typeof descriptor !== "object") throw new TypeError("Extension descriptor is invalid");
	assertExtensionBytes(descriptor.providerId, 16, "providerId");
	assertExtensionBytes(descriptor.catalogDigest, 32, "catalogDigest");
	assertExtensionId(descriptor.entryId, "entryId");
	assertExtensionId(descriptor.entryVersion, "entryVersion");
	if (typeof descriptor.encodeProps !== "function") throw new TypeError("Extension encodeProps must be a function");
	if (!Array.isArray(eventIds) || eventIds.length > 256) throw new RangeError("eventIds must contain at most 256 IDs");
	assertSortedIds(eventIds, "eventIds");
	const fields = descriptor.encodeProps(props);
	assertSortedExtensionFields(fields);
	return {
		providerId: descriptor.providerId.slice(),
		catalogDigest: descriptor.catalogDigest.slice(),
		entryId: descriptor.entryId,
		entryVersion: descriptor.entryVersion,
		fields: fields.map((field) => ({
			id: field.id,
			value: cloneExtensionValue(field.value)
		})),
		eventIds: [...eventIds]
	};
}
function cloneExtensionValue(value) {
	return value.type === "bytes" ? {
		type: "bytes",
		value: value.value.slice()
	} : value;
}
function assertSortedIds(values, name) {
	if (!Array.isArray(values) || values.length > 256) throw new RangeError(`${name} is too large`);
	let previous = -1;
	for (const value of values) {
		assertExtensionId(value, `${name} entry`);
		if (value <= previous) throw new RangeError(`${name} must be sorted and unique`);
		previous = value;
	}
}
function assertSortedExtensionFields(fields) {
	if (!Array.isArray(fields) || fields.length > 256) throw new RangeError("extension fields must contain at most 256 fields");
	let previous = -1;
	let textBytes = 0;
	let bytes = 0;
	const encoder = EXTENSION_ENCODER;
	for (const field of fields) {
		if (field === null || typeof field !== "object") throw new TypeError("extension field is invalid");
		assertExtensionId(field.id, "extension field id");
		if (field.id <= previous) throw new RangeError("extension fields must be sorted and unique");
		assertExtensionValue(field.value, `extension field ${field.id}`);
		if (field.value.type === "text") textBytes += encoder.encode(field.value.value).byteLength;
		if (field.value.type === "bytes") bytes += field.value.value.byteLength;
		if (textBytes > 1048576 || bytes > 1048576) throw new RangeError("extension field payload exceeds its aggregate byte limit");
		previous = field.id;
	}
}
//#endregion
//#region packages/solid-gpui/src/renderer/nodes.ts
var ListenerRegistry = class {
	slots = /* @__PURE__ */ new Map();
	byId = /* @__PURE__ */ new Map();
	pendingNodes = /* @__PURE__ */ new Set();
	publishedRevision = 0;
	bind(node, listenerId, hasListener) {
		const slot = this.slots.get(node.id) ?? {
			current: null,
			previous: null
		};
		const next = hasListener ? this.binding(node, listenerId) : null;
		if (next !== null && slot.current !== null && this.sameCallbacks(slot.current, next)) return;
		if (slot.current !== null && slot.current.revision <= this.publishedRevision) slot.previous = slot.current;
		slot.current = next;
		this.setSlot(node.id, slot);
	}
	remove(nodeId) {
		this.setSlot(nodeId, {
			current: null,
			previous: null
		});
	}
	clear() {
		this.slots.clear();
		this.byId.clear();
		this.pendingNodes.clear();
	}
	current(nodeId) {
		return this.slots.get(nodeId)?.current ?? null;
	}
	snapshotNode(nodeId) {
		const slot = this.slots.get(nodeId);
		if (slot === void 0) return void 0;
		return {
			current: slot.current === null ? null : this.cloneBinding(slot.current),
			previous: slot.previous === null ? null : this.cloneBinding(slot.previous)
		};
	}
	restoreNode(nodeId, snapshot) {
		if (snapshot === void 0) {
			this.setSlot(nodeId, {
				current: null,
				previous: null
			});
			return;
		}
		this.setSlot(nodeId, {
			current: snapshot.current === null ? null : this.cloneBinding(snapshot.current),
			previous: snapshot.previous === null ? null : this.cloneBinding(snapshot.previous)
		});
	}
	lookup(listenerId, revision) {
		const entry = this.byId.get(listenerId);
		if (entry === void 0) return void 0;
		const current = entry.current;
		let match = current !== null && current.revision <= revision ? current : void 0;
		const previous = entry.previous;
		if (previous !== null && previous.revision <= revision && (match === void 0 || previous.revision > match.revision)) match = previous;
		return match;
	}
	publishRevision(revision) {
		this.publishedRevision = revision;
		for (const nodeId of this.pendingNodes) {
			const current = this.slots.get(nodeId)?.current;
			if (current?.pending) {
				current.revision = revision;
				current.pending = false;
			}
		}
		this.pendingNodes.clear();
	}
	setSlot(nodeId, next) {
		const previous = this.slots.get(nodeId);
		if (previous !== void 0) {
			this.removeById(previous.current);
			this.removeById(previous.previous);
		}
		if (next.current === null && next.previous === null) {
			this.slots.delete(nodeId);
			this.pendingNodes.delete(nodeId);
			return;
		}
		this.slots.set(nodeId, next);
		this.addById(next.current, true);
		this.addById(next.previous, false);
		if (next.current?.pending) this.pendingNodes.add(nodeId);
		else this.pendingNodes.delete(nodeId);
	}
	addById(binding, current) {
		if (binding === null) return;
		const entry = this.byId.get(binding.listenerId) ?? {
			current: null,
			previous: null
		};
		if (current) entry.current = binding;
		else entry.previous = binding;
		this.byId.set(binding.listenerId, entry);
	}
	removeById(binding) {
		if (binding === null) return;
		const entry = this.byId.get(binding.listenerId);
		if (entry === void 0) return;
		if (entry.current === binding) entry.current = null;
		if (entry.previous === binding) entry.previous = null;
		if (entry.current === null && entry.previous === null) this.byId.delete(binding.listenerId);
	}
	binding(node, listenerId) {
		return {
			node,
			listenerId,
			revision: this.publishedRevision + 1,
			pending: true,
			listener: node.listener,
			keyListener: node.keyListener,
			pointerCallbacks: node.pointerCallbacks === null ? null : { ...node.pointerCallbacks },
			pointerMoveCallback: node.pointerMoveCallback,
			focusCallback: node.focusCallback,
			blurCallback: node.blurCallback,
			pointerDownOutsideCallback: node.pointerDownOutsideCallback,
			hoverCallback: node.hoverCallback,
			scrollCallback: node.scrollCallback,
			dragCallbacks: { ...node.dragCallbacks },
			layoutCallback: node.layoutCallback,
			inputCallbacks: node.inputCallbacks,
			visibleRangeCallback: node.visibleRangeCallback,
			animationCompleteCallback: node.animationCompleteCallback,
			extensionEventCallback: node.extensionEventCallback,
			extensionEventIds: [...node.extensionEventIds]
		};
	}
	cloneBinding(binding) {
		return {
			...binding,
			pointerCallbacks: binding.pointerCallbacks === null ? null : { ...binding.pointerCallbacks },
			dragCallbacks: { ...binding.dragCallbacks }
		};
	}
	sameCallbacks(a, b) {
		return a.listenerId === b.listenerId && a.listener === b.listener && a.keyListener === b.keyListener && a.pointerCallbacks?.down === b.pointerCallbacks?.down && a.pointerCallbacks?.up === b.pointerCallbacks?.up && a.pointerMoveCallback === b.pointerMoveCallback && a.focusCallback === b.focusCallback && a.blurCallback === b.blurCallback && a.pointerDownOutsideCallback === b.pointerDownOutsideCallback && a.hoverCallback === b.hoverCallback && a.scrollCallback === b.scrollCallback && a.dragCallbacks.over === b.dragCallbacks.over && a.dragCallbacks.drop === b.dragCallbacks.drop && a.dragCallbacks.externalFileDrop === b.dragCallbacks.externalFileDrop && a.layoutCallback === b.layoutCallback && a.inputCallbacks === b.inputCallbacks && a.visibleRangeCallback === b.visibleRangeCallback && a.animationCompleteCallback === b.animationCompleteCallback && a.extensionEventCallback === b.extensionEventCallback && a.extensionEventIds.length === b.extensionEventIds.length && a.extensionEventIds.every((id, index) => id === b.extensionEventIds[index]);
	}
};
function equalTransition(a, b) {
	if (a === b) return true;
	if (a === void 0 || b === void 0) return false;
	if (a.durationMs !== b.durationMs || a.delayMs !== b.delayMs || a.easing !== b.easing || a.onComplete !== b.onComplete || a.properties?.length !== b.properties?.length) return false;
	if (a.properties === void 0 || b.properties === void 0) return true;
	for (let index = 0; index < a.properties.length; index += 1) if (a.properties[index] !== b.properties[index]) return false;
	return true;
}
function equalBoxShadow(a, b) {
	if (a === b) return true;
	if (a === void 0 || b === void 0 || Array.isArray(a) !== Array.isArray(b)) return false;
	const left = Array.isArray(a) ? a : [a];
	const right = Array.isArray(b) ? b : [b];
	if (left.length !== right.length) return false;
	for (let index = 0; index < left.length; index += 1) {
		const x = left[index];
		const y = right[index];
		if (x.offsetX !== y.offsetX || x.offsetY !== y.offsetY || x.blurRadius !== y.blurRadius || x.spreadRadius !== y.spreadRadius || x.color !== y.color || x.inset !== y.inset) return false;
	}
	return true;
}
function equalStyle(a, b) {
	if (a === b) return true;
	if (a == null || b == null) return false;
	return a.width === b.width && a.height === b.height && a.gridColumns === b.gridColumns && a.gridRows === b.gridRows && a.gridColumnSpan === b.gridColumnSpan && a.gridRowSpan === b.gridRowSpan && a.flexDirection === b.flexDirection && a.flexGrow === b.flexGrow && a.padding === b.padding && a.gap === b.gap && a.justifyContent === b.justifyContent && a.alignItems === b.alignItems && a.borderRadius === b.borderRadius && a.borderWidth === b.borderWidth && a.borderColor === b.borderColor && a.fontSize === b.fontSize && a.fontWeight === b.fontWeight && a.overflow === b.overflow && a.lineClamp === b.lineClamp && a.textOverflow === b.textOverflow && a.marginTop === b.marginTop && a.marginRight === b.marginRight && a.marginBottom === b.marginBottom && a.marginLeft === b.marginLeft && a.fontStyle === b.fontStyle && a.textDecoration === b.textDecoration && a.lineHeight === b.lineHeight && a.minWidth === b.minWidth && a.maxWidth === b.maxWidth && a.minHeight === b.minHeight && a.maxHeight === b.maxHeight && a.flexShrink === b.flexShrink && a.alignSelf === b.alignSelf && a.position === b.position && a.left === b.left && a.top === b.top && a.right === b.right && a.bottom === b.bottom && a.cursor === b.cursor && a.textAlign === b.textAlign && a.backgroundColor === b.backgroundColor && a.color === b.color && a.opacity === b.opacity && a.fontFamily === b.fontFamily && equalTransition(a.transition, b.transition) && equalBoxShadow(a.boxShadow, b.boxShadow);
}
function equalHostProperties(a, b) {
	if (a === b) return true;
	if (a === null || b === null || a.type !== b.type) return false;
	if (a.type === "text-input" && b.type === "text-input") {
		const x = a.value;
		const y = b.value;
		return x.value === y.value && x.placeholder === y.placeholder && x.multiline === y.multiline && x.disabled === y.disabled && x.controlled === y.controlled && x.ackEditSeq === y.ackEditSeq && x.selectionStart === y.selectionStart && x.selectionEnd === y.selectionEnd && x.markedStart === y.markedStart && x.markedEnd === y.markedEnd && x.maxLength === y.maxLength && x.selectionReversed === y.selectionReversed;
	}
	if (a.type === "virtual-list" && b.type === "virtual-list") {
		const x = a.value;
		const y = b.value;
		if (x.itemCount !== y.itemCount || x.rangeStart !== y.rangeStart || x.rangeEnd !== y.rangeEnd || x.estimatedItemSize !== y.estimatedItemSize || x.overscan !== y.overscan || x.dataRevision !== y.dataRevision) return false;
		if (x.dataEdit === null || y.dataEdit === null) return x.dataEdit === y.dataEdit;
		return x.dataEdit.baseRevision === y.dataEdit.baseRevision && x.dataEdit.start === y.dataEdit.start && x.dataEdit.oldCount === y.dataEdit.oldCount && x.dataEdit.newCount === y.dataEdit.newCount;
	}
	if (a.type === "image" && b.type === "image") {
		const x = a.value;
		const y = b.value;
		return x.source === y.source && x.objectFit === y.objectFit && x.fallbackSource === y.fallbackSource && x.sources.length === y.sources.length && x.sources.every((candidate, index) => candidate.source === y.sources[index]?.source && candidate.width === y.sources[index]?.width && candidate.height === y.sources[index]?.height);
	}
	if (a.type === "drag" && b.type === "drag") {
		if (a.value.dragType !== b.value.dragType || a.value.acceptsDragOver !== b.value.acceptsDragOver || a.value.acceptsDrop !== b.value.acceptsDrop || a.value.exportFiles?.length !== b.value.exportFiles?.length) return false;
		if (a.value.exportFiles === null || b.value.exportFiles === null) return true;
		return a.value.exportFiles.every((path, index) => path === b.value.exportFiles?.[index]);
	}
	if (a.type === "extension" && b.type === "extension") {
		const x = a.value;
		const y = b.value;
		if (x.entryId !== y.entryId || x.entryVersion !== y.entryVersion || !sameBytes(x.providerId, y.providerId) || !sameBytes(x.catalogDigest, y.catalogDigest) || x.fields.length !== y.fields.length || x.eventIds.length !== y.eventIds.length) return false;
		return x.fields.every((field, index) => field.id === y.fields[index]?.id && equalExtensionValue(field.value, y.fields[index]?.value)) && x.eventIds.every((id, index) => id === y.eventIds[index]);
	}
	if (a.type === "icon" && b.type === "icon") return a.value.name === b.value.name && a.value.size === b.value.size && a.value.color === b.value.color;
	return false;
}
function sameBytes(a, b) {
	return a.byteLength === b.byteLength && a.every((byte, index) => byte === b[index]);
}
function equalExtensionValue(a, b) {
	if (b === void 0 || a.type !== b.type) return false;
	if (a.type === "bytes" && b.type === "bytes") return sameBytes(a.value, b.value);
	return a.value === b.value;
}
function equalAccessibility(a, b) {
	if (a === b) return true;
	if (a === null || b === null) return false;
	return a.role === b.role && a.label === b.label && a.description === b.description && a.disabled === b.disabled && a.checked === b.checked && a.selected === b.selected && a.value === b.value && a.expanded === b.expanded && a.level === b.level && a.live === b.live;
}
var NodeGraph = class {
	owner;
	children;
	syntheticRoot;
	nodesById = /* @__PURE__ */ new Map();
	listenerRegistry = new ListenerRegistry();
	createdIds = /* @__PURE__ */ new Set();
	deletedRoots = /* @__PURE__ */ new Set();
	movedIds = /* @__PURE__ */ new Set();
	updatedMasks = /* @__PURE__ */ new Map();
	dirtyProps = /* @__PURE__ */ new Map();
	nextNodeId = 2;
	nextListenerId = 1;
	transaction;
	/** Intrusive head of parents whose materialized child order lags their links. */
	dirtyOrderHead = null;
	constructor(owner) {
		this.owner = owner;
		this.syntheticRoot = {
			id: 1,
			kind: "View",
			root: owner,
			parent: null,
			children: [],
			index: 0,
			firstChild: null,
			lastChild: null,
			previousSibling: null,
			nextSibling: null,
			childOrderDirty: false,
			dirtyNext: null,
			style: null,
			text: null,
			listenerId: 0,
			listener: void 0,
			dragCallbacks: {},
			layoutCallback: void 0,
			observesLayout: false,
			focusable: false,
			selectable: false,
			disabled: false,
			keyListener: void 0,
			pointerCallbacks: null,
			pointerMoveCallback: void 0,
			focusCallback: void 0,
			blurCallback: void 0,
			detachedFocusPending: false,
			nativeFocused: false,
			pointerDownOutsideCallback: void 0,
			scrollCallback: void 0,
			hovered: false,
			hoverCallback: void 0,
			hostProperties: null,
			tooltip: null,
			acceptsPointerMove: false,
			latestNativeText: null,
			latestNativeEditSeq: 0,
			latestNativeSelection: null,
			inputCallbacks: null,
			inputCallbackSources: null,
			extensionEventCallback: void 0,
			extensionDescriptor: void 0,
			extensionEventIds: [],
			lastAnimationGeneration: null,
			accessibility: null,
			attached: true,
			props: {},
			mounted: true
		};
		this.nodesById.set(1, this.syntheticRoot);
		this.children = this.syntheticRoot.children;
	}
	allocateNode(kind) {
		const node = {
			id: this.nextNodeId,
			kind,
			root: this.owner,
			parent: null,
			children: [],
			index: 0,
			firstChild: null,
			lastChild: null,
			previousSibling: null,
			nextSibling: null,
			childOrderDirty: false,
			dirtyNext: null,
			style: null,
			text: null,
			listenerId: 0,
			listener: void 0,
			dragCallbacks: {},
			layoutCallback: void 0,
			observesLayout: false,
			focusable: false,
			selectable: false,
			disabled: false,
			keyListener: void 0,
			pointerCallbacks: null,
			pointerMoveCallback: void 0,
			focusCallback: void 0,
			blurCallback: void 0,
			detachedFocusPending: false,
			nativeFocused: false,
			pointerDownOutsideCallback: void 0,
			hoverCallback: void 0,
			scrollCallback: void 0,
			hovered: false,
			hostProperties: null,
			tooltip: null,
			acceptsPointerMove: false,
			latestNativeText: null,
			latestNativeEditSeq: 0,
			latestNativeSelection: null,
			inputCallbacks: null,
			inputCallbackSources: null,
			extensionDescriptor: void 0,
			extensionEventIds: [],
			lastAnimationGeneration: null,
			accessibility: null,
			attached: false,
			props: {},
			mounted: false
		};
		this.nextNodeId = nextU32(this.nextNodeId, "node id");
		this.transaction?.newNodes.add(node);
		if (kind === "TextInput") {
			node.focus = () => this.owner.submitCommand(node, COMMAND_FOCUS, null);
			node.blur = () => this.owner.submitCommand(node, COMMAND_BLUR, null);
			node.setSelection = (start, end) => this.owner.submitCommand(node, COMMAND_SET_SELECTION, {
				type: "selection",
				start,
				end
			});
		}
		if (kind === "View") {
			node.focus = () => this.owner.submitCommand(node, COMMAND_FOCUS, null);
			node.isFocused = () => this.owner.submitCommandValue(node, COMMAND_GET_FOCUS, null).then((value) => {
				if (value === null || value.type !== "boolean") throw new Error(`native getFocus returned an invalid value for View node ${node.id}; update the host binary and renderer package together`);
				return value.value;
			});
			node.blur = () => this.owner.submitCommand(node, COMMAND_BLUR, null);
		}
		if (kind === "VirtualList") {
			node.scrollToIndex = (index) => this.owner.submitCommand(node, COMMAND_SCROLL_TO_INDEX, {
				type: "scroll-index",
				index,
				alignment: 0
			});
			node.scrollToEnd = () => this.owner.submitCommand(node, COMMAND_SCROLL_TO_END, null);
			node.getScrollOffset = () => this.owner.submitCommandValue(node, COMMAND_GET_SCROLL_OFFSET, null).then((value) => {
				if (value === null || value.type !== "scroll-offset") throw new Error(`native getScrollOffset returned an invalid value for VirtualList node ${node.id}; update the host binary and renderer package together`);
				return value.value;
			});
			node.scrollToOffset = (offset) => {
				if (typeof offset !== "number" || !Number.isFinite(offset) || offset < 0) return Promise.reject(/* @__PURE__ */ new TypeError(`scroll offset is invalid for VirtualList node ${node.id}; provide a finite non-negative number`));
				return this.owner.submitCommand(node, COMMAND_SCROLL_TO_OFFSET, {
					type: "number",
					value: offset
				});
			};
		}
		return node;
	}
	allocateListener() {
		const listenerId = this.nextListenerId;
		this.nextListenerId = nextU32(this.nextListenerId, "listener id");
		return listenerId;
	}
	journalNode(node) {
		const transaction = this.transaction;
		if (transaction !== void 0 && !transaction.nodeStates.has(node)) transaction.nodeStates.set(node, this.captureNode(node));
	}
	recordNodeMutation(node) {
		this.journalNode(node);
	}
	markPropsDirty(node, groups) {
		this.journalNode(node);
		const transaction = this.transaction;
		if (transaction !== void 0 && !transaction.dirtyProps.has(node)) transaction.dirtyProps.set(node, this.dirtyProps.get(node));
		this.dirtyProps.set(node, (this.dirtyProps.get(node) ?? 0) | groups);
	}
	finalizeDirtyProps() {
		for (const [node] of this.dirtyProps) {
			const mask = this.updateNodeProps(node, node.props);
			if (node.attached) this.markUpdated(node, mask, true);
		}
		this.dirtyProps.clear();
		for (const id of this.createdIds) {
			const node = this.nodesById.get(id);
			const properties = node?.hostProperties;
			if (node === void 0 || properties?.type !== "virtual-list") continue;
			if (properties.value.dataRevision === 0 && properties.value.dataEdit === null) continue;
			this.journalNode(node);
			node.hostProperties = {
				type: "virtual-list",
				value: {
					...properties.value,
					dataRevision: 0,
					dataEdit: null
				}
			};
		}
	}
	journalMap(changes, map, key) {
		if (changes !== void 0 && !changes.has(key)) changes.set(key, map.get(key));
	}
	journalSet(changes, set, key) {
		if (changes !== void 0 && !changes.has(key)) changes.set(key, set.has(key) ? true : void 0);
	}
	journalRegistry(nodeId) {
		if (this.transaction !== void 0 && !this.transaction.registry.has(nodeId)) this.transaction.registry.set(nodeId, this.listenerRegistry.snapshotNode(nodeId));
	}
	findListener(listenerId, revision) {
		return this.listenerRegistry.lookup(listenerId, revision);
	}
	publishRevision(revision) {
		this.listenerRegistry.publishRevision(revision);
	}
	beginTransaction() {
		if (this.transaction !== void 0) return;
		this.transaction = {
			nodeStates: /* @__PURE__ */ new Map(),
			newNodes: /* @__PURE__ */ new Set(),
			dirtyProps: /* @__PURE__ */ new Map(),
			nodesById: /* @__PURE__ */ new Map(),
			createdIds: /* @__PURE__ */ new Map(),
			deletedRoots: /* @__PURE__ */ new Map(),
			movedIds: /* @__PURE__ */ new Map(),
			updatedMasks: /* @__PURE__ */ new Map(),
			registry: /* @__PURE__ */ new Map(),
			dirtyOrderHead: this.dirtyOrderHead,
			nextNodeId: this.nextNodeId,
			nextListenerId: this.nextListenerId
		};
	}
	dispose() {
		this.transaction = void 0;
		this.dirtyProps.clear();
		this.nodesById.clear();
		this.createdIds.clear();
		this.deletedRoots.clear();
		this.movedIds.clear();
		this.updatedMasks.clear();
		this.listenerRegistry.clear();
		this.dirtyOrderHead = null;
		this.syntheticRoot.firstChild = null;
		this.syntheticRoot.lastChild = null;
		this.children.length = 0;
	}
	completeTransaction() {
		this.transaction = void 0;
	}
	/** True while a transaction is open and mutations are being journaled. */
	get inTransaction() {
		return this.transaction !== void 0;
	}
	publishedNode(id) {
		return this.transaction?.nodesById.has(id) ? this.transaction.nodesById.get(id) : this.nodesById.get(id);
	}
	publishedChildren(parentId) {
		const parent = parentId === this.syntheticRoot.id ? this.syntheticRoot : this.publishedNode(parentId);
		if (parent === void 0) return [];
		return (this.transaction?.nodeStates.get(parent)?.children ?? parent.children).filter((child) => !this.createdIds.has(child.id));
	}
	publishedParentId(id) {
		const node = this.publishedNode(id);
		if (node === void 0) throw new Error(`Node ${id} was not published`);
		const previous = this.transaction?.nodeStates.get(node);
		return (previous === void 0 ? node.parent : previous.parent)?.id ?? this.syntheticRoot.id;
	}
	/** Disjoint removals from the published tree, before any current reparenting. */
	deletedNodeIds() {
		const transaction = this.transaction;
		if (transaction === void 0) return [];
		const removed = /* @__PURE__ */ new Map();
		for (const [id, previous] of transaction.nodesById) if (previous !== void 0 && !this.nodesById.has(id)) removed.set(id, previous);
		const roots = [];
		for (const [id, node] of removed) {
			const previous = transaction.nodeStates.get(node);
			const parent = previous === void 0 ? node.parent : previous.parent;
			if (parent === null || !removed.has(parent.id)) roots.push(id);
		}
		return roots.sort((a, b) => a - b);
	}
	rollbackTransaction() {
		const transaction = this.transaction;
		if (transaction === void 0) return;
		for (const node of transaction.newNodes) this.resetNode(node);
		for (const [node, value] of transaction.dirtyProps) if (value === void 0) this.dirtyProps.delete(node);
		else this.dirtyProps.set(node, value);
		for (const [node, state] of transaction.nodeStates) this.restoreNode(state);
		for (const [key, value] of transaction.nodesById) if (value === void 0) this.nodesById.delete(key);
		else this.nodesById.set(key, value);
		for (const [key, value] of transaction.createdIds) if (value === void 0) this.createdIds.delete(key);
		else this.createdIds.add(key);
		for (const [key, value] of transaction.deletedRoots) if (value === void 0) this.deletedRoots.delete(key);
		else this.deletedRoots.add(key);
		for (const [key, value] of transaction.movedIds) if (value === void 0) this.movedIds.delete(key);
		else this.movedIds.add(key);
		for (const [key, value] of transaction.updatedMasks) if (value === void 0) this.updatedMasks.delete(key);
		else this.updatedMasks.set(key, value);
		for (const [nodeId, snapshot] of transaction.registry) this.listenerRegistry.restoreNode(nodeId, snapshot);
		this.dirtyOrderHead = transaction.dirtyOrderHead;
		this.nextNodeId = transaction.nextNodeId;
		this.nextListenerId = transaction.nextListenerId;
		this.transaction = void 0;
	}
	captureNode(node) {
		return {
			node,
			parent: node.parent,
			children: [...node.children],
			index: node.index,
			firstChild: node.firstChild,
			lastChild: node.lastChild,
			previousSibling: node.previousSibling,
			nextSibling: node.nextSibling,
			childOrderDirty: node.childOrderDirty,
			dirtyNext: node.dirtyNext,
			style: node.style,
			text: node.text,
			tooltip: node.tooltip,
			acceptsPointerMove: node.acceptsPointerMove,
			keyListener: node.keyListener,
			listenerId: node.listenerId,
			focusCallback: node.focusCallback,
			blurCallback: node.blurCallback,
			detachedFocusPending: node.detachedFocusPending,
			nativeFocused: node.nativeFocused,
			pointerDownOutsideCallback: node.pointerDownOutsideCallback,
			hoverCallback: node.hoverCallback,
			hovered: node.hovered,
			scrollCallback: node.scrollCallback,
			listener: node.listener,
			focusable: node.focusable,
			selectable: node.selectable,
			disabled: node.disabled,
			dragCallbacks: { ...node.dragCallbacks },
			hostProperties: node.hostProperties,
			latestNativeText: node.latestNativeText,
			latestNativeEditSeq: node.latestNativeEditSeq,
			layoutCallback: node.layoutCallback,
			observesLayout: node.observesLayout,
			latestNativeSelection: node.latestNativeSelection === null ? null : { ...node.latestNativeSelection },
			inputCallbacks: node.inputCallbacks,
			inputCallbackSources: node.inputCallbackSources,
			visibleRangeCallback: node.visibleRangeCallback,
			animationCompleteCallback: node.animationCompleteCallback,
			extensionEventCallback: node.extensionEventCallback,
			extensionDescriptor: node.extensionDescriptor,
			extensionEventIds: [...node.extensionEventIds],
			lastAnimationGeneration: node.lastAnimationGeneration,
			accessibility: node.accessibility,
			attached: node.attached,
			props: { ...node.props },
			mounted: node.mounted
		};
	}
	restoreNode(state) {
		const node = state.node;
		node.parent = state.parent;
		node.children.length = state.children.length;
		for (let index = 0; index < state.children.length; index++) node.children[index] = state.children[index];
		node.index = state.index;
		node.firstChild = state.firstChild;
		node.lastChild = state.lastChild;
		node.previousSibling = state.previousSibling;
		node.nextSibling = state.nextSibling;
		node.childOrderDirty = state.childOrderDirty;
		node.dirtyNext = state.dirtyNext;
		node.style = state.style;
		node.text = state.text;
		node.tooltip = state.tooltip;
		node.acceptsPointerMove = state.acceptsPointerMove;
		node.keyListener = state.keyListener;
		node.listenerId = state.listenerId;
		node.focusCallback = state.focusCallback;
		node.blurCallback = state.blurCallback;
		node.detachedFocusPending = state.detachedFocusPending;
		node.nativeFocused = state.nativeFocused;
		node.pointerDownOutsideCallback = state.pointerDownOutsideCallback;
		node.hoverCallback = state.hoverCallback;
		node.hovered = state.hovered;
		node.scrollCallback = state.scrollCallback;
		node.listener = state.listener;
		node.focusable = state.focusable;
		node.selectable = state.selectable;
		node.disabled = state.disabled;
		node.dragCallbacks = { ...state.dragCallbacks };
		node.hostProperties = state.hostProperties;
		node.latestNativeText = state.latestNativeText;
		node.latestNativeEditSeq = state.latestNativeEditSeq;
		node.layoutCallback = state.layoutCallback;
		node.observesLayout = state.observesLayout;
		node.latestNativeSelection = state.latestNativeSelection === null ? null : { ...state.latestNativeSelection };
		node.inputCallbacks = state.inputCallbacks;
		node.inputCallbackSources = state.inputCallbackSources;
		node.visibleRangeCallback = state.visibleRangeCallback;
		node.animationCompleteCallback = state.animationCompleteCallback;
		node.extensionEventCallback = state.extensionEventCallback;
		node.extensionDescriptor = state.extensionDescriptor;
		node.extensionEventIds = [...state.extensionEventIds];
		node.lastAnimationGeneration = state.lastAnimationGeneration;
		node.accessibility = state.accessibility;
		node.attached = state.attached;
		const props = node.props;
		for (const key of Object.keys(props)) delete props[key];
		Object.assign(props, state.props);
		node.mounted = state.mounted;
	}
	resetNode(node) {
		node.parent = null;
		node.children.length = 0;
		node.index = 0;
		node.firstChild = null;
		node.lastChild = null;
		node.previousSibling = null;
		node.nextSibling = null;
		node.childOrderDirty = false;
		node.dirtyNext = null;
		node.style = null;
		node.text = null;
		node.tooltip = null;
		node.acceptsPointerMove = false;
		node.keyListener = void 0;
		node.listenerId = 0;
		node.focusCallback = void 0;
		node.blurCallback = void 0;
		node.detachedFocusPending = false;
		node.nativeFocused = false;
		node.pointerDownOutsideCallback = void 0;
		node.hoverCallback = void 0;
		node.hovered = false;
		node.scrollCallback = void 0;
		node.listener = void 0;
		node.focusable = false;
		node.selectable = false;
		node.disabled = false;
		node.dragCallbacks = {};
		node.hostProperties = null;
		node.latestNativeText = null;
		node.latestNativeEditSeq = 0;
		node.layoutCallback = void 0;
		node.observesLayout = false;
		node.latestNativeSelection = null;
		node.inputCallbacks = null;
		node.inputCallbackSources = null;
		node.visibleRangeCallback = void 0;
		node.animationCompleteCallback = void 0;
		node.extensionEventCallback = void 0;
		node.extensionDescriptor = void 0;
		node.extensionEventIds = [];
		node.lastAnimationGeneration = null;
		node.accessibility = null;
		node.attached = false;
		const props = node.props;
		for (const key of Object.keys(props)) delete props[key];
		node.mounted = false;
	}
	setNodeProps(node, props) {
		this.journalNode(node);
		this.journalRegistry(node.id);
		try {
			validateProps(node.kind, props);
		} catch (error) {
			node.root.invalid = true;
			node.root.validationError = error instanceof Error ? error : new Error(String(error));
			throw error;
		}
		const spec = hostKindSpec(node.kind);
		node.style = node.kind === "RawText" ? null : props.style;
		node.tooltip = node.kind === "View" || node.kind === "Pressable" ? props.tooltip ?? null : null;
		node.extensionDescriptor = node.kind === "Extension" ? props.__extensionDescriptor : void 0;
		node.extensionEventIds = node.extensionDescriptor?.eventIds ?? [];
		node.extensionEventCallback = node.kind === "Extension" && typeof props.onExtensionEvent === "function" ? props.onExtensionEvent : void 0;
		node.hostProperties = node.kind === "Extension" && node.extensionDescriptor !== void 0 ? {
			type: "extension",
			value: normalizeExtensionProperties(node.extensionDescriptor, props.__extensionProps ?? props)
		} : spec.projector === "input" ? inputFor(node, props) : spec.projector === "virtualList" ? virtualListFor(node, props) : spec.projector === "image" ? imageFor(node, props) : spec.projector === "icon" ? iconFor(node, props) : spec.projector === "drag" ? dragFor(node, props) : null;
		node.accessibility = accessibilityFor(node.kind, props);
		node.disabled = (node.kind === "Pressable" || node.kind === "TextInput") && props.disabled === true;
		node.focusable = !node.disabled && (node.kind === "View" || node.kind === "Pressable" ? props.focusable ?? false : node.kind === "Text" && props.onPress !== void 0);
		node.selectable = node.kind === "Text" && props.selectable === true;
		node.keyListener = !node.disabled && (node.kind === "View" || node.kind === "Pressable" || node.kind === "TextInput") ? props.onKeyDown : void 0;
		node.listener = (node.kind === "Pressable" || node.kind === "Text") && !node.disabled ? props.onPress : void 0;
		node.pointerCallbacks = !node.disabled && (node.kind === "View" || node.kind === "Pressable") ? {
			down: props.onPointerDown,
			up: props.onPointerUp
		} : null;
		node.pointerMoveCallback = !node.disabled && (node.kind === "View" || node.kind === "Pressable") ? props.onPointerMove : void 0;
		node.acceptsPointerMove = node.pointerMoveCallback !== void 0;
		node.focusCallback = !node.disabled && (node.kind === "View" || node.kind === "Pressable" || node.kind === "Text") ? props.onFocus : void 0;
		node.blurCallback = !node.disabled && (node.kind === "View" || node.kind === "Pressable" || node.kind === "Text") ? props.onBlur : void 0;
		node.pointerDownOutsideCallback = !node.disabled && node.kind === "View" ? props.onPointerDownOutside : void 0;
		node.hoverCallback = !node.disabled && (node.kind === "View" || node.kind === "Pressable") ? props.onHoverChange : void 0;
		node.scrollCallback = node.kind === "View" ? props.onScroll : void 0;
		node.dragCallbacks = !node.disabled && (node.kind === "View" || node.kind === "Pressable") ? {
			over: props.onDragOver,
			drop: props.onDrop,
			externalFileDrop: props.onExternalFileDrop
		} : {};
		node.layoutCallback = spec.allowsLayout ? props.onLayout : void 0;
		node.observesLayout = node.layoutCallback !== void 0;
		if (node.hoverCallback === void 0) node.hovered = false;
		const previousInputCallbacks = node.inputCallbacks;
		const previousInputSources = node.inputCallbackSources;
		node.inputCallbacks = null;
		node.inputCallbackSources = null;
		node.visibleRangeCallback = node.kind === "VirtualList" ? props.__onVisibleRange : void 0;
		node.animationCompleteCallback = node.style?.transition?.onComplete;
		if (node.kind === "TextInput") {
			const input = props;
			const inputSources = {
				change: input.onChangeText,
				selection: input.onSelectionChange,
				focus: input.onFocus,
				blur: input.onBlur,
				submit: input.onSubmitEditing
			};
			const sameSources = previousInputSources?.change === inputSources.change && previousInputSources?.selection === inputSources.selection && previousInputSources?.focus === inputSources.focus && previousInputSources?.blur === inputSources.blur && previousInputSources?.submit === inputSources.submit;
			node.inputCallbackSources = inputSources;
			node.inputCallbacks = sameSources && previousInputCallbacks !== null ? previousInputCallbacks : {
				change: inputSources.change ? (event) => inputSources.change?.(event.text) : void 0,
				selection: inputSources.selection ? (event) => inputSources.selection?.(event.selection) : void 0,
				focus: inputSources.focus,
				blur: inputSources.blur,
				submit: inputSources.submit
			};
		}
		const hasListener = node.listener !== void 0 || node.keyListener !== void 0 || node.acceptsPointerMove || node.pointerCallbacks !== null && (node.pointerCallbacks.down !== void 0 || node.pointerCallbacks.up !== void 0) || node.focusCallback !== void 0 || node.blurCallback !== void 0 || node.pointerDownOutsideCallback !== void 0 || node.hoverCallback !== void 0 || node.scrollCallback !== void 0 || node.layoutCallback !== void 0 || Object.values(node.dragCallbacks).some((callback) => callback !== void 0) || Object.values(node.inputCallbacks ?? {}).some((callback) => callback !== void 0) || node.visibleRangeCallback !== void 0 || node.animationCompleteCallback !== void 0 || node.kind === "Extension" && node.extensionEventCallback !== void 0 && node.extensionEventIds.length > 0;
		if (hasListener && node.listenerId === 0) node.listenerId = this.allocateListener();
		if (!hasListener) node.listenerId = 0;
		this.listenerRegistry.bind(node, node.listenerId, hasListener);
	}
	updateNodeProps(node, props) {
		const previousStyle = node.style;
		const previousListenerId = node.listenerId;
		const previousListener = this.listenerRegistry.current(node.id);
		const previousFocusable = node.focusable;
		const previousSelectable = node.selectable;
		const previousProperties = node.hostProperties;
		const previousAccessibility = node.accessibility;
		const previousTooltip = node.tooltip;
		const previousAcceptsPointerMove = node.acceptsPointerMove;
		const previousObservesLayout = node.observesLayout;
		this.setNodeProps(node, props);
		let mask = 0;
		if (!equalStyle(previousStyle, node.style)) mask |= 1;
		if (previousListenerId !== node.listenerId || previousListener !== this.listenerRegistry.current(node.id)) mask |= 4;
		if (previousFocusable !== node.focusable) mask |= 32;
		if (previousSelectable !== node.selectable) mask |= 64;
		if (previousTooltip !== node.tooltip) mask |= 128;
		if (previousAcceptsPointerMove !== node.acceptsPointerMove) mask |= 256;
		if (previousObservesLayout !== node.observesLayout) mask |= 512;
		if (!equalHostProperties(previousProperties, node.hostProperties)) mask |= 8;
		if (!equalAccessibility(previousAccessibility, node.accessibility)) mask |= 16;
		return mask;
	}
	insertNode(parent, node, anchor, bootstrapped) {
		if (anchor !== void 0 && ((anchor.parent ?? this.syntheticRoot) !== parent || !this.isLinkedChild(parent, anchor))) {
			this.owner.invalid = true;
			throw new TypeError("insertion anchor is not a child of the parent");
		}
		if (anchor === node) return;
		const wasAttached = node.attached;
		if (node.parent !== null || wasAttached) this.detachFromParent(node);
		this.journalNode(parent);
		this.journalNode(node);
		node.detachedFocusPending = false;
		this.linkChild(parent, node, anchor);
		if (parent.attached) {
			if (wasAttached) this.markMoved(node, bootstrapped);
			else this.attachSubtree(node, bootstrapped);
		} else if (wasAttached) {
			this.markDeleted(node, bootstrapped);
			this.detachSubtree(node);
		}
	}
	removeNode(parent, node, bootstrapped) {
		if ((node.parent ?? this.syntheticRoot) !== parent || !this.isLinkedChild(parent, node)) {
			this.owner.invalid = true;
			throw new TypeError("removed node is not a child of the parent");
		}
		if (node.attached) this.markDeleted(node, bootstrapped);
		this.detachFromParent(node);
		if (node.attached) this.detachSubtree(node);
	}
	/** A node counts as a child only while its sibling links actually chain it under `parent`. */
	isLinkedChild(parent, node) {
		return node.previousSibling !== null || node.nextSibling !== null || parent.firstChild === node;
	}
	/** O(1) sibling-link insertion before `anchor` (or append); no suffix work. */
	linkChild(parent, node, anchor) {
		this.dirtyChildOrder(parent);
		node.parent = parent === this.syntheticRoot ? null : parent;
		const previous = anchor === void 0 ? parent.lastChild : anchor.previousSibling;
		const next = anchor ?? null;
		if (previous !== null) this.journalNode(previous);
		if (next !== null) this.journalNode(next);
		node.previousSibling = previous;
		node.nextSibling = next;
		if (previous === null) parent.firstChild = node;
		else previous.nextSibling = node;
		if (next === null) parent.lastChild = node;
		else next.previousSibling = node;
	}
	detachFromParent(node) {
		const parent = node.parent ?? this.syntheticRoot;
		this.journalNode(parent);
		this.journalNode(node);
		this.unlinkChild(parent, node);
	}
	/** O(1) sibling-link removal. The detached node keeps its last materialized index. */
	unlinkChild(parent, node) {
		this.dirtyChildOrder(parent);
		const { previousSibling, nextSibling } = node;
		if (previousSibling !== null) this.journalNode(previousSibling);
		if (nextSibling !== null) this.journalNode(nextSibling);
		if (previousSibling === null) parent.firstChild = nextSibling;
		else previousSibling.nextSibling = nextSibling;
		if (nextSibling === null) parent.lastChild = previousSibling;
		else nextSibling.previousSibling = previousSibling;
		node.parent = null;
		node.previousSibling = null;
		node.nextSibling = null;
	}
	dirtyChildOrder(parent) {
		if (parent.childOrderDirty) return;
		this.journalNode(parent);
		parent.childOrderDirty = true;
		parent.dirtyNext = this.dirtyOrderHead;
		this.dirtyOrderHead = parent;
	}
	/** Rebuild `children` and `index` from sibling links; once per changed parent. */
	materializeOrder(parent) {
		if (!parent.childOrderDirty) return;
		this.journalNode(parent);
		const children = parent.children;
		let child = parent.firstChild;
		let index = 0;
		while (child !== null) {
			if (child.index !== index) {
				this.journalNode(child);
				child.index = index;
			}
			children[index] = child;
			index += 1;
			child = child.nextSibling;
		}
		children.length = index;
		parent.childOrderDirty = false;
	}
	/**
	* Materialize every parent whose child order changed. Called once per commit
	* before snapshot/patch planning; the scalar fast path drains an empty list.
	* The list is consumed so every transaction boundary leaves arrays in sync.
	*/
	materializeChildOrder() {
		let parent = this.dirtyOrderHead;
		this.dirtyOrderHead = null;
		while (parent !== null) {
			const next = parent.dirtyNext;
			this.journalNode(parent);
			parent.dirtyNext = null;
			this.materializeOrder(parent);
			parent = next;
		}
	}
	attachSubtree(node, bootstrapped) {
		const wasCreated = this.createdIds.has(node.id);
		this.journalSet(this.transaction?.deletedRoots, this.deletedRoots, node.id);
		const wasDeleted = this.deletedRoots.delete(node.id);
		const attach = (current) => {
			this.journalNode(current);
			current.attached = true;
			current.detachedFocusPending = false;
			this.journalMap(this.transaction?.nodesById, this.nodesById, current.id);
			this.nodesById.set(current.id, current);
			for (let child = current.firstChild; child !== null; child = child.nextSibling) attach(child);
		};
		attach(node);
		if (!bootstrapped || wasCreated) return;
		if (wasDeleted) {
			this.journalSet(this.transaction?.movedIds, this.movedIds, node.id);
			this.movedIds.add(node.id);
			return;
		}
		const markCreated = (current) => {
			this.journalSet(this.transaction?.createdIds, this.createdIds, current.id);
			this.createdIds.add(current.id);
			for (let child = current.firstChild; child !== null; child = child.nextSibling) markCreated(child);
		};
		markCreated(node);
	}
	detachSubtree(node) {
		const retainFocusRouting = node.nativeFocused;
		this.journalNode(node);
		node.attached = false;
		node.detachedFocusPending = retainFocusRouting;
		this.journalMap(this.transaction?.nodesById, this.nodesById, node.id);
		this.nodesById.delete(node.id);
		if (!retainFocusRouting) {
			this.journalRegistry(node.id);
			this.listenerRegistry.remove(node.id);
		}
		for (let child = node.firstChild; child !== null; child = child.nextSibling) this.detachSubtree(child);
	}
	releaseDetachedFocus(node) {
		if (node.attached || !node.detachedFocusPending) return;
		this.journalNode(node);
		node.detachedFocusPending = false;
		this.journalRegistry(node.id);
		this.listenerRegistry.remove(node.id);
		node.nativeFocused = false;
	}
	markMoved(node, bootstrapped) {
		if (bootstrapped && !this.createdIds.has(node.id)) {
			this.journalSet(this.transaction?.movedIds, this.movedIds, node.id);
			this.movedIds.add(node.id);
		}
	}
	markUpdated(node, mask, bootstrapped) {
		if (bootstrapped && mask !== 0 && !this.createdIds.has(node.id)) {
			if (this.transaction !== void 0 && !this.transaction.updatedMasks.has(node.id)) this.transaction.updatedMasks.set(node.id, this.updatedMasks.get(node.id));
			this.updatedMasks.set(node.id, (this.updatedMasks.get(node.id) ?? 0) | mask);
		}
	}
	markDeleted(node, bootstrapped) {
		if (bootstrapped) {
			this.journalSet(this.transaction?.deletedRoots, this.deletedRoots, node.id);
			this.deletedRoots.add(node.id);
		}
	}
	clearMutations() {
		for (const id of this.createdIds) this.journalSet(this.transaction?.createdIds, this.createdIds, id);
		for (const id of this.deletedRoots) this.journalSet(this.transaction?.deletedRoots, this.deletedRoots, id);
		for (const id of this.movedIds) this.journalSet(this.transaction?.movedIds, this.movedIds, id);
		for (const id of this.updatedMasks.keys()) if (this.transaction !== void 0 && !this.transaction.updatedMasks.has(id)) this.transaction.updatedMasks.set(id, this.updatedMasks.get(id));
		this.createdIds.clear();
		this.deletedRoots.clear();
		this.movedIds.clear();
		this.updatedMasks.clear();
	}
	snapshotNodes() {
		const nodes = [];
		const visit = (node, parentId, index) => {
			nodes.push({
				id: node.id,
				parentId,
				index,
				kind: node.kind,
				style: node.style,
				text: node.text,
				listenerId: node.listenerId,
				hostProperties: node.hostProperties,
				accessibility: node.accessibility,
				focusable: node.focusable,
				selectable: node.selectable,
				tooltip: node.tooltip,
				acceptsPointerMove: node.acceptsPointerMove,
				observesLayout: node.observesLayout
			});
			let childIndex = 0;
			for (let child = node.firstChild; child !== null; child = child.nextSibling) {
				visit(child, node.id, childIndex);
				childIndex += 1;
			}
		};
		visit(this.syntheticRoot, 0, 0);
		return nodes;
	}
	nodeDepth(node) {
		let depth = 0;
		let parent = node.parent;
		while (parent !== null) {
			depth += 1;
			parent = parent.parent;
		}
		return depth;
	}
	nativeParentId(node) {
		return node.parent?.id ?? this.syntheticRoot.id;
	}
};
//#endregion
//#region packages/solid-gpui/src/renderer/structural-patch.ts
/**
* Fenwick tree with exclusive-prefix queries: prefix(x) sums indexes < x.
*/
var RankIndex = class {
	tree;
	constructor(size) {
		this.tree = new Array(size + 1).fill(0);
	}
	add(index, delta) {
		for (let i = index + 1; i < this.tree.length; i += i & -i) this.tree[i] += delta;
	}
	prefix(index) {
		let sum = 0;
		for (let i = Math.min(index, this.tree.length - 1); i > 0; i -= i & -i) sum += this.tree[i];
		return sum;
	}
};
/**
* Plan sequential wire indexes against the published tree, not final Solid
* indexes.
*
* Per touched parent: children whose published order already agrees with the
* final order (one longest increasing subsequence of published positions) act
* as fixed anchors and never move; every other child moves at most once,
* immediately before the next anchor to its right (or the end). A right-to-left
* walk over final slots with two Fenwick rank indexes — original positions that
* still remain, and insertion buckets ahead of each anchor — derives each
* emitted Move's exact sequential index with O(log n) rank arithmetic and no
* rescans. A rotation collapses to a single Move and swapping the two endpoint
* children costs two, regardless of parent size.
*/
function structuralPatch(graph, snapshotNode) {
	const operations = [];
	const children = /* @__PURE__ */ new Map();
	const parents = /* @__PURE__ */ new Map();
	const siblings = (parentId) => {
		let value = children.get(parentId);
		if (value === void 0) {
			value = graph.publishedChildren(parentId).map((node) => node.id);
			children.set(parentId, value);
		}
		return value;
	};
	const parentOf = (id) => parents.get(id) ?? graph.publishedParentId(id);
	const detach = (id) => {
		const old = siblings(parentOf(id));
		const index = old.indexOf(id);
		if (index < 0) throw new Error(`Published parent does not contain node ${id}`);
		old.splice(index, 1);
	};
	const current = (ids) => [...ids].map((id) => graph.nodesById.get(id)).filter((node) => node !== void 0).sort((a, b) => graph.nodeDepth(a) - graph.nodeDepth(b) || a.index - b.index || a.id - b.id);
	for (const node of current(graph.createdIds)) {
		const parentId = graph.nativeParentId(node);
		const target = siblings(parentId);
		operations.push({
			type: "create",
			node: {
				...snapshotNode(node),
				index: target.length
			}
		});
		target.push(node.id);
		parents.set(node.id, parentId);
		children.set(node.id, []);
	}
	for (const node of current(graph.movedIds)) {
		if (graph.createdIds.has(node.id)) continue;
		const parentId = graph.nativeParentId(node);
		if (parentOf(node.id) !== parentId) {
			detach(node.id);
			const target = siblings(parentId);
			operations.push({
				type: "move",
				id: node.id,
				parentId,
				index: target.length
			});
			target.push(node.id);
			parents.set(node.id, parentId);
		} else siblings(parentId);
	}
	for (const id of graph.deletedNodeIds()) {
		detach(id);
		operations.push({
			type: "delete",
			id
		});
	}
	for (const [parentId, order] of children) {
		const parent = parentId === graph.syntheticRoot.id ? graph.syntheticRoot : graph.nodesById.get(parentId);
		if (parent === void 0) continue;
		const final = parent.children;
		const publishedPosition = /* @__PURE__ */ new Map();
		for (let index = 0; index < order.length; index++) publishedPosition.set(order[index], index);
		const positions = new Array(final.length);
		for (let index = 0; index < final.length; index++) {
			const published = publishedPosition.get(final[index].id);
			if (published === void 0) throw new Error(`Structural patch lost child ${final[index].id}`);
			positions[index] = published;
		}
		let sorted = positions.length < 2;
		for (let index = 1; index < positions.length; index++) if (positions[index] <= positions[index - 1]) {
			sorted = false;
			break;
		}
		if (sorted) continue;
		const tails = [];
		const previous = new Array(positions.length);
		for (let index = 0; index < positions.length; index++) {
			const value = positions[index];
			let low = 0;
			let high = tails.length;
			while (low < high) {
				const middle = low + high >> 1;
				if (positions[tails[middle]] < value) low = middle + 1;
				else high = middle;
			}
			previous[index] = low === 0 ? -1 : tails[low - 1];
			if (low === tails.length) tails.push(index);
			else tails[low] = index;
		}
		const anchored = new Array(positions.length).fill(false);
		for (let index = tails[tails.length - 1]; index !== -1; index = previous[index]) anchored[index] = true;
		const remaining = new RankIndex(order.length);
		const inserted = new RankIndex(order.length + 1);
		for (let index = 0; index < order.length; index++) remaining.add(index, 1);
		let boundary = order.length;
		for (let index = positions.length - 1; index >= 0; index--) {
			const published = positions[index];
			if (anchored[index]) {
				boundary = published;
				continue;
			}
			const source = remaining.prefix(published) + inserted.prefix(published + 1);
			remaining.add(published, -1);
			const destination = remaining.prefix(boundary) + inserted.prefix(boundary);
			inserted.add(boundary, 1);
			if (source !== destination) operations.push({
				type: "move",
				id: final[index].id,
				parentId,
				index: destination
			});
		}
	}
	return operations;
}
//#endregion
//#region packages/solid-gpui/src/renderer/host-tree.ts
var HostTree = class {
	options;
	graph;
	syntheticRoot;
	children;
	invalid = false;
	validationError;
	bootstrapped = false;
	disposed = false;
	transactionBootstrapped;
	constructor(options) {
		this.options = options;
		this.graph = new NodeGraph(this);
		this.syntheticRoot = this.graph.syntheticRoot;
		this.children = this.graph.children;
	}
	isDisposed() {
		return this.disposed;
	}
	invokeNative(moduleId, moduleDigest, functionId, args, options) {
		return this.options.invokeNative(moduleId, moduleDigest, functionId, args, options);
	}
	allocateNode(kind) {
		return this.graph.allocateNode(kind);
	}
	insertNode(parent, node, anchor) {
		this.graph.insertNode(parent, node, anchor, this.bootstrapped);
	}
	removeNode(parent, node) {
		this.graph.removeNode(parent, node, this.bootstrapped);
	}
	submitCommand(node, kind, payload) {
		return this.options.submitCommand(node, kind, payload);
	}
	submitCommandValue(node, kind, payload, options) {
		return this.options.submitCommandValue(node, kind, payload, options);
	}
	releaseDetachedFocus(node) {
		this.graph.releaseDetachedFocus(node);
	}
	markPropsDirty(node, groups) {
		this.graph.markPropsDirty(node, groups);
	}
	recordNodeMutation(node) {
		this.graph.recordNodeMutation(node);
	}
	setNodeProps(node, props) {
		this.graph.setNodeProps(node, props);
	}
	markUpdated(node, mask) {
		this.graph.markUpdated(node, mask, this.bootstrapped);
	}
	beginRender() {
		if (this.graph.inTransaction) return;
		this.graph.beginTransaction();
		this.transactionBootstrapped = this.bootstrapped;
		this.invalid = false;
		this.validationError = void 0;
		this.graph.clearMutations();
	}
	commit() {
		if (this.disposed) {
			this.graph.rollbackTransaction();
			return;
		}
		if (this.invalid) {
			this.abortTransaction();
			return;
		}
		try {
			this.graph.finalizeDirtyProps();
			this.graph.materializeChildOrder();
			const baseRevision = this.options.getRevision();
			const revision = nextU32(baseRevision, "revision");
			if (!this.bootstrapped && this.children.length === 0) {
				this.graph.completeTransaction();
				this.transactionBootstrapped = void 0;
				this.graph.clearMutations();
				return;
			}
			const commit = this.buildCommit(baseRevision, revision);
			if (commit === null) {
				this.graph.completeTransaction();
				this.transactionBootstrapped = void 0;
				this.graph.clearMutations();
				return;
			}
			if (!this.options.submitCommit(commit)) {
				this.abortTransaction();
				return;
			}
			this.graph.publishRevision(revision);
			this.graph.completeTransaction();
			this.transactionBootstrapped = void 0;
			this.graph.clearMutations();
			this.bootstrapped = true;
		} catch (error) {
			this.abortTransaction();
			this.options.onCommitError(error);
		}
	}
	dispose() {
		if (this.disposed) return;
		this.disposed = true;
		this.graph.dispose();
		this.transactionBootstrapped = void 0;
		this.invalid = false;
		this.validationError = void 0;
	}
	findListener(listenerId, revision) {
		return this.graph.findListener(listenerId, revision);
	}
	abortTransaction() {
		this.graph.rollbackTransaction();
		this.bootstrapped = this.transactionBootstrapped ?? this.bootstrapped;
		this.transactionBootstrapped = void 0;
		this.invalid = false;
		this.graph.clearMutations();
	}
	buildCommit(baseRevision, revision) {
		if (!this.bootstrapped) return {
			type: "snapshot",
			surfaceId: this.options.surfaceId,
			epoch: this.options.epoch,
			baseRevision,
			revision,
			nodes: this.graph.snapshotNodes()
		};
		const operations = structuralPatch(this.graph, (node) => this.snapshotNode(node));
		for (const [id, mask] of [...this.graph.updatedMasks.entries()].sort(([a], [b]) => a - b)) {
			const node = this.graph.nodesById.get(id);
			if (node === void 0) continue;
			operations.push({
				type: "update",
				id,
				mask,
				style: mask & 1 ? node.style : null,
				text: mask & 2 && node.kind === "RawText" ? node.text : null,
				listenerId: mask & 4 ? node.listenerId : 0,
				hostProperties: mask & 8 ? node.hostProperties : null,
				accessibility: mask & 16 ? node.accessibility : null,
				focusable: node.focusable,
				selectable: node.selectable,
				tooltip: node.tooltip,
				acceptsPointerMove: node.acceptsPointerMove,
				observesLayout: node.observesLayout
			});
		}
		if (operations.length === 0) return null;
		return {
			type: "patch",
			surfaceId: this.options.surfaceId,
			epoch: this.options.epoch,
			baseRevision,
			revision,
			operations
		};
	}
	snapshotNode(node) {
		return {
			id: node.id,
			parentId: this.graph.nativeParentId(node),
			index: node.index,
			kind: node.kind,
			style: node.style,
			text: node.text,
			listenerId: node.listenerId,
			hostProperties: node.hostProperties,
			accessibility: node.accessibility,
			focusable: node.focusable,
			selectable: node.selectable,
			tooltip: node.tooltip,
			acceptsPointerMove: node.acceptsPointerMove,
			observesLayout: node.observesLayout
		};
	}
};
//#endregion
//#region packages/solid-gpui/src/renderer/host-config.ts
var ownerTrees = /* @__PURE__ */ new WeakMap();
var pendingCommits = /* @__PURE__ */ new WeakMap();
var activeTree;
var transactionDepths = /* @__PURE__ */ new WeakMap();
function bindRootOwner(tree) {
	const owner = getOwner();
	if (owner !== null) ownerTrees.set(owner, tree);
}
function ownerTree() {
	let owner = getOwner();
	const currentOwner = owner;
	while (owner !== null) {
		const tree = ownerTrees.get(owner);
		if (tree !== void 0) {
			if (currentOwner !== null && currentOwner !== owner) ownerTrees.set(currentOwner, tree);
			return tree;
		}
		owner = owner.owner ?? null;
	}
}
function resolveTree(node) {
	if (node !== void 0) return node.root;
	const owner = ownerTree();
	if (owner !== void 0) return owner;
	if (activeTree !== void 0) {
		bindRootOwner(activeTree);
		return activeTree;
	}
	throw new Error("Solid GPUI host operation is not associated with a root");
}
function withRoot(tree, callback) {
	const previous = activeTree;
	activeTree = tree;
	try {
		return callback();
	} finally {
		activeTree = previous;
	}
}
function withRootTransaction(tree, callback) {
	return withRoot(tree, () => {
		transactionDepths.set(tree, (transactionDepths.get(tree) ?? 0) + 1);
		try {
			return callback();
		} finally {
			const depth = (transactionDepths.get(tree) ?? 1) - 1;
			if (depth === 0) transactionDepths.delete(tree);
			else transactionDepths.set(tree, depth);
		}
	});
}
function cancelScheduledCommit(tree) {
	return pendingCommits.delete(tree) && tree.graph.inTransaction;
}
/** Commands cross the same root transaction seam as renderer mutations. */
function afterRootCommit(tree, submit) {
	if (transactionDepths.has(tree)) return Promise.resolve().then(() => afterRootCommit(tree, submit));
	try {
		if (cancelScheduledCommit(tree)) tree.commit();
		if (tree.invalid || tree.validationError) throw tree.validationError ?? /* @__PURE__ */ new Error("Native command follows an invalid commit");
		return submit();
	} catch (error) {
		return Promise.reject(error);
	}
}
function prepareMutation(tree) {
	if (tree.isDisposed() || transactionDepths.has(tree)) return;
	if (!tree.graph.inTransaction) tree.beginRender();
	if (pendingCommits.has(tree)) return;
	const token = {};
	pendingCommits.set(tree, token);
	queueMicrotask(() => {
		if (pendingCommits.get(tree) !== token) return;
		pendingCommits.delete(tree);
		if (!tree.isDisposed()) tree.commit();
	});
}
function configure(node) {
	if (node.mounted) return;
	node.root.setNodeProps(node, node.props);
	node.mounted = true;
}
function parentDepth(parent) {
	return parent.kind === "Text" && parent.parent?.kind === "Text" ? 2 : 1;
}
function assertParentAndChild(parent, child) {
	const tree = parent.root;
	if (child.root !== tree) {
		tree.invalid = true;
		throw new TypeError("cannot insert a host node from a different Solid GPUI root");
	}
	try {
		assertChildKind(parent.kind, child.kind, parentDepth(parent));
	} catch (error) {
		tree.invalid = true;
		throw error;
	}
	return tree;
}
function createElement$1(type) {
	const tree = resolveTree();
	prepareMutation(tree);
	if (!Object.hasOwn(VALID_HOST_TYPES, type)) {
		tree.invalid = true;
		throw new TypeError(`Unknown GPUI host type: ${type}`);
	}
	return tree.allocateNode(type);
}
function createTextNode$1(value) {
	const tree = resolveTree();
	prepareMutation(tree);
	const node = tree.allocateNode("RawText");
	node.text = String(value);
	return node;
}
function replaceText(node, value) {
	const text = String(value);
	if (node.text === text) return;
	const tree = node.root;
	if (node.attached) {
		prepareMutation(tree);
		tree.recordNodeMutation(node);
	}
	node.text = text;
	if (node.attached && node.mounted) tree.markUpdated(node, 2);
}
function setProperty(node, name, value) {
	if (name === "key") return;
	const tree = node.root;
	if (node.mounted) {
		prepareMutation(tree);
		tree.recordNodeMutation(node);
	}
	const props = node.props;
	if (value === void 0) delete props[name];
	else props[name] = value;
	if (node.mounted) tree.markPropsDirty(node, propGroup(node.kind, name));
}
function insertNode$1(parent, node, anchor) {
	const tree = assertParentAndChild(parent, node);
	prepareMutation(tree);
	configure(node);
	tree.insertNode(parent, node, anchor);
}
function removeNode(parent, node) {
	const tree = assertParentAndChild(parent, node);
	prepareMutation(tree);
	tree.removeNode(parent, node);
}
function isTextNode(node) {
	return node.kind === "RawText";
}
function getParentNode(node) {
	if (!node.attached) return void 0;
	return node.parent ?? node.root.syntheticRoot;
}
function getFirstChild(node) {
	return node.firstChild ?? void 0;
}
function getNextSibling(node) {
	return node.nextSibling ?? void 0;
}
var hostConfig = {
	createElement: createElement$1,
	createTextNode: createTextNode$1,
	replaceText,
	isTextNode,
	setProperty,
	insertNode: insertNode$1,
	removeNode,
	getParentNode,
	getFirstChild,
	getNextSibling
};
//#endregion
//#region packages/solid-gpui/src/renderer/dispatch.ts
var POINTER_BUTTON_NAMES = {
	1: "left",
	2: "right",
	3: "middle",
	4: "back",
	5: "forward"
};
var INTERACTIVE_NODE_KINDS = ["View", "Pressable"];
var FOCUSABLE_NODE_KINDS = [
	"View",
	"Pressable",
	"Text"
];
function nodeMatchesListener(context, event, allowedKinds) {
	const binding = context.findListener(event.listenerId, event.revision);
	const node = binding?.node;
	if (node === void 0 || !node.attached || !allowedKinds.includes(node.kind) || node.id !== event.nodeId) return;
	return binding;
}
function dispatchEvent(context, event) {
	if (event === null || !context.acceptEvent(event)) return;
	const payload = event.payload;
	if (payload.type === "surface-closed") {
		context.onSurfaceClosed?.();
		return;
	}
	if (payload.type === "close-requested") {
		context.onCloseRequested?.(payload.requestId);
		return;
	}
	if (payload.type === "action") {
		context.onAction?.(payload.action);
		return;
	}
	if (payload.type === "notification-response") {
		context.onNotificationResponse?.({
			tag: payload.tag,
			actionId: payload.actionId
		});
		return;
	}
	if (payload.type === "command-result") {
		context.resolveCommandResult(payload.result);
		return;
	}
	if (payload.type === "window-resize") {
		context.onWindowResize?.(payload.width, payload.height, payload.scaleFactor);
		return;
	}
	if (payload.type === "window-activation") {
		context.onWindowActivation?.(payload.active);
		return;
	}
	if (payload.type === "window-appearance") {
		context.onAppearance?.(payload.appearance);
		return;
	}
	if (payload.type === "pointer-down-outside") {
		nodeMatchesListener(context, event, ["View"])?.pointerDownOutsideCallback?.({
			x: payload.x,
			y: payload.y
		});
		return;
	}
	if ((payload.type === "focus" || payload.type === "blur") && payload.data === void 0) {
		const binding = context.findListener(event.listenerId, event.revision);
		const node = binding?.node;
		if (binding === void 0 || node === void 0 || node.id !== event.nodeId) return;
		if (!node.attached) {
			if (!node.detachedFocusPending) return;
		} else if (!FOCUSABLE_NODE_KINDS.includes(node.kind) || !node.focusable) return;
		const callback = payload.type === "focus" ? binding.focusCallback : binding.blurCallback;
		node.nativeFocused = payload.type === "focus";
		if (!node.attached) node.root.releaseDetachedFocus(node);
		callback?.({
			type: payload.type,
			target: node
		});
		return;
	}
	if (payload.type === "layout") {
		nodeMatchesListener(context, event, [
			"View",
			"Pressable",
			"Text",
			"Image"
		])?.layoutCallback?.({
			x: payload.x,
			y: payload.y,
			width: payload.width,
			height: payload.height
		});
		return;
	}
	if (payload.type === "drag-over" || payload.type === "drag-drop" || payload.type === "external-file-drop") {
		const binding = nodeMatchesListener(context, event, INTERACTIVE_NODE_KINDS);
		if (binding === void 0) return;
		switch (payload.type) {
			case "drag-over":
				binding.dragCallbacks.over?.(payload.dragType);
				break;
			case "drag-drop":
				binding.dragCallbacks.drop?.(payload.dragType);
				break;
			case "external-file-drop": binding.dragCallbacks.externalFileDrop?.([...payload.paths]);
		}
		return;
	}
	if (payload.type === "press") {
		const binding = nodeMatchesListener(context, event, ["Pressable", "Text"]);
		if (binding === void 0 || binding.listener === void 0) return;
		const node = binding.node;
		binding.listener({
			type: "press",
			surfaceId: event.surfaceId,
			epoch: event.epoch,
			revision: event.revision,
			sequence: event.sequence,
			target: node
		});
		return;
	}
	if (payload.type === "scroll") {
		const binding = nodeMatchesListener(context, event, ["View"]);
		if (binding === void 0 || binding.scrollCallback === void 0) return;
		binding.scrollCallback({
			deltaKind: payload.deltaKind === 1 ? "pixels" : "lines",
			dx: payload.dx,
			dy: payload.dy,
			x: payload.x,
			y: payload.y,
			modifiers: [...payload.modifiers]
		});
		return;
	}
	if (payload.type === "key") {
		const binding = nodeMatchesListener(context, event, [
			"View",
			"Pressable",
			"TextInput",
			"Text"
		]);
		if (binding === void 0) return;
		const node = binding.node;
		if (FOCUSABLE_NODE_KINDS.includes(node.kind) && !node.focusable || binding.keyListener === void 0) return;
		binding.keyListener({
			key: payload.key,
			modifiers: [...payload.modifiers],
			action: payload.action === 1 ? "down" : payload.action === 2 ? "repeat" : "up"
		});
		return;
	}
	if (payload.type === "pointer" || payload.type === "pointer-move") {
		const binding = nodeMatchesListener(context, event, INTERACTIVE_NODE_KINDS);
		if (binding === void 0) return;
		const node = binding.node;
		if (payload.type === "pointer-move") {
			if (binding.pointerMoveCallback === void 0) return;
			binding.pointerMoveCallback({
				type: "pointermove",
				x: payload.x,
				y: payload.y,
				modifiers: [...payload.modifiers],
				target: node
			});
			return;
		}
		(payload.action === 1 ? binding.pointerCallbacks?.down : binding.pointerCallbacks?.up)?.({
			type: payload.action === 1 ? "pointerdown" : "pointerup",
			button: POINTER_BUTTON_NAMES[payload.button],
			modifiers: [...payload.modifiers],
			clickCount: payload.clickCount,
			x: payload.x,
			y: payload.y,
			target: node
		});
		return;
	}
	if (payload.type === "hover") {
		const binding = nodeMatchesListener(context, event, INTERACTIVE_NODE_KINDS);
		if (binding === void 0) return;
		const node = binding.node;
		node.hovered = !node.hovered;
		binding.hoverCallback?.(node.hovered);
		return;
	}
	if (payload.type === "extension") {
		const binding = context.findListener(event.listenerId, event.revision);
		const node = binding?.node;
		if (binding === void 0 || node === void 0 || !node.attached || node.kind !== "Extension" || node.id !== event.nodeId || payload.eventId === 0 || !node.extensionEventIds.includes(payload.eventId)) return;
		binding.extensionEventCallback?.({
			eventId: payload.eventId,
			fields: payload.fields,
			target: node
		});
		return;
	}
	const binding = context.findListener(event.listenerId, event.revision);
	const node = binding?.node;
	if (binding === void 0 || node === void 0 || !node.attached || node.id !== event.nodeId) return;
	if (payload.type === "visible-range") {
		if (node.kind !== "VirtualList") return;
		binding.visibleRangeCallback?.(payload.start, payload.end);
		return;
	}
	if (payload.type === "animation-complete") {
		if (node.lastAnimationGeneration === payload.generation) return;
		node.lastAnimationGeneration = payload.generation;
		binding.animationCompleteCallback?.(payload.generation);
		return;
	}
	if (payload.type === "submit") {
		if (node.kind !== "TextInput") return;
		binding.inputCallbacks?.submit?.(payload.text);
		return;
	}
	if ((payload.type === "change" || payload.type === "selection" || payload.type === "focus" || payload.type === "blur") && payload.data !== void 0) {
		if (node.kind !== "TextInput") return;
		const callbacks = binding.inputCallbacks;
		if (callbacks === null || callbacks === void 0) return;
		const hostProperties = node.hostProperties;
		const maxLength = hostProperties?.type === "text-input" ? hostProperties.value.maxLength : null;
		const text = truncateUtf16(payload.data.text, maxLength);
		const textEvent = {
			type: payload.type,
			text,
			selection: {
				start: payload.data.selectionStart,
				end: payload.data.selectionEnd,
				reversed: payload.data.reversed,
				composing: payload.data.markedStart === null ? null : {
					start: payload.data.markedStart,
					end: payload.data.markedEnd
				}
			},
			editSeq: payload.data.editSeq,
			target: node
		};
		node.latestNativeText = textEvent.text;
		node.latestNativeEditSeq = textEvent.editSeq;
		node.latestNativeSelection = {
			start: textEvent.selection.start,
			end: textEvent.selection.end,
			reversed: textEvent.selection.reversed,
			markedStart: textEvent.selection.composing?.start ?? null,
			markedEnd: textEvent.selection.composing?.end ?? null
		};
		switch (payload.type) {
			case "change":
				callbacks.change?.(textEvent);
				break;
			case "selection":
				callbacks.selection?.(textEvent);
				break;
			case "focus":
				callbacks.focus?.(textEvent);
				break;
			case "blur": callbacks.blur?.(textEvent);
		}
	}
}
//#endregion
//#region packages/solid-gpui/src/native-call.ts
function validateCallOptions(options) {
	const timeout = options?.timeoutMs;
	if (timeout !== void 0 && (!Number.isInteger(timeout) || timeout < 0 || timeout > 2147483647)) throw new RangeError("Native timeout must be an integer between 0 and 2147483647 milliseconds");
	options?.signal?.throwIfAborted();
}
var NativeCommandError = class extends Error {
	identity;
	name = "NativeCommandError";
	constructor(message, identity) {
		super(message);
		this.identity = identity;
		Object.freeze(identity);
	}
};
//#endregion
//#region packages/solid-gpui/src/renderer/command-client.ts
var CommandClient = class {
	sink;
	pending = /* @__PURE__ */ new Map();
	nextRequest = 1;
	constructor(sink) {
		this.sink = sink;
	}
	allocateRequestId(nextU32) {
		const requestId = this.nextRequest;
		this.nextRequest = nextU32(this.nextRequest, "command request id");
		return requestId;
	}
	submit(command, options) {
		const requestId = command.requestId;
		return new Promise((resolve, reject) => {
			validateCallOptions(options);
			let timeout;
			let submitted = false;
			let cancelled = false;
			const signal = options?.signal;
			const cleanup = () => {
				if (timeout !== void 0) clearTimeout(timeout);
				signal?.removeEventListener("abort", onAbort);
			};
			const cancel = (reason) => {
				if (!this.pending.delete(requestId)) return;
				cancelled = true;
				cleanup();
				reject(reason);
				if (submitted) this.sink.cancelNative(requestId);
			};
			const onAbort = () => cancel(signal.reason);
			this.pending.set(requestId, {
				command: command.command,
				nodeId: command.nodeId,
				resolve,
				reject,
				cleanup,
				identity: {
					surfaceId: command.surfaceId,
					epoch: command.epoch,
					requestId,
					nodeId: command.nodeId,
					command: command.command,
					...command.payload?.type === "invoke-native" ? { functionId: command.payload.functionId } : {}
				}
			});
			signal?.addEventListener("abort", onAbort, { once: true });
			if (options?.timeoutMs !== void 0) timeout = setTimeout(() => cancel(new DOMException("Native command timed out", "TimeoutError")), options.timeoutMs);
			try {
				submitted = this.sink.submitFrame(encodeFrame(command));
				if (!submitted) {
					this.pending.delete(requestId);
					cleanup();
					reject(this.sink.getTerminationError() ?? new TransportTerminatedError("transport is terminated"));
				} else if (cancelled) this.sink.cancelNative(requestId);
			} catch (error) {
				this.pending.delete(requestId);
				cleanup();
				reject(error instanceof Error ? error : new Error(String(error)));
			}
		});
	}
	resolve(result) {
		const pending = this.pending.get(result.requestId);
		if (pending === void 0) return;
		this.pending.delete(result.requestId);
		pending.cleanup();
		if (result.command !== pending.command || result.nodeId !== pending.nodeId) {
			pending.reject(new NativeCommandError("native command response does not match the pending command", pending.identity));
			return;
		}
		if (result.value?.type === "bytes" && (pending.command !== COMMAND_INVOKE_NATIVE || !result.success || !(result.value.value instanceof Uint8Array) || result.value.value.byteLength > 1048576) || pending.command === COMMAND_INVOKE_NATIVE && result.success && result.value?.type !== "bytes") {
			pending.reject(new NativeCommandError("native command returned an invalid byte result", pending.identity));
			return;
		}
		if (result.success) pending.resolve(result.value);
		else pending.reject(new NativeCommandError(String(result.error ?? "native command failed"), pending.identity));
	}
	rejectAll(error) {
		for (const pending of this.pending.values()) {
			pending.cleanup();
			pending.reject(error);
		}
		this.pending.clear();
	}
};
//#endregion
//#region packages/solid-gpui/src/renderer/root-container.ts
var SurfaceClosedError = class extends Error {
	surfaceId;
	constructor(surfaceId) {
		super(`surface ${surfaceId} is closed`);
		this.name = "SurfaceClosedError";
		this.surfaceId = surfaceId;
	}
};
function normalizeClipboardImage(image) {
	if (image === null || typeof image !== "object") throw new TypeError("clipboard image must be an object");
	if (image.format !== "png" && image.format !== "jpeg" && image.format !== "gif" && image.format !== "svg") throw new TypeError("clipboard image format must be png, jpeg, gif, or svg");
	const normalized = image.bytes;
	if (!(normalized instanceof Uint8Array)) throw new TypeError("clipboard image bytes must be a Uint8Array");
	if (normalized.byteLength === 0 || normalized.byteLength > 16776192) throw new RangeError("clipboard image bytes must be non-empty and within the supported size");
	return {
		format: image.format,
		bytes: normalized
	};
}
var TEXT_INPUT_COMMANDS = [
	COMMAND_FOCUS,
	COMMAND_BLUR,
	COMMAND_SET_SELECTION,
	COMMAND_GET_FOCUS
];
var VIEW_COMMANDS = [
	COMMAND_FOCUS,
	COMMAND_BLUR,
	COMMAND_GET_FOCUS
];
var VIRTUAL_LIST_COMMANDS = [
	COMMAND_SCROLL_TO_INDEX,
	COMMAND_SCROLL_TO_END,
	COMMAND_GET_SCROLL_OFFSET,
	COMMAND_SCROLL_TO_OFFSET
];
var RootContainer = class {
	tree;
	surfaceId;
	commandClient;
	epoch;
	revision = 0;
	unmounted = false;
	unhandledError;
	lastEventSequence = 0;
	hasEventSequence = false;
	unsubscribe;
	started = false;
	scheduleDispatch;
	submitFrameImpl;
	transportTerminated = false;
	terminationError;
	onTransportTermination;
	surfaceClosedHandler;
	onCloseRequested;
	onWindowResize;
	onWindowActivation;
	onAction;
	onAppearance;
	onNotificationResponse;
	constructor(options) {
		this.surfaceId = assertU32Option("surfaceId", options.surfaceId);
		this.epoch = assertU32Option("epoch", options.epoch);
		this.scheduleDispatch = options.scheduleDispatch;
		this.submitFrameImpl = options.submitFrame;
		this.onTransportTermination = options.onTransportTermination;
		this.onWindowResize = options.onWindowResize;
		this.onWindowActivation = options.onWindowActivation;
		this.surfaceClosedHandler = options.onSurfaceClosed;
		this.onCloseRequested = options.onCloseRequested;
		this.onAction = options.onAction;
		this.onAppearance = options.onAppearance;
		this.onNotificationResponse = options.onNotificationResponse;
		this.commandClient = new CommandClient(this);
		this.tree = new HostTree({
			invokeNative: (moduleId, moduleDigest, functionId, args, options) => this.invokeNative(moduleId, moduleDigest, functionId, args, options),
			surfaceId: this.surfaceId,
			epoch: this.epoch,
			getRevision: () => this.revision,
			submitCommit: (commit) => {
				if (!this.submitFrame(encodeFrame(commit))) return false;
				this.revision = commit.revision;
				return true;
			},
			onCommitError: (error) => this.recordUnhandledError(error),
			submitCommand: (node, kind, payload) => this.submitCommand(node, kind, payload),
			submitCommandValue: (node, kind, payload, options) => this.submitCommandValue(node, kind, payload, options)
		});
	}
	start(unsubscribeEvents) {
		if (this.started || this.unmounted) {
			unsubscribeEvents();
			return;
		}
		this.started = true;
		this.unsubscribe = unsubscribeEvents;
	}
	terminate(error) {
		if (this.unmounted || this.transportTerminated) return;
		this.transportTerminated = true;
		this.terminationError = error;
		this.tree.invalid = true;
		this.detachEventSubscription();
		this.commandClient.rejectAll(error);
		this.onTransportTermination?.(error);
	}
	detachEventSubscription() {
		const unsubscribe = this.unsubscribe;
		this.unsubscribe = void 0;
		unsubscribe?.();
	}
	submitFrame(frame) {
		if (this.transportTerminated) return false;
		try {
			return this.submitFrameImpl(frame);
		} catch (error) {
			if (error instanceof TransportTerminatedError) {
				this.terminate(error);
				return false;
			}
			throw error;
		}
	}
	isTerminated() {
		return this.transportTerminated;
	}
	getTerminationError() {
		return this.terminationError;
	}
	submitCommand(node, kind, payload) {
		return this.submitCommandValue(node, kind, payload).then(() => void 0);
	}
	submitCommandValue(node, kind, payload, options) {
		if (this.transportTerminated) return Promise.reject(this.terminationError ?? new TransportTerminatedError("transport is terminated"));
		if (this.unmounted) return Promise.reject(new SurfaceClosedError(this.surfaceId));
		if (!node.attached) return Promise.reject(/* @__PURE__ */ new Error(`host node ${node.id} is unavailable; ensure the node is mounted before invoking commands`));
		const isInput = node.kind === "TextInput";
		const isList = node.kind === "VirtualList";
		const isView = node.kind === "View";
		const isExtension = node.kind === "Extension";
		if (!isInput && !isList && !isView && !isExtension) return Promise.reject(/* @__PURE__ */ new Error("host node does not support commands"));
		if (isExtension && (kind !== COMMAND_INVOKE_NATIVE || payload?.type !== "invoke-native")) return Promise.reject(/* @__PURE__ */ new Error("unknown Extension command"));
		if (isView && !node.focusable) return Promise.reject(/* @__PURE__ */ new Error(`View node ${node.id} is not focusable; set focusable={true} before calling focus`));
		if (isInput && !TEXT_INPUT_COMMANDS.includes(kind)) return Promise.reject(/* @__PURE__ */ new Error("unknown TextInput command"));
		if (isView && !VIEW_COMMANDS.includes(kind)) return Promise.reject(/* @__PURE__ */ new Error("unknown View command"));
		if (isList && !VIRTUAL_LIST_COMMANDS.includes(kind)) return Promise.reject(/* @__PURE__ */ new Error("unknown VirtualList command"));
		if (kind === COMMAND_SCROLL_TO_OFFSET && (payload === null || payload.type !== "number" || !Number.isFinite(payload.value) || payload.value < 0)) return Promise.reject(/* @__PURE__ */ new TypeError("scroll offset must be finite and non-negative"));
		if (kind === COMMAND_SET_SELECTION && (payload === null || payload.type !== "selection" || !Number.isInteger(payload.start) || !Number.isInteger(payload.end) || payload.start < 0 || payload.end < payload.start || payload.end > 4294967295)) return Promise.reject(/* @__PURE__ */ new Error(`invalid UTF-16 selection for TextInput node ${node.id}; provide offsets within the text range`));
		if (kind === COMMAND_SCROLL_TO_INDEX && (payload === null || payload.type !== "scroll-index" || !Number.isInteger(payload.index) || payload.index < 0 || payload.index > 4294967295 || node.hostProperties?.type === "virtual-list" && payload.index >= node.hostProperties.value.itemCount)) return Promise.reject(/* @__PURE__ */ new Error(`VirtualList index is out of range for node ${node.id}; use an index below itemCount`));
		let requestId;
		try {
			requestId = this.commandClient.allocateRequestId(nextU32);
		} catch (error) {
			return Promise.reject(error);
		}
		const command = {
			type: "command",
			surfaceId: this.surfaceId,
			epoch: this.epoch,
			afterRevision: this.revision,
			requestId,
			nodeId: node.id,
			command: kind,
			payload
		};
		return this.commandClient.submit(command, options);
	}
	setTitle(title) {
		if (this.transportTerminated) return Promise.reject(this.terminationError ?? new TransportTerminatedError("transport is terminated"));
		if (this.unmounted) return Promise.reject(new SurfaceClosedError(this.surfaceId));
		if (typeof title !== "string" || title.length === 0 || [...title].length > 256) return Promise.reject(/* @__PURE__ */ new TypeError("title must be a non-empty string of at most 256 characters"));
		let requestId;
		try {
			requestId = this.commandClient.allocateRequestId(nextU32);
		} catch (error) {
			return Promise.reject(error);
		}
		const command = {
			type: "command",
			surfaceId: this.surfaceId,
			epoch: this.epoch,
			afterRevision: this.revision,
			requestId,
			nodeId: 1,
			command: COMMAND_SET_TITLE,
			payload: {
				type: "text",
				value: title
			}
		};
		return this.commandClient.submit(command).then(() => void 0);
	}
	submitSurfaceCommand(kind, payload) {
		return this.submitSurfaceCommandValue(kind, payload).then(() => void 0);
	}
	submitSurfaceCommandValue(kind, payload, options) {
		try {
			return this.beginSurfaceCommand(kind, payload, options).result;
		} catch (error) {
			return Promise.reject(error);
		}
	}
	beginSurfaceCommand(kind, payload, options) {
		if (this.transportTerminated) throw this.terminationError ?? new TransportTerminatedError("transport is terminated");
		if (this.unmounted) throw new SurfaceClosedError(this.surfaceId);
		const requestId = this.commandClient.allocateRequestId(nextU32);
		const command = {
			type: "command",
			surfaceId: this.surfaceId,
			epoch: this.epoch,
			afterRevision: this.revision,
			requestId,
			nodeId: 1,
			command: kind,
			payload
		};
		return {
			requestId,
			result: this.commandClient.submit(command, options)
		};
	}
	cancelNative(requestId) {
		if (this.transportTerminated || this.unmounted) return;
		try {
			if (!this.submitFrame(encodeFrame({
				type: "command",
				surfaceId: this.surfaceId,
				epoch: this.epoch,
				afterRevision: this.revision,
				requestId: this.commandClient.allocateRequestId(nextU32),
				nodeId: 1,
				command: COMMAND_CANCEL_NATIVE,
				payload: {
					type: "cancel-native",
					requestId
				}
			}))) throw this.terminationError ?? /* @__PURE__ */ new Error("transport rejected cancellation");
		} catch (error) {
			this.terminate(new TransportTerminatedError("Native cancellation could not be delivered", error));
		}
	}
	invokeNative(moduleId, moduleDigest, functionId, args, options) {
		if (!(moduleId instanceof Uint8Array) || moduleId.byteLength !== 16) return Promise.reject(/* @__PURE__ */ new TypeError("native module id must contain exactly 16 bytes"));
		if (!(moduleDigest instanceof Uint8Array) || moduleDigest.byteLength !== 32) return Promise.reject(/* @__PURE__ */ new TypeError("native module digest must contain exactly 32 bytes"));
		if (typeof functionId !== "number" || !Number.isInteger(functionId) || functionId <= 0 || functionId > 4294967295) return Promise.reject(/* @__PURE__ */ new TypeError("native function id must be a positive u32"));
		if (!(args instanceof Uint8Array)) return Promise.reject(/* @__PURE__ */ new TypeError("native arguments must be a Uint8Array"));
		if (args.byteLength > 1048576) return Promise.reject(/* @__PURE__ */ new RangeError("native arguments exceed the supported size"));
		return afterRootCommit(this.tree, () => this.submitSurfaceCommandValue(COMMAND_INVOKE_NATIVE, {
			type: "invoke-native",
			moduleId,
			moduleDigest,
			functionId,
			args
		}, options)).then((value) => {
			if (value === null || value.type !== "bytes" || !(value.value instanceof Uint8Array) || value.value.byteLength > 1048576) throw new Error("native invokeNative returned an invalid byte result");
			return value.value;
		});
	}
	recordUnhandledError(error) {
		this.tree.invalid = true;
		this.unhandledError = error instanceof Error ? error : new Error(String(error));
	}
	resize(width, height) {
		if (!Number.isInteger(width) || !Number.isInteger(height) || width < 1 || width > 16384 || height < 1 || height > 16384) return Promise.reject(/* @__PURE__ */ new RangeError("window size must be integer pixels in the range 1..16384"));
		return this.submitSurfaceCommand(COMMAND_RESIZE_WINDOW, {
			type: "window-size",
			width,
			height
		});
	}
	setClosePolicy(policy) {
		if (policy !== "allow" && policy !== "require-confirmation") return Promise.reject(/* @__PURE__ */ new TypeError("close policy must be allow or require-confirmation"));
		return this.submitSurfaceCommand(COMMAND_SET_CLOSE_POLICY, {
			type: "text",
			value: policy
		});
	}
	resolveCloseRequest(requestId, allow) {
		if (!Number.isInteger(requestId) || requestId < 0 || requestId > 4294967295) return Promise.reject(/* @__PURE__ */ new RangeError("close request id must be a u32"));
		if (typeof allow !== "boolean") return Promise.reject(/* @__PURE__ */ new TypeError("close request allow must be a boolean"));
		return this.submitSurfaceCommand(COMMAND_RESOLVE_CLOSE_REQUEST, {
			type: "close-resolution",
			requestId,
			allow
		});
	}
	zoom() {
		return this.submitSurfaceCommand(COMMAND_ZOOM_WINDOW, null);
	}
	toggleFullscreen() {
		return this.submitSurfaceCommand(COMMAND_TOGGLE_FULLSCREEN, null);
	}
	openUrl(url) {
		if (typeof url !== "string" || utf8ByteLength(url) > 2048 || /\s/.test(url) || !url.startsWith("http://") && !url.startsWith("https://") || url.slice(url.indexOf("://") + 3).length === 0) return Promise.reject(/* @__PURE__ */ new TypeError("url must be a non-empty http or https URL of at most 2048 UTF-8 bytes"));
		return this.submitSurfaceCommand(COMMAND_OPEN_URL, {
			type: "text",
			value: url
		});
	}
	openSurface(options = {}) {
		const title = options.title ?? "";
		const width = options.width ?? 0;
		const height = options.height ?? 0;
		if (typeof title !== "string" || [...title].length > 256) return Promise.reject(/* @__PURE__ */ new TypeError("surface title must be at most 256 characters"));
		if (!Number.isInteger(width) || !Number.isInteger(height) || width < 0 || width > 16384 || height < 0 || height > 16384 || width === 0 !== (height === 0)) return Promise.reject(/* @__PURE__ */ new RangeError("surface size must be 0x0 or integer pixels in the range 1..16384"));
		if (options.kind !== void 0 && ![
			"normal",
			"floating",
			"dialog"
		].includes(options.kind)) return Promise.reject(/* @__PURE__ */ new TypeError("surface kind must be normal, floating, or dialog"));
		if (options.resizable !== void 0 && typeof options.resizable !== "boolean") return Promise.reject(/* @__PURE__ */ new TypeError("surface resizable must be a boolean"));
		let minWidth = null;
		let minHeight = null;
		if (options.minSize !== void 0) {
			if (!Array.isArray(options.minSize) || options.minSize.length !== 2 || !Number.isInteger(options.minSize[0]) || !Number.isInteger(options.minSize[1]) || options.minSize[0] <= 0 || options.minSize[1] <= 0 || options.minSize[0] > 16384 || options.minSize[1] > 16384) return Promise.reject(/* @__PURE__ */ new RangeError("surface minSize must be positive integer pixels in the range 1..16384"));
			[minWidth, minHeight] = options.minSize;
		}
		const hasOptions = options.kind !== void 0 || options.resizable !== void 0 || options.minSize !== void 0;
		const kindCode = options.kind === void 0 ? null : {
			normal: 0,
			floating: 1,
			dialog: 2
		}[options.kind];
		const windowOptions = hasOptions ? {
			kind: kindCode,
			resizable: options.resizable ?? null,
			minWidth,
			minHeight
		} : void 0;
		const payload = {
			type: "open-surface",
			title,
			width,
			height,
			...windowOptions === void 0 ? {} : { options: windowOptions }
		};
		return this.submitSurfaceCommandValue(COMMAND_OPEN_SURFACE, payload).then((value) => {
			if (value === null || value.type !== "number" || !Number.isInteger(value.value) || value.value < 1 || value.value > 4294967295) throw new Error("native openSurface returned an invalid surface id");
			return value.value;
		});
	}
	pickFiles(options = {}) {
		const title = options.title ?? "";
		const directories = options.directories ?? false;
		const multiple = options.multiple ?? false;
		if (typeof title !== "string" || [...title].length > 256) return Promise.reject(/* @__PURE__ */ new TypeError("file dialog title must be at most 256 characters"));
		if (typeof directories !== "boolean" || typeof multiple !== "boolean") return Promise.reject(/* @__PURE__ */ new TypeError("file dialog options must be boolean"));
		const payload = {
			type: "file-dialog-open",
			title,
			directories,
			multiple
		};
		return this.submitSurfaceCommandValue(COMMAND_FILE_DIALOG_OPEN, payload).then((value) => {
			if (value === null) return null;
			if (value.type !== "paths" || value.paths.length === 0 || !value.paths.every((path) => path.length > 0)) throw new Error("native pickFiles returned an invalid value");
			return [...value.paths];
		});
	}
	pickSavePath(options = {}) {
		const defaultName = options.defaultName ?? "";
		if (typeof defaultName !== "string" || [...defaultName].length > 256) return Promise.reject(/* @__PURE__ */ new TypeError("save dialog defaultName must be at most 256 characters"));
		return this.submitSurfaceCommandValue(COMMAND_FILE_DIALOG_SAVE, {
			type: "text",
			value: defaultName
		}).then((value) => {
			if (value === null || value.type !== "text" || value.value.length === 0) throw new Error("native pickSavePath returned an invalid value");
			return value.value;
		});
	}
	readTextFile(path) {
		if (typeof path !== "string" || path.length === 0 || utf8ByteLength(path) > 1024 || /[\u0000-\u001f\u007f]/.test(path) || !path.startsWith("/")) return Promise.reject(/* @__PURE__ */ new TypeError("file path must be a non-empty absolute path of at most 1024 UTF-8 bytes"));
		return this.submitSurfaceCommandValue(COMMAND_READ_TEXT_FILE, {
			type: "text",
			value: path
		}).then((value) => {
			if (value === null || value.type !== "file-text") throw new Error("native readTextFile returned an invalid value");
			return value.value;
		});
	}
	loadFont(path) {
		if (typeof path !== "string" || path.length === 0 || utf8ByteLength(path) > 1024 || /[\u0000-\u001f\u007f]/.test(path) || !path.startsWith("/")) return Promise.reject(/* @__PURE__ */ new TypeError("font path must be a non-empty absolute path of at most 1024 UTF-8 bytes"));
		return this.submitSurfaceCommandValue(COMMAND_LOAD_FONT, {
			type: "text",
			value: path
		}).then((value) => {
			if (value === null || value.type !== "text" || value.value.length === 0) throw new Error("native loadFont returned an invalid family name");
			return value.value;
		});
	}
	writeTextFile(path, content) {
		if (typeof path !== "string" || path.length === 0 || utf8ByteLength(path) > 1024 || /[\u0000-\u001f\u007f]/.test(path) || !path.startsWith("/")) return Promise.reject(/* @__PURE__ */ new TypeError("file path must be a non-empty absolute path of at most 1024 UTF-8 bytes"));
		if (typeof content !== "string" || utf8ByteLength(content) > 16776192) return Promise.reject(/* @__PURE__ */ new RangeError("file content exceeds the supported size"));
		return this.submitSurfaceCommandValue(COMMAND_WRITE_TEXT_FILE, {
			type: "file-write",
			path,
			content
		}).then((value) => {
			if (value === null || value.type !== "number" || !Number.isInteger(value.value) || value.value < 0 || value.value > 4294967295) throw new Error("native writeTextFile returned an invalid byte count");
			return value.value;
		});
	}
	showNotification(options) {
		const { title, body, actions } = options;
		if (typeof title !== "string" || utf8ByteLength(title) > 256) return Promise.reject(/* @__PURE__ */ new TypeError("notification title must be at most 256 UTF-8 bytes"));
		if (typeof body !== "string" || utf8ByteLength(body) > 1024) return Promise.reject(/* @__PURE__ */ new TypeError("notification body must be at most 1024 UTF-8 bytes"));
		if (actions !== void 0 && (!Array.isArray(actions) || actions.length > 3 || actions.some((action) => typeof action.id !== "string" || action.id.length === 0 || utf8ByteLength(action.id) > 64 || typeof action.label !== "string" || action.label.length === 0 || utf8ByteLength(action.label) > 256))) return Promise.reject(/* @__PURE__ */ new TypeError("notification actions must contain at most three bounded id/label pairs"));
		const payload = {
			type: "notification",
			title,
			body,
			...actions === void 0 ? {} : { actions }
		};
		return this.submitSurfaceCommand(COMMAND_SHOW_NOTIFICATION, payload);
	}
	setMenus(menus) {
		try {
			const encodeItem = (item) => {
				if (item.type === "separator") return { type: "separator" };
				if (item.type === "action") {
					if (typeof item.name !== "string" || item.name.length === 0 || [...item.name].length > 256) throw new TypeError("menu action name must be 1..256 Unicode scalar values");
					if (item.disabled !== void 0 && typeof item.disabled !== "boolean" || item.checked !== void 0 && typeof item.checked !== "boolean") throw new TypeError("menu action disabled/checked states must be boolean");
					return {
						type: "action",
						name: item.name,
						...item.disabled === void 0 ? {} : { disabled: item.disabled },
						...item.checked === void 0 ? {} : { checked: item.checked }
					};
				}
				if (typeof item.title !== "string" || item.title.length === 0 || [...item.title].length > 256 || !Array.isArray(item.items)) throw new TypeError("submenu title must be 1..256 Unicode scalar values");
				return {
					type: "submenu",
					title: item.title,
					items: item.items.map(encodeItem)
				};
			};
			const normalizedMenus = menus.map((menu) => {
				if (typeof menu.title !== "string" || menu.title.length === 0 || [...menu.title].length > 256 || !Array.isArray(menu.items)) throw new TypeError("menu title must be 1..256 Unicode scalar values");
				return {
					title: menu.title,
					items: menu.items.map(encodeItem)
				};
			});
			return this.submitSurfaceCommand(COMMAND_SET_MENUS, {
				type: "menus",
				menus: normalizedMenus
			});
		} catch (error) {
			return Promise.reject(error);
		}
	}
	setKeybindings(bindings) {
		try {
			if (!Array.isArray(bindings) || bindings.length > 64) throw new RangeError("setKeybindings accepts at most 64 bindings");
			const normalizedBindings = bindings.map((binding, index) => {
				if (binding === null || typeof binding !== "object" || typeof binding.keystrokes !== "string" || binding.keystrokes.trim().length === 0 || utf8ByteLength(binding.keystrokes) > 64 || /[\u0000-\u001f\u007f]/.test(binding.keystrokes)) throw new TypeError(`keybinding[${index}].keystrokes must be a non-empty string of at most 64 UTF-8 bytes`);
				if (typeof binding.actionName !== "string" || binding.actionName.length === 0 || [...binding.actionName].length > 64 || /[\u0000-\u001f\u007f]/.test(binding.actionName)) throw new TypeError(`keybinding[${index}].actionName must be 1..64 Unicode characters`);
				return {
					keystrokes: binding.keystrokes,
					actionName: binding.actionName
				};
			});
			return this.submitSurfaceCommand(COMMAND_SET_KEYBINDINGS, {
				type: "keybindings",
				bindings: normalizedBindings
			});
		} catch (error) {
			return Promise.reject(error);
		}
	}
	focusNext() {
		return this.submitSurfaceCommand(COMMAND_FOCUS_NEXT, null);
	}
	focusPrev() {
		return this.submitSurfaceCommand(COMMAND_FOCUS_PREV, null);
	}
	getWindowSize() {
		return this.submitSurfaceCommandValue(COMMAND_GET_WINDOW_SIZE, null).then((value) => {
			if (value === null || value.type !== "pair" || value.width < 0 || value.height < 0) throw new Error("native getWindowSize returned an invalid value");
			return [value.width, value.height];
		});
	}
	minimizeWindow() {
		return this.submitSurfaceCommand(COMMAND_MINIMIZE_WINDOW, null);
	}
	getWindowBounds() {
		return this.submitSurfaceCommandValue(COMMAND_GET_WINDOW_BOUNDS, null).then((value) => {
			if (value === null || value.type !== "bounds" || value.width < 0 || value.height < 0) throw new Error("native getWindowBounds returned an invalid value");
			return {
				x: value.x,
				y: value.y,
				width: value.width,
				height: value.height
			};
		});
	}
	getWindowState() {
		return this.submitSurfaceCommandValue(COMMAND_GET_WINDOW_STATE, null).then((value) => {
			if (value === null || value.type !== "window-state") throw new Error("native getWindowState returned an invalid value");
			return {
				fullscreen: value.fullscreen,
				maximized: value.maximized
			};
		});
	}
	activateWindow() {
		return this.submitSurfaceCommand(COMMAND_ACTIVATE_WINDOW, null);
	}
	setClipboardText(text) {
		if (typeof text !== "string") return Promise.reject(/* @__PURE__ */ new TypeError("clipboard text must be a string"));
		if (utf8ByteLength(text) > 1048576) return Promise.reject(/* @__PURE__ */ new RangeError("clipboard text exceeds the supported size"));
		return this.submitSurfaceCommand(COMMAND_CLIPBOARD_WRITE, {
			type: "text",
			value: text
		});
	}
	getClipboardText() {
		return this.submitSurfaceCommandValue(COMMAND_CLIPBOARD_READ, null).then((value) => {
			if (value === null || value.type !== "text" || utf8ByteLength(value.value) > 1048576) throw new Error("native getClipboardText returned an invalid value");
			return value.value;
		});
	}
	setClipboardImage(image) {
		try {
			const normalized = normalizeClipboardImage(image);
			return this.submitSurfaceCommand(COMMAND_CLIPBOARD_WRITE_IMAGE, {
				type: "clipboard-image",
				image: normalized
			});
		} catch (error) {
			return Promise.reject(error);
		}
	}
	getClipboardImage() {
		return this.submitSurfaceCommandValue(COMMAND_CLIPBOARD_READ_IMAGE, null).then((value) => {
			if (value === null) return null;
			if (value.type !== "image") throw new Error("native getClipboardImage returned an invalid value");
			return value.image;
		});
	}
	throwIfUnhandledError() {
		if (this.unhandledError !== void 0) {
			const error = this.unhandledError;
			this.unhandledError = void 0;
			throw error;
		}
	}
	receiveEvents(events) {
		if (this.transportTerminated || this.unmounted || events.length === 0) return;
		this.scheduleDispatch(() => {
			for (const event of events) dispatchEvent(this, event);
		});
	}
	onSurfaceClosed() {
		if (this.unmounted) return;
		this.dispose(new SurfaceClosedError(this.surfaceId));
		this.surfaceClosedHandler?.();
	}
	dispose(error = new SurfaceClosedError(this.surfaceId)) {
		if (this.unmounted) return;
		this.unmounted = true;
		this.detachEventSubscription();
		this.commandClient.rejectAll(error);
		this.tree.dispose();
	}
	acceptEvent(event) {
		if (this.unmounted || this.transportTerminated || event.surfaceId !== this.surfaceId || event.epoch !== this.epoch || event.revision > this.revision || this.hasEventSequence && event.sequence <= this.lastEventSequence) return false;
		this.lastEventSequence = event.sequence;
		this.hasEventSequence = true;
		return true;
	}
	findListener(listenerId, revision) {
		return this.tree.findListener(listenerId, revision);
	}
	resolveCommandResult(result) {
		this.commandClient.resolve(result);
	}
};
//#endregion
//#region packages/solid-gpui/src/renderer/surface-router.ts
var SurfaceRouter = class {
	transport;
	options;
	decoder;
	routes = /* @__PURE__ */ new Map();
	onTermination;
	unsubscribeData;
	unsubscribeTermination;
	unsubscribeDrain;
	pressured = false;
	output = [];
	outputBytes = 0;
	input = [];
	inputBytes = 0;
	started = false;
	terminated = false;
	disposed = false;
	terminationError;
	constructor(transport, options = {}) {
		this.transport = transport;
		this.options = options;
		this.decoder = new FrameDecoder(options.maxFrameSize ?? 16777216);
		this.onTermination = options.onTermination;
	}
	start() {
		if (this.started || this.disposed) return;
		this.started = true;
		this.unsubscribeDrain = this.transport.onDrain(() => this.drain());
		const unsubscribeData = this.transport.onData((chunk) => this.receive(chunk));
		this.unsubscribeData = unsubscribeData;
		if (this.terminated) {
			unsubscribeData();
			this.unsubscribeData = void 0;
			return;
		}
		const unsubscribeTermination = this.transport.onTermination((error) => this.terminate(error));
		this.unsubscribeTermination = unsubscribeTermination;
		if (this.terminated) {
			unsubscribeTermination();
			this.unsubscribeTermination = void 0;
		}
	}
	register(surfaceId, route) {
		if (this.terminated || this.disposed) {
			try {
				route.terminate(this.terminationError ?? new TransportTerminatedError("surface router is disposed"));
			} catch {}
			return () => void 0;
		}
		if (this.routes.has(surfaceId)) throw new Error(`surface ${surfaceId} is already registered`);
		this.routes.set(surfaceId, route);
		return () => {
			if (this.routes.get(surfaceId) === route) this.routes.delete(surfaceId);
		};
	}
	submit(frame) {
		if (this.terminated) throw this.terminationError;
		if (this.disposed) throw new Error("SurfaceRouter is disposed");
		try {
			if (this.pressured || this.output.length) {
				if (this.output.length >= 4096 || frame.length > 16777220 - this.outputBytes) throw new RangeError("renderer pending output capacity exceeded");
				this.output.push(frame.slice());
				this.outputBytes += frame.length;
			} else this.pressured = !this.transport.submit(frame);
		} catch (error) {
			const termination = error instanceof TransportTerminatedError ? error : new TransportTerminatedError(`surface router output failed: ${error instanceof Error ? error.message : String(error)}`, error);
			this.terminate(termination);
			throw termination;
		}
	}
	drain() {
		if (this.disposed || this.terminated) return;
		this.pressured = false;
		try {
			while (!this.pressured && this.output.length) {
				const frame = this.output.shift();
				this.outputBytes -= frame.length;
				this.pressured = !this.transport.submit(frame);
			}
			while (!this.pressured && this.input.length) {
				const frame = this.input.shift();
				this.inputBytes -= frame.length;
				this.receive(frame);
			}
		} catch (error) {
			this.terminate(new TransportTerminatedError("renderer drain failed", error));
		}
	}
	terminate(error) {
		if (this.terminated || this.disposed) return;
		this.terminated = true;
		this.terminationError = error;
		this.detachTransport();
		const routes = [...this.routes.values()];
		this.routes.clear();
		for (const route of routes) try {
			route.terminate(error);
		} catch {}
		try {
			this.onTermination?.(error);
		} catch {}
	}
	dispose() {
		if (this.disposed) return;
		this.disposed = true;
		this.detachTransport();
		this.routes.clear();
	}
	receive(chunk) {
		if (this.terminated || this.disposed) return;
		if (this.pressured) {
			const bytes = chunk instanceof Uint8Array ? chunk : new Uint8Array(chunk);
			if (this.input.length >= 4096 || bytes.length > 16777220 - this.inputBytes) {
				this.terminate(new TransportTerminatedError("renderer pending input capacity exceeded"));
				return;
			}
			this.input.push(bytes.slice());
			this.inputBytes += bytes.length;
			return;
		}
		let payloads;
		try {
			payloads = this.decoder.push(chunk);
		} catch (error) {
			this.terminate(new TransportTerminatedError(`SurfaceRouter protocol failure: ${error instanceof Error ? error.message : String(error)}`, {
				kind: "protocol",
				detail: error instanceof Error ? error.message : String(error)
			}));
			return;
		}
		if (payloads.length === 0) return;
		const decoded = [];
		for (const payload of payloads) {
			let event;
			try {
				event = decodeEvent(payload);
			} catch (error) {
				this.terminate(new TransportTerminatedError(`SurfaceRouter event decode failed: ${error instanceof Error ? error.message : String(error)}`, {
					kind: "protocol",
					detail: error instanceof Error ? error.message : String(error)
				}));
				return;
			}
			if (event === null) {
				this.terminate(new TransportTerminatedError("SurfaceRouter received malformed event frame", {
					kind: "protocol",
					detail: "malformed event frame"
				}));
				return;
			}
			decoded.push(event);
		}
		const bySurface = /* @__PURE__ */ new Map();
		const flush = () => {
			for (const [surfaceId, events] of bySurface) this.routes.get(surfaceId)?.deliver(events);
			bySurface.clear();
		};
		for (const event of decoded) if (event.payload.type === "application-activation") {
			flush();
			if (this.disposed || this.terminated) return;
			try {
				this.options.onApplicationActivation?.(event);
			} catch (error) {
				this.terminate(new TransportTerminatedError("application activation failed", error));
				return;
			}
		} else {
			const events = bySurface.get(event.surfaceId);
			if (events === void 0) bySurface.set(event.surfaceId, [event]);
			else events.push(event);
		}
		flush();
	}
	detachTransport() {
		this.unsubscribeDrain?.();
		this.unsubscribeDrain = void 0;
		this.output = [];
		this.input = [];
		this.outputBytes = this.inputBytes = 0;
		const unsubscribeData = this.unsubscribeData;
		this.unsubscribeData = void 0;
		unsubscribeData?.();
		const unsubscribeTermination = this.unsubscribeTermination;
		this.unsubscribeTermination = void 0;
		unsubscribeTermination?.();
	}
};
//#endregion
//#region packages/solid-gpui/src/renderer.ts
var solidRenderer = createRenderer(hostConfig);
var surfaceContexts = /* @__PURE__ */ new WeakMap();
function normalizeRefProps(props) {
	const ref = props.ref;
	if (ref == null || ref === false) return props;
	if (typeof ref === "function" || Array.isArray(ref)) {
		const callback = (node) => applyRef(ref, node);
		return new Proxy(props, { get(target, property, receiver) {
			return property === "ref" ? callback : Reflect.get(target, property, receiver);
		} });
	}
	if (typeof ref === "object" && ref !== null && "current" in ref) {
		const callback = (node) => {
			ref.current = node;
		};
		return new Proxy(props, { get(target, property, receiver) {
			return property === "ref" ? callback : Reflect.get(target, property, receiver);
		} });
	}
	throw new TypeError("Solid GPUI ref must be a callback, callback array, or mutable ref object");
}
function createHostElement(type, props = {}) {
	const node = solidRenderer.createElement(type);
	solidRenderer.spread(node, normalizeRefProps(props));
	return node;
}
var nextSurfaceId = 1;
function unwrapElement(value) {
	return typeof value === "function" ? value() : value;
}
function createRoot(transport, options = {}) {
	return createRootWithRouter(new SurfaceRouter(transport, { maxFrameSize: options.maxFrameSize }), options, true);
}
function createRootWithRouter(router, options = {}, startRouter = false) {
	assertSolidRuntime();
	const surfaceId = options.surfaceId ?? nextSurfaceId++;
	const epoch = options.epoch ?? generationHost()?.epoch ?? 1;
	let closed = false;
	let setElement;
	let disposeRender;
	let solidDisposed = false;
	let container;
	const disposeSolid = () => {
		if (solidDisposed) return;
		solidDisposed = true;
		try {
			disposeRender?.();
		} finally {
			disposeRender = void 0;
			if (startRouter) router.dispose();
		}
	};
	const scheduleDispatch = (dispatch) => {
		if (closed) return;
		if (!cancelScheduledCommit(container.tree)) container.tree.beginRender();
		try {
			withRootTransaction(container.tree, dispatch);
		} catch (error) {
			container.recordUnhandledError(error);
		} finally {
			if (!container.unmounted) container.tree.commit();
		}
		container.throwIfUnhandledError();
	};
	container = new RootContainer({
		surfaceId,
		epoch,
		scheduleDispatch,
		submitFrame: (frame) => {
			router.submit(frame);
			return true;
		},
		onTransportTermination: (error) => {
			if (!closed) {
				closed = true;
				cancelScheduledCommit(container.tree);
				container.dispose(error);
				disposeSolid();
			}
			options.onTransportTermination?.(error);
		},
		onWindowResize: options.onWindowResize,
		onWindowActivation: options.onWindowActivation,
		onSurfaceClosed: () => {
			if (closed) return;
			closed = true;
			disposeSolid();
			options.onClose?.();
		},
		onCloseRequested: options.onCloseRequested,
		onAction: options.onAction,
		onAppearance: options.onAppearance,
		onNotificationResponse: options.onNotificationResponse
	});
	const unregister = router.register(surfaceId, {
		deliver(events) {
			container.receiveEvents(events);
		},
		terminate(error) {
			container.terminate(error);
		}
	});
	container.start(unregister);
	const children = /* @__PURE__ */ new Set();
	surfaceContexts.set(container.tree, {
		router,
		container,
		children
	});
	if (startRouter) router.start();
	if (!closed) createRoot$1((dispose) => {
		disposeRender = dispose;
		bindRootOwner(container.tree);
		withRoot(container.tree, () => {
			const [element, set] = createSignal(null, { equals: false });
			setElement = set;
			solidRenderer.insert(container.tree.syntheticRoot, () => withRoot(container.tree, () => unwrapElement(element())));
		});
	});
	return {
		invokeNative(moduleId, moduleDigest, functionId, args, options) {
			if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
			return container.invokeNative(moduleId, moduleDigest, functionId, args, options);
		},
		render(element) {
			if (closed) throw new SurfaceClosedError(surfaceId);
			if (!cancelScheduledCommit(container.tree)) container.tree.beginRender();
			try {
				withRootTransaction(container.tree, () => setElement(() => element));
			} catch (error) {
				container.recordUnhandledError(error);
			} finally {
				container.tree.commit();
			}
			container.throwIfUnhandledError();
		},
		setTitle(title) {
			if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
			return container.setTitle(title);
		},
		resize(width, height) {
			if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
			return container.resize(width, height);
		},
		getWindowSize() {
			if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
			return container.getWindowSize();
		},
		minimizeWindow() {
			if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
			return container.minimizeWindow();
		},
		getWindowBounds() {
			if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
			return container.getWindowBounds();
		},
		getWindowState() {
			if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
			return container.getWindowState();
		},
		activateWindow() {
			if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
			return container.activateWindow();
		},
		setClipboardText(text) {
			if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
			return container.setClipboardText(text);
		},
		getClipboardText() {
			if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
			return container.getClipboardText();
		},
		setClipboardImage(image) {
			if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
			return container.setClipboardImage(image);
		},
		getClipboardImage() {
			if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
			return container.getClipboardImage();
		},
		zoom() {
			if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
			return container.zoom();
		},
		toggleFullscreen() {
			if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
			return container.toggleFullscreen();
		},
		setClosePolicy(policy) {
			if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
			return container.setClosePolicy(policy);
		},
		resolveCloseRequest(requestId, allow) {
			if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
			return container.resolveCloseRequest(requestId, allow);
		},
		openSurface(options) {
			if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
			return container.openSurface(options);
		},
		pickFiles(options) {
			if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
			return container.pickFiles(options);
		},
		pickSavePath(options) {
			if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
			return container.pickSavePath(options);
		},
		readTextFile(path) {
			if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
			return container.readTextFile(path);
		},
		loadFont(path) {
			if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
			return container.loadFont(path);
		},
		writeTextFile(path, content) {
			if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
			return container.writeTextFile(path, content);
		},
		showNotification(options) {
			if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
			return container.showNotification(options);
		},
		setMenus(menus) {
			if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
			return container.setMenus(menus);
		},
		setKeybindings(bindings) {
			if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
			return container.setKeybindings(bindings);
		},
		openUrl(url) {
			if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
			return container.openUrl(url);
		},
		focusNext() {
			if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
			return container.focusNext();
		},
		focusPrev() {
			if (closed) return Promise.reject(new SurfaceClosedError(surfaceId));
			return container.focusPrev();
		},
		unmount() {
			if (closed) return;
			for (const dispose of children) dispose();
			children.clear();
			closed = true;
			cancelScheduledCommit(container.tree);
			container.dispose();
			disposeSolid();
		}
	};
}
solidRenderer.render;
solidRenderer.effect;
var memo = solidRenderer.memo;
var createUniversalComponent = createComponent$1;
function createComponent(component, props) {
	return createUniversalComponent(component, props);
}
solidRenderer.createElement;
solidRenderer.createTextNode;
solidRenderer.insertNode;
solidRenderer.insert;
solidRenderer.spread;
solidRenderer.setProp;
solidRenderer.mergeProps;
solidRenderer.use;
var For = For$1;
//#endregion
//#region packages/solid-gpui/src/hooks.ts
/** Create the explicit store wired to a root's `onWindowResize` callback. */
function createWindowSizeStore(initial = {
	width: 1024,
	height: 720,
	scaleFactor: 1
}) {
	let snapshot = {
		width: initial.width,
		height: initial.height,
		scaleFactor: initial.scaleFactor ?? 1
	};
	const listeners = /* @__PURE__ */ new Set();
	return {
		getSnapshot: () => snapshot,
		subscribe(listener) {
			listeners.add(listener);
			return () => listeners.delete(listener);
		},
		set(width, height, scaleFactor = 1) {
			if (snapshot.width === width && snapshot.height === height && snapshot.scaleFactor === scaleFactor) return;
			snapshot = {
				width,
				height,
				scaleFactor
			};
			for (const listener of listeners) listener();
		}
	};
}
/** Subscribe to an explicit window-size store from a Solid component owner. */
function useWindowSize(store) {
	const [snapshot, setSnapshot] = createSignal(store.getSnapshot());
	onCleanup(store.subscribe(() => setSnapshot(store.getSnapshot())));
	return snapshot;
}
//#endregion
//#region packages/solid-gpui/src/index.ts
function hostComponent(kind) {
	return (props) => createHostElement(kind, props);
}
var View = hostComponent("View");
var TextInput = hostComponent("TextInput");
var Text = hostComponent("Text");
var Pressable = hostComponent("Pressable");
function VirtualList(props) {
	const dataSnapshot = createMemo(() => props.data.slice());
	const [range, setRange] = createSignal([0, Math.min(props.data.length, props.initialNumToRender ?? 10)]);
	let endReached = false;
	const onVisibleRange = (start, end) => {
		const nextStart = Math.max(0, Math.min(start, props.data.length));
		const nextEnd = Math.max(nextStart, Math.min(end, props.data.length));
		setRange([nextStart, nextEnd]);
		if (nextEnd >= props.data.length) {
			if (!endReached) {
				endReached = true;
				props.onEndReached?.();
			}
		} else endReached = false;
	};
	const committedRange = createMemo(() => {
		const count = props.data.length;
		const [start, end] = range();
		if (start >= count || end <= start) return [0, Math.min(count, props.initialNumToRender ?? 10)];
		return [start, Math.min(end, count)];
	});
	let previousRows = /* @__PURE__ */ new Map();
	const visibleChildren = mapArray(createMemo(() => {
		const [start, end] = committedRange();
		const rows = [];
		const nextRows = /* @__PURE__ */ new Map();
		const keys = /* @__PURE__ */ new Set();
		for (let index = start; index < Math.min(end, props.data.length); index += 1) {
			const item = props.data[index];
			if (item === void 0) continue;
			const key = props.itemKey(item, index);
			if (typeof key !== "string" && typeof key !== "number") throw new TypeError("VirtualList itemKey must return a string or number");
			const normalized = String(key);
			if (keys.has(normalized)) throw new TypeError(`VirtualList itemKey must be unique in committed range (duplicate ${normalized})`);
			keys.add(normalized);
			const previous = previousRows.get(normalized);
			const row = previous && Object.is(previous.item, item) && previous.index === index ? previous : {
				key: normalized,
				item,
				index
			};
			rows.push(row);
			nextRows.set(normalized, row);
		}
		previousRows = nextRows;
		return rows;
	}), (row) => createHostElement("View", {
		key: row.key,
		children: props.renderItem(row.item, row.index)
	}));
	const node = createHostElement("VirtualList", {
		get accessibilityLabel() {
			return props.accessibilityLabel;
		},
		get accessibilityRole() {
			return props.accessibilityRole;
		},
		get accessibilityDescription() {
			return props.accessibilityDescription;
		},
		get accessibilityDisabled() {
			return props.accessibilityDisabled;
		},
		get accessibilityChecked() {
			return props.accessibilityChecked;
		},
		get accessibilitySelected() {
			return props.accessibilitySelected;
		},
		get accessibilityValue() {
			return props.accessibilityValue;
		},
		get accessibilityExpanded() {
			return props.accessibilityExpanded;
		},
		get accessibilityLevel() {
			return props.accessibilityLevel;
		},
		get accessibilityLive() {
			return props.accessibilityLive;
		},
		get style() {
			return props.style;
		},
		get ref() {
			return props.ref;
		},
		get __itemCount() {
			return props.data.length;
		},
		get __data() {
			return dataSnapshot();
		},
		get __rangeStart() {
			return committedRange()[0];
		},
		get __rangeEnd() {
			return committedRange()[1];
		},
		get __estimatedItemSize() {
			return props.estimatedItemSize;
		},
		get __overscan() {
			return props.overscan ?? 2;
		},
		__onVisibleRange: onVisibleRange,
		get children() {
			return visibleChildren();
		}
	});
	return () => {
		if (props.data.length === 0) {
			endReached = false;
			return props.emptyState ?? null;
		}
		return node;
	};
}
//#endregion
//#region node_modules/.bun/@tanstack+router-core@1.171.28/node_modules/@tanstack/router-core/dist/esm/utils.js
function isAbsoluteUrl(url) {
	if (URL.canParse) return URL.canParse(url);
	try {
		new URL(url);
		return true;
	} catch {
		return false;
	}
}
/**
* Return the last element of an array.
* Intended for non-empty arrays used within router internals.
*/
function last(arr) {
	return arr[arr.length - 1];
}
/**
* Apply a value-or-updater to a previous value.
* Accepts either a literal value or a function of the previous value.
*/
function functionalUpdate(updater, previous) {
	if (typeof updater === "function") return updater(previous);
	return updater;
}
var hasOwn = Object.prototype.hasOwnProperty;
var isEnumerable = Object.prototype.propertyIsEnumerable;
function hasKeys(obj) {
	for (const key in obj) if (hasOwn.call(obj, key)) return true;
	return false;
}
var createNull = () => Object.create(null);
var nullReplaceEqualDeep = (prev, next) => replaceEqualDeep(prev, next, createNull);
/**
* This function returns `prev` if `_next` is deeply equal.
* If not, it will replace any deeply equal children of `b` with those of `a`.
* This can be used for structural sharing between immutable JSON values for example.
* Do not use this with signals
*/
function replaceEqualDeep(prev, _next, _makeObj = () => ({}), _depth = 0) {
	if (prev === _next) return prev;
	if (_depth > 500) return _next;
	const next = _next;
	const array = isPlainArray(prev) && isPlainArray(next);
	if (!array && !(isPlainObject(prev) && isPlainObject(next))) return next;
	const prevItems = array ? prev : getEnumerableOwnKeys(prev);
	if (!prevItems) return next;
	const nextItems = array ? next : getEnumerableOwnKeys(next);
	if (!nextItems) return next;
	const prevSize = prevItems.length;
	const nextSize = nextItems.length;
	const copy = array ? new Array(nextSize) : _makeObj();
	let equalItems = 0;
	for (let i = 0; i < nextSize; i++) {
		const key = array ? i : nextItems[i];
		const p = prev[key];
		const n = next[key];
		if (p === n) {
			copy[key] = p;
			if (array ? i < prevSize : hasOwn.call(prev, key)) equalItems++;
			continue;
		}
		if (p === null || n === null || typeof p !== "object" || typeof n !== "object") {
			copy[key] = n;
			continue;
		}
		const v = replaceEqualDeep(p, n, _makeObj, _depth + 1);
		copy[key] = v;
		if (v === p) equalItems++;
	}
	return prevSize === nextSize && equalItems === prevSize ? prev : copy;
}
/**
* Equivalent to `Reflect.ownKeys`, but ensures that objects are "clone-friendly":
* will return false if object has any non-enumerable properties.
*
* Optimized for the common case where objects have no symbol properties.
*/
function getEnumerableOwnKeys(o) {
	const keys = Object.keys(o);
	if (keys.length !== Object.getOwnPropertyNames(o).length) return false;
	const symbols = Object.getOwnPropertySymbols(o);
	if (symbols.length === 0) return keys;
	for (const symbol of symbols) {
		if (!isEnumerable.call(o, symbol)) return false;
		keys.push(symbol);
	}
	return keys;
}
function isPlainObject(o) {
	if (!hasObjectPrototype(o)) return false;
	const ctor = o.constructor;
	if (typeof ctor === "undefined") return true;
	const prot = ctor.prototype;
	if (!hasObjectPrototype(prot)) return false;
	if (!prot.hasOwnProperty("isPrototypeOf")) return false;
	return true;
}
function hasObjectPrototype(o) {
	return Object.prototype.toString.call(o) === "[object Object]";
}
/**
* Check if a value is a "plain" array (no extra enumerable keys).
*/
function isPlainArray(value) {
	return Array.isArray(value) && value.length === Object.keys(value).length;
}
/**
* Perform a deep equality check with options for partial comparison and
* ignoring `undefined` values. Optimized for router state comparisons.
*/
function deepEqual(a, b, opts) {
	if (a === b) return true;
	if (typeof a !== typeof b) return false;
	if (Array.isArray(a) && Array.isArray(b)) {
		if (a.length !== b.length) return false;
		for (let i = 0, l = a.length; i < l; i++) if (!deepEqual(a[i], b[i], opts)) return false;
		return true;
	}
	if (isPlainObject(a) && isPlainObject(b)) {
		const ignoreUndefined = opts?.ignoreUndefined ?? true;
		if (opts?.partial) {
			for (const k in b) if (!ignoreUndefined || b[k] !== void 0) {
				if (!deepEqual(a[k], b[k], opts)) return false;
			}
			return true;
		}
		let aCount = 0;
		if (!ignoreUndefined) aCount = Object.keys(a).length;
		else for (const k in a) if (a[k] !== void 0) aCount++;
		let bCount = 0;
		for (const k in b) if (!ignoreUndefined || b[k] !== void 0) {
			bCount++;
			if (bCount > aCount || !deepEqual(a[k], b[k], opts)) return false;
		}
		return aCount === bCount;
	}
	return false;
}
/**
* Re-encode characters that are unsafe in URL paths.
* Includes ASCII control characters (0x00-0x1F, 0x7F) and a subset of the
* WHATWG URL "path percent-encode set" (", <, >, `, {, }).
*
* Space (0x20) is intentionally excluded — decodeURI decodes %20 to space
* and the router stores decoded spaces in location.pathname. The existing
* encodePathLikeUrl already handles re-encoding spaces for outgoing URLs.
*
* These characters are decoded by decodeURI but must remain percent-encoded
* in paths to match how upstream layers (CDNs, edge middleware, browsers)
* interpret the URL, preventing infinite redirect loops and path mismatches.
*/
var PATH_UNSAFE_RE = /[\x00-\x1f\x7f"<>`{}]/g;
function sanitizePathSegment(segment) {
	return segment.replace(PATH_UNSAFE_RE, (ch) => "%" + ch.charCodeAt(0).toString(16).toUpperCase().padStart(2, "0"));
}
function decodeSegment(segment) {
	let decoded;
	try {
		decoded = decodeURI(segment);
	} catch {
		decoded = segment.replaceAll(/%[0-9A-F]{2}/gi, (match) => {
			try {
				return decodeURI(match);
			} catch {
				return match;
			}
		});
	}
	return sanitizePathSegment(decoded);
}
/**
* Default list of URL protocols to allow in links, redirects, and navigation.
* Any absolute URL protocol not in this list is treated as dangerous by default.
*/
var DEFAULT_PROTOCOL_ALLOWLIST = [
	"http:",
	"https:",
	"mailto:",
	"tel:"
];
/**
* Check if a URL string uses a protocol that is not in the allowlist.
* Returns true for blocked protocols like javascript:, blob:, data:, etc.
*
* The URL constructor correctly normalizes:
* - Mixed case (JavaScript: → javascript:)
* - Whitespace/control characters (java\nscript: → javascript:)
* - Leading whitespace
*
* For relative URLs (no protocol), returns false (safe).
*
* @param url - The URL string to check
* @param allowlist - Set of protocols to allow
* @returns true if the URL uses a protocol that is not allowed
*/
function isDangerousProtocol(url, allowlist) {
	if (!url) return false;
	try {
		const parsed = new URL(url);
		return !allowlist.has(parsed.protocol);
	} catch {
		return false;
	}
}
function decodePath(path) {
	if (!path) return {
		path,
		handledProtocolRelativeURL: false
	};
	if (!/[%\\\x00-\x1f\x7f]/.test(path) && !path.startsWith("//")) return {
		path,
		handledProtocolRelativeURL: false
	};
	const re = /%25|%5C/gi;
	let cursor = 0;
	let result = "";
	let match;
	while (null !== (match = re.exec(path))) {
		result += decodeSegment(path.slice(cursor, match.index)) + match[0];
		cursor = re.lastIndex;
	}
	result = result + decodeSegment(cursor ? path.slice(cursor) : path);
	let handledProtocolRelativeURL = false;
	if (result.startsWith("//")) {
		handledProtocolRelativeURL = true;
		result = "/" + result.replace(/^\/+/, "");
	}
	return {
		path: result,
		handledProtocolRelativeURL
	};
}
/**
* Encodes a path the same way `new URL()` would, but without the overhead of full URL parsing.
*
* This function encodes:
* - Whitespace characters (spaces → %20, tabs → %09, etc.)
* - Non-ASCII/Unicode characters (emojis, accented characters, etc.)
*
* It preserves:
* - Already percent-encoded sequences (won't double-encode %2F, %25, etc.)
* - ASCII special characters valid in URL paths (@, $, &, +, etc.)
* - Forward slashes as path separators
*
* Used to generate proper href values for SSR without constructing URL objects.
*
* @example
* encodePathLikeUrl('/path/file name.pdf') // '/path/file%20name.pdf'
* encodePathLikeUrl('/path/日本語') // '/path/%E6%97%A5%E6%9C%AC%E8%AA%9E'
* encodePathLikeUrl('/path/already%20encoded') // '/path/already%20encoded' (preserved)
*/
function encodePathLikeUrl(path) {
	if (!/\s|[^\u0000-\u007F]/.test(path)) return path;
	return path.replace(/\s|[^\u0000-\u007F]/gu, encodeURIComponent);
}
function arraysEqual(a, b) {
	if (a === b) return true;
	if (a.length !== b.length) return false;
	for (let i = 0; i < a.length; i++) if (a[i] !== b[i]) return false;
	return true;
}
//#endregion
//#region node_modules/.bun/@tanstack+router-core@1.171.28/node_modules/@tanstack/router-core/dist/esm/invariant.js
function invariant() {
	throw new Error("Invariant failed");
}
//#endregion
//#region node_modules/.bun/@tanstack+router-core@1.171.28/node_modules/@tanstack/router-core/dist/esm/sieve-cache.js
/**
* A fixed-capacity cache using the SIEVE eviction algorithm
* (https://cachemon.github.io/SIEVE-website/).
*
* Entries live in the Map's FIFO insertion order; a hit only flips a `visited`
* bit instead of relinking the entry, which makes `get` (by far the hottest
* operation here) one `Map.get` plus a boolean store. Eviction sweeps a `hand`
* from the oldest entry towards the newest, clearing `visited` bits until it
* finds an unvisited entry to drop, so entries touched since the last sweep
* survive one more round. This keeps LRU-like hit ratios while being
* scan-resistant.
*/
function createSieveCache(max) {
	const cache = /* @__PURE__ */ new Map();
	let hand;
	let newest;
	return {
		get(key) {
			const entry = cache.get(key);
			if (!entry) return;
			entry.visited = true;
			return entry.value;
		},
		set(key, value) {
			const existing = cache.get(key);
			if (existing) {
				existing.value = value;
				return;
			}
			if (cache.size >= max) {
				let node = hand?.next().value;
				while (!node || node.visited) {
					if (node) node.visited = false;
					else hand = cache.values();
					node = hand.next().value;
				}
				if (node === newest) hand = void 0;
				cache.delete(node.key);
			}
			const entry = {
				key,
				value,
				visited: false
			};
			newest = entry;
			cache.set(key, entry);
		},
		clear() {
			cache.clear();
			hand = void 0;
			newest = void 0;
		}
	};
}
//#endregion
//#region node_modules/.bun/@tanstack+router-core@1.171.28/node_modules/@tanstack/router-core/dist/esm/new-process-route-tree.js
var SEGMENT_TYPE_INDEX = 4;
var SEGMENT_TYPE_PATHLESS = 5;
/**
* Populates the `output` array with the parsed representation of the given `segment` string.
*
* Usage:
* ```ts
* let output
* let cursor = 0
* while (cursor < path.length) {
*   output = parseSegment(path, cursor, output)
*   const end = output[5]
*   cursor = end + 1
* ```
*
* `output` is stored outside to avoid allocations during repeated calls. It doesn't need to be typed
* or initialized, it will be done automatically.
*/
function parseSegment(path, start, output = /* @__PURE__ */ new Uint16Array(6)) {
	const next = path.indexOf("/", start);
	const end = next === -1 ? path.length : next;
	const part = path.substring(start, end);
	if (!part || !part.includes("$")) {
		output[0] = 0;
		output[1] = start;
		output[2] = start;
		output[3] = end;
		output[4] = end;
		output[5] = end;
		return output;
	}
	if (part === "$") {
		const total = path.length;
		output[0] = 2;
		output[1] = start;
		output[2] = start;
		output[3] = total;
		output[4] = total;
		output[5] = total;
		return output;
	}
	if (part.charCodeAt(0) === 36) {
		output[0] = 1;
		output[1] = start;
		output[2] = start + 1;
		output[3] = end;
		output[4] = end;
		output[5] = end;
		return output;
	}
	const openBrace = part.indexOf("{");
	let closeBrace;
	if (openBrace !== -1 && openBrace + 1 < part.length && (closeBrace = part.indexOf("}", openBrace)) !== -1) {
		const firstChar = part.charCodeAt(openBrace + 1);
		if (firstChar === 45) {
			if (openBrace + 2 < part.length && part.charCodeAt(openBrace + 2) === 36) {
				const paramStart = openBrace + 3;
				const paramEnd = closeBrace;
				if (paramStart < paramEnd) {
					output[0] = 3;
					output[1] = start + openBrace;
					output[2] = start + paramStart;
					output[3] = start + paramEnd;
					output[4] = start + closeBrace + 1;
					output[5] = end;
					return output;
				}
			}
		} else if (firstChar === 36) {
			const dollarPos = openBrace + 1;
			const afterDollar = openBrace + 2;
			if (afterDollar === closeBrace) {
				output[0] = 2;
				output[1] = start + openBrace;
				output[2] = start + dollarPos;
				output[3] = start + afterDollar;
				output[4] = start + closeBrace + 1;
				output[5] = path.length;
				return output;
			}
			output[0] = 1;
			output[1] = start + openBrace;
			output[2] = start + afterDollar;
			output[3] = start + closeBrace;
			output[4] = start + closeBrace + 1;
			output[5] = end;
			return output;
		}
	}
	output[0] = 0;
	output[1] = start;
	output[2] = start;
	output[3] = end;
	output[4] = end;
	output[5] = end;
	return output;
}
/**
* Recursively parses the segments of the given route tree and populates a segment trie.
*
* @param data A reusable Uint16Array for parsing segments. (non important, we're just avoiding allocations)
* @param route The current route to parse.
* @param start The starting index for parsing within the route's full path.
* @param node The current segment node in the trie to populate.
* @param onRoute Callback invoked for each route processed.
*/
function parseSegments(defaultCaseSensitive, data, route, start, node, depth, dynamicListsToSort, onRoute) {
	onRoute?.(route);
	let cursor = start;
	{
		const path = route.fullPath ?? route.from;
		const options = route.options;
		const length = path.length;
		const caseSensitive = options?.caseSensitive ?? defaultCaseSensitive;
		const parseParams = options?.params?.parse ?? options?.parseParams;
		while (cursor < length) {
			const segment = parseSegment(path, cursor, data);
			let nextNode;
			const start = cursor;
			const end = segment[5];
			cursor = end + 1;
			depth++;
			const kind = segment[0];
			switch (kind) {
				case 0: {
					const value = path.substring(segment[2], segment[3]);
					let name = value;
					let staticChildren;
					if (caseSensitive) staticChildren = node.static ??= /* @__PURE__ */ new Map();
					else {
						name = value.toLowerCase();
						staticChildren = node.staticInsensitive ??= /* @__PURE__ */ new Map();
					}
					const existingNode = staticChildren.get(name);
					if (existingNode) nextNode = existingNode;
					else {
						const next = createStaticNode(path);
						next.parent = node;
						next.depth = depth;
						nextNode = next;
						staticChildren.set(name, next);
					}
					break;
				}
				case 1:
				case 3:
				case 2: {
					let prefix = path.substring(start, segment[1]);
					let suffix = path.substring(segment[4], end);
					const actuallyCaseSensitive = caseSensitive && !!(prefix || suffix);
					if (!caseSensitive) {
						prefix = prefix.toLowerCase();
						suffix = suffix.toLowerCase();
					}
					const siblings = kind === 1 ? node.dynamic : kind === 3 ? node.optional : node.wildcard;
					const existingNode = kind !== 2 && !parseParams && siblings?.find((s) => !s.parse && s.caseSensitive === actuallyCaseSensitive && s.prefix === prefix && s.suffix === suffix);
					if (existingNode) nextNode = existingNode;
					else {
						const next = createDynamicNode(kind, path, actuallyCaseSensitive, prefix, suffix);
						nextNode = next;
						next.parent = node;
						next.depth = depth;
						let nodes;
						if (kind === 1) nodes = node.dynamic ??= [];
						else if (kind === 3) nodes = node.optional ??= [];
						else nodes = node.wildcard ??= [];
						nodes.push(next);
						if (nodes.length === 2) dynamicListsToSort?.push(nodes);
					}
					break;
				}
			}
			node = nextNode;
		}
		if (parseParams && route.children && !route.isRoot && route.id && route.id.charCodeAt(route.id.lastIndexOf("/") + 1) === 95) {
			const pathlessNode = createStaticNode(path);
			pathlessNode.kind = SEGMENT_TYPE_PATHLESS;
			pathlessNode.parent = node;
			depth++;
			pathlessNode.depth = depth;
			node.pathless ??= [];
			node.pathless.push(pathlessNode);
			node = pathlessNode;
		}
		const isLeaf = (route.path || !route.children) && !route.isRoot;
		if (isLeaf && path.endsWith("/")) {
			const indexNode = createStaticNode(path);
			indexNode.kind = SEGMENT_TYPE_INDEX;
			indexNode.parent = node;
			depth++;
			indexNode.depth = depth;
			node.index = indexNode;
			node = indexNode;
		}
		node.parse = parseParams ?? null;
		node.priority = options?.params?.priority ?? 0;
		if (isLeaf && !node.route) {
			node.route = route;
			node.fullPath = path;
		}
	}
	if (route.children) for (const child of route.children) parseSegments(defaultCaseSensitive, data, child, cursor, node, depth, dynamicListsToSort, onRoute);
}
function sortDynamic(a, b) {
	if (a.parse && !b.parse) return -1;
	if (!a.parse && b.parse) return 1;
	if (a.parse && b.parse && (a.priority || b.priority)) return b.priority - a.priority;
	if (a.prefix && b.prefix && a.prefix !== b.prefix) {
		if (a.prefix.startsWith(b.prefix)) return -1;
		if (b.prefix.startsWith(a.prefix)) return 1;
	}
	if (a.suffix && b.suffix && a.suffix !== b.suffix) {
		if (a.suffix.endsWith(b.suffix)) return -1;
		if (b.suffix.endsWith(a.suffix)) return 1;
	}
	if (a.prefix && !b.prefix) return -1;
	if (!a.prefix && b.prefix) return 1;
	if (a.suffix && !b.suffix) return -1;
	if (!a.suffix && b.suffix) return 1;
	if (a.caseSensitive && !b.caseSensitive) return -1;
	if (!a.caseSensitive && b.caseSensitive) return 1;
	return 0;
}
function createStaticNode(fullPath) {
	return {
		kind: 0,
		depth: 0,
		pathless: null,
		index: null,
		static: null,
		staticInsensitive: null,
		dynamic: null,
		optional: null,
		wildcard: null,
		route: null,
		fullPath,
		parent: null,
		parse: null,
		priority: 0
	};
}
/**
* Keys must be declared in the same order as in `SegmentNode` type,
* to ensure they are represented as the same object class in the engine.
*/
function createDynamicNode(kind, fullPath, caseSensitive, prefix, suffix) {
	return {
		kind,
		depth: 0,
		pathless: null,
		index: null,
		static: null,
		staticInsensitive: null,
		dynamic: null,
		optional: null,
		wildcard: null,
		route: null,
		fullPath,
		parent: null,
		parse: null,
		priority: 0,
		caseSensitive,
		prefix,
		suffix
	};
}
function processRouteMasks(routeList, processedTree) {
	const segmentTree = createStaticNode("/");
	const data = /* @__PURE__ */ new Uint16Array(6);
	const dynamicListsToSort = [];
	for (const route of routeList) parseSegments(false, data, route, 1, segmentTree, 0, dynamicListsToSort);
	for (const nodes of dynamicListsToSort) nodes.sort(sortDynamic);
	processedTree.masksTree = segmentTree;
	processedTree.flatCache = createSieveCache(1e3);
}
/**
* Take an arbitrary list of routes, create a tree from them (if it hasn't been created already), and match a path against it.
*/
function findFlatMatch(path, processedTree) {
	path ||= "/";
	const cached = processedTree.flatCache.get(path);
	if (cached !== void 0) return cached;
	const result = findMatch(path, processedTree.masksTree);
	processedTree.flatCache.set(path, result);
	return result;
}
/**
* @deprecated keep until v2 so that `router.matchRoute` can keep not caring about the actual route tree
*/
function findSingleMatch(from, caseSensitive, fuzzy, path, processedTree) {
	from ||= "/";
	path ||= "/";
	const key = caseSensitive ? `case\0${from}` : from;
	let tree = processedTree.singleCache.get(key);
	if (!tree) {
		tree = createStaticNode("/");
		parseSegments(caseSensitive, /* @__PURE__ */ new Uint16Array(6), { from }, 1, tree, 0);
		processedTree.singleCache.set(key, tree);
	}
	return findMatch(path, tree, fuzzy);
}
function findRouteMatch(path, processedTree, fuzzy = false) {
	const key = fuzzy ? path : `nofuzz\0${path}`;
	const cached = processedTree.matchCache.get(key);
	if (cached !== void 0) return cached;
	path ||= "/";
	let result;
	try {
		result = findMatch(path, processedTree.segmentTree, fuzzy);
	} catch (err) {
		if (err instanceof URIError) result = null;
		else throw err;
	}
	if (result) result.branch = buildRouteBranch(result.route);
	processedTree.matchCache.set(key, result);
	return result;
}
/** Trim trailing slashes (except preserving root '/'). */
function trimPathRight$1(path) {
	return path === "/" ? path : path.replace(/\/{1,}$/, "");
}
/**
* Processes a route tree into a segment trie for efficient path matching.
* Also builds lookup maps for routes by ID and by trimmed full path.
*/
function processRouteTree(routeTree, caseSensitive = false, initRoute) {
	const segmentTree = createStaticNode(routeTree.fullPath);
	const data = /* @__PURE__ */ new Uint16Array(6);
	const dynamicListsToSort = [];
	const routesById = {};
	const routesByPath = {};
	let index = 0;
	parseSegments(caseSensitive, data, routeTree, 1, segmentTree, 0, dynamicListsToSort, (route) => {
		initRoute?.(route, index);
		if (route.id in routesById) invariant();
		routesById[route.id] = route;
		if (index !== 0 && route.path) {
			const trimmedFullPath = trimPathRight$1(route.fullPath);
			if (!routesByPath[trimmedFullPath] || route.fullPath.endsWith("/")) routesByPath[trimmedFullPath] = route;
		}
		index++;
	});
	for (const nodes of dynamicListsToSort) nodes.sort(sortDynamic);
	return {
		processedTree: {
			segmentTree,
			singleCache: createSieveCache(1e3),
			matchCache: createSieveCache(1e3),
			flatCache: null,
			masksTree: null
		},
		routesById,
		routesByPath
	};
}
function findMatch(path, segmentTree, fuzzy = false) {
	const parts = path.split("/");
	const leaf = getNodeMatch(path, parts, segmentTree, fuzzy);
	if (!leaf) return null;
	const [rawParams] = extractParams(path, parts, leaf);
	return {
		route: leaf.node.route,
		rawParams
	};
}
/**
* This function is "resumable":
* - the `leaf` input can contain `extract` and `rawParams` properties from a previous `extractParams` call
* - the returned `state` can be passed back as `extract` in a future call to continue extracting params from where we left off
*
* Inputs are *not* mutated.
*/
function extractParams(path, parts, leaf) {
	const list = buildBranch(leaf.node);
	let nodeParts = null;
	const rawParams = Object.create(null);
	/** which segment of the path we're currently processing */
	let partIndex = leaf.extract?.part ?? 0;
	/** which node of the route tree branch we're currently processing */
	let nodeIndex = leaf.extract?.node ?? 0;
	/** index of the 1st character of the segment we're processing in the path string */
	let pathIndex = leaf.extract?.path ?? 0;
	/** which fullPath segment we're currently processing */
	let segmentCount = leaf.extract?.segment ?? 0;
	for (; nodeIndex < list.length; partIndex++, nodeIndex++, pathIndex++, segmentCount++) {
		const node = list[nodeIndex];
		if (node.kind === SEGMENT_TYPE_INDEX) break;
		if (node.kind === SEGMENT_TYPE_PATHLESS) {
			segmentCount--;
			partIndex--;
			pathIndex--;
			continue;
		}
		const part = parts[partIndex];
		const currentPathIndex = pathIndex;
		if (part) pathIndex += part.length;
		if (node.kind === 1) {
			nodeParts ??= leaf.node.fullPath.split("/");
			const nodePart = nodeParts[segmentCount];
			const preLength = node.prefix.length;
			if (nodePart.charCodeAt(preLength) === 123) {
				const sufLength = node.suffix.length;
				const name = nodePart.substring(preLength + 2, nodePart.length - sufLength - 1);
				const value = part.substring(preLength, part.length - sufLength);
				rawParams[name] = decodeURIComponent(value);
			} else {
				const name = nodePart.substring(1);
				rawParams[name] = decodeURIComponent(part);
			}
		} else if (node.kind === 3) {
			if (leaf.skipped & 1 << nodeIndex) {
				partIndex--;
				pathIndex = currentPathIndex - 1;
				continue;
			}
			nodeParts ??= leaf.node.fullPath.split("/");
			const nodePart = nodeParts[segmentCount];
			const preLength = node.prefix.length;
			const sufLength = node.suffix.length;
			const name = nodePart.substring(preLength + 3, nodePart.length - sufLength - 1);
			const value = node.suffix || node.prefix ? part.substring(preLength, part.length - sufLength) : part;
			if (value) rawParams[name] = decodeURIComponent(value);
		} else if (node.kind === 2) {
			const n = node;
			const value = path.substring(currentPathIndex + n.prefix.length, path.length - n.suffix.length);
			const splat = decodeURIComponent(value);
			rawParams["*"] = splat;
			rawParams._splat = splat;
			break;
		}
	}
	if (leaf.rawParams) Object.assign(rawParams, leaf.rawParams);
	return [rawParams, {
		part: partIndex,
		node: nodeIndex,
		path: pathIndex,
		segment: segmentCount
	}];
}
function buildRouteBranch(route) {
	const list = [route];
	while (route.parentRoute) {
		route = route.parentRoute;
		list.push(route);
	}
	list.reverse();
	return list;
}
function buildBranch(node) {
	const list = Array(node.depth + 1);
	do {
		list[node.depth] = node;
		node = node.parent;
	} while (node);
	return list;
}
function getNodeMatch(path, parts, segmentTree, fuzzy) {
	if (path === "/" && segmentTree.index) return {
		node: segmentTree.index,
		skipped: 0
	};
	const trailingSlash = !last(parts);
	const pathIsIndex = trailingSlash && path !== "/";
	const partsLength = parts.length - (trailingSlash ? 1 : 0);
	const stack = [{
		node: segmentTree,
		index: 1,
		skipped: 0,
		statics: 0,
		dynamics: 0,
		optionals: 0
	}];
	let bestFuzzy = null;
	let bestMatch = null;
	while (stack.length) {
		const frame = stack.pop();
		const { node, index, skipped, statics, dynamics, optionals } = frame;
		let { extract, rawParams } = frame;
		if (node.kind === 2 && node.route && !isFrameMoreSpecific(bestMatch, frame)) continue;
		if (node.parse) {
			if (!validateParseParams(path, parts, frame)) continue;
			rawParams = frame.rawParams;
			extract = frame.extract;
		}
		if (fuzzy && node.route && node.kind !== SEGMENT_TYPE_INDEX && isFrameMoreSpecific(bestFuzzy, frame)) bestFuzzy = frame;
		const isBeyondPath = index === partsLength;
		if (isBeyondPath) {
			if (node.route && (!pathIsIndex || node.kind === SEGMENT_TYPE_INDEX || node.kind === 2) && isFrameMoreSpecific(bestMatch, frame)) bestMatch = frame;
			if (!node.optional && !node.wildcard && !node.index && !node.pathless) continue;
		}
		const part = isBeyondPath ? void 0 : parts[index];
		let lowerPart;
		if (isBeyondPath && node.index) {
			const indexFrame = {
				node: node.index,
				index,
				skipped,
				statics,
				dynamics,
				optionals,
				extract,
				rawParams
			};
			let indexValid = true;
			if (node.index.parse) {
				if (!validateParseParams(path, parts, indexFrame)) indexValid = false;
			}
			if (indexValid) {
				if (!dynamics && !optionals && !skipped && isPerfectStaticMatch(statics, partsLength)) return indexFrame;
				if (isFrameMoreSpecific(bestMatch, indexFrame)) bestMatch = indexFrame;
			}
		}
		if (node.wildcard) for (let i = node.wildcard.length - 1; i >= 0; i--) {
			const segment = node.wildcard[i];
			const { prefix, suffix } = segment;
			if (prefix) {
				if (isBeyondPath) continue;
				if (!(segment.caseSensitive ? part : lowerPart ??= part.toLowerCase()).startsWith(prefix)) continue;
			}
			if (suffix) {
				if (isBeyondPath) continue;
				const end = parts.slice(index).join("/");
				const suffixPart = end.slice(-suffix.length);
				if ((segment.caseSensitive ? suffixPart : suffixPart.toLowerCase()) !== suffix || end.length - suffix.length < prefix.length) continue;
			}
			stack.push({
				node: segment,
				index: partsLength,
				skipped,
				statics,
				dynamics,
				optionals,
				extract,
				rawParams
			});
		}
		if (node.optional) {
			const nextSkipped = skipped | 1 << node.depth + 1;
			for (let i = node.optional.length - 1; i >= 0; i--) {
				const segment = node.optional[i];
				stack.push({
					node: segment,
					index,
					skipped: nextSkipped,
					statics,
					dynamics,
					optionals,
					extract,
					rawParams
				});
			}
			if (!isBeyondPath) for (let i = node.optional.length - 1; i >= 0; i--) {
				const segment = node.optional[i];
				const { prefix, suffix } = segment;
				if (prefix || suffix) {
					const casePart = segment.caseSensitive ? part : lowerPart ??= part.toLowerCase();
					if (prefix && !casePart.startsWith(prefix)) continue;
					if (suffix && casePart.indexOf(suffix, casePart.length - suffix.length) < prefix.length) continue;
				}
				stack.push({
					node: segment,
					index: index + 1,
					skipped,
					statics,
					dynamics,
					optionals: optionals + segmentScore(partsLength, index),
					extract,
					rawParams
				});
			}
		}
		if (!isBeyondPath && node.dynamic && part) for (let i = node.dynamic.length - 1; i >= 0; i--) {
			const segment = node.dynamic[i];
			const { prefix, suffix } = segment;
			if (prefix || suffix) {
				const casePart = segment.caseSensitive ? part : lowerPart ??= part.toLowerCase();
				if (prefix && !casePart.startsWith(prefix)) continue;
				if (suffix && casePart.indexOf(suffix, casePart.length - suffix.length) < prefix.length) continue;
			}
			stack.push({
				node: segment,
				index: index + 1,
				skipped,
				statics,
				dynamics: dynamics + segmentScore(partsLength, index),
				optionals,
				extract,
				rawParams
			});
		}
		if (!isBeyondPath && node.staticInsensitive) {
			const match = node.staticInsensitive.get(lowerPart ??= part.toLowerCase());
			if (match) stack.push({
				node: match,
				index: index + 1,
				skipped,
				statics: statics + segmentScore(partsLength, index),
				dynamics,
				optionals,
				extract,
				rawParams
			});
		}
		if (!isBeyondPath && node.static) {
			const match = node.static.get(part);
			if (match) stack.push({
				node: match,
				index: index + 1,
				skipped,
				statics: statics + segmentScore(partsLength, index),
				dynamics,
				optionals,
				extract,
				rawParams
			});
		}
		if (node.pathless) for (let i = node.pathless.length - 1; i >= 0; i--) {
			const segment = node.pathless[i];
			stack.push({
				node: segment,
				index,
				skipped,
				statics,
				dynamics,
				optionals,
				extract,
				rawParams
			});
		}
	}
	if (bestMatch) return bestMatch;
	if (fuzzy && bestFuzzy) {
		let sliceIndex = bestFuzzy.index;
		for (let i = 0; i < bestFuzzy.index; i++) sliceIndex += parts[i].length;
		const splat = sliceIndex === path.length ? "/" : path.slice(sliceIndex);
		bestFuzzy.rawParams ??= Object.create(null);
		bestFuzzy.rawParams["**"] = decodeURIComponent(splat);
		return bestFuzzy;
	}
	return null;
}
function segmentScore(partsLength, index) {
	return 2 ** (partsLength - index - 1);
}
function isPerfectStaticMatch(statics, partsLength) {
	return statics === 2 ** (partsLength - 1) - 1;
}
function validateParseParams(path, parts, frame) {
	let rawParams;
	let state;
	try {
		[rawParams, state] = extractParams(path, parts, frame);
	} catch {
		return null;
	}
	frame.rawParams = rawParams;
	frame.extract = state;
	if (!frame.node.parse) return true;
	try {
		if (frame.node.parse(rawParams) === false) return null;
	} catch {}
	return true;
}
function isFrameMoreSpecific(prev, next) {
	if (!prev) return true;
	return next.statics > prev.statics || next.statics === prev.statics && (next.dynamics > prev.dynamics || next.dynamics === prev.dynamics && (next.optionals > prev.optionals || next.optionals === prev.optionals && ((next.node.kind === SEGMENT_TYPE_INDEX) > (prev.node.kind === SEGMENT_TYPE_INDEX) || next.node.kind === SEGMENT_TYPE_INDEX === (prev.node.kind === SEGMENT_TYPE_INDEX) && next.node.depth > prev.node.depth)));
}
//#endregion
//#region node_modules/.bun/@tanstack+router-core@1.171.28/node_modules/@tanstack/router-core/dist/esm/path.js
/** Join path segments, cleaning duplicate slashes between parts. */
function joinPaths(paths) {
	return cleanPath(paths.filter((val) => {
		return val !== void 0;
	}).join("/"));
}
/** Remove repeated slashes from a path string. */
function cleanPath(path) {
	return path.replace(/\/{2,}/g, "/");
}
/** Trim leading slashes (except preserving root '/'). */
function trimPathLeft(path) {
	return path === "/" ? path : path.replace(/^\/{1,}/, "");
}
/** Trim trailing slashes (except preserving root '/'). */
function trimPathRight(path) {
	const len = path.length;
	return len > 1 && path[len - 1] === "/" ? path.replace(/\/{1,}$/, "") : path;
}
/** Trim both leading and trailing slashes. */
function trimPath(path) {
	return trimPathRight(trimPathLeft(path));
}
/**
* Resolve a destination path against a base, honoring trailing-slash policy
* and supporting relative segments (`.`/`..`) and absolute `to` values.
*/
function resolvePath({ base, to, trailingSlash = "never", cache }) {
	if (to.includes("//")) to = cleanPath(to);
	if (to.startsWith("/")) {
		if (to.length === 1 || trailingSlash === "preserve") return to;
		if (trailingSlash === "always") return to.endsWith("/") ? to : `${to}/`;
		return to.endsWith("/") ? to.slice(0, -1) : to;
	}
	const isBase = to === ".";
	let key;
	if (cache) {
		key = isBase ? base : base + "\0" + to;
		const cached = cache.get(key);
		if (cached) return cached;
	}
	let baseSegments;
	if (isBase) baseSegments = base.split("/");
	else {
		if (base.includes("//")) base = cleanPath(base);
		baseSegments = base.split("/");
		while (baseSegments.length > 1 && last(baseSegments) === "") baseSegments.pop();
		const toSegments = to.split("/");
		for (let index = 0, length = toSegments.length; index < length; index++) {
			const value = toSegments[index];
			if (value === "") {
				if (!index) baseSegments = [value];
				else if (index === length - 1) baseSegments.push(value);
			} else if (value === "..") if (baseSegments.length > 1) baseSegments.pop();
			else baseSegments = [""];
			else if (value === ".") {} else baseSegments.push(value);
		}
	}
	if (baseSegments.length > 1) {
		if (last(baseSegments) === "") {
			if (trailingSlash === "never") baseSegments.pop();
		} else if (trailingSlash === "always") baseSegments.push("");
	}
	const joined = baseSegments.join("/");
	const result = (isBase ? cleanPath(joined) : joined) || "/";
	if (key && cache) cache.set(key, result);
	return result;
}
/**
* Create a pre-compiled decode config from allowed characters.
* This should be called once at router initialization.
*/
function compileDecodeCharMap(pathParamsAllowedCharacters) {
	const charMap = new Map(pathParamsAllowedCharacters.map((char) => [encodeURIComponent(char), char]));
	const pattern = Array.from(charMap.keys()).map((key) => key.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")).join("|");
	const regex = new RegExp(pattern, "g");
	return (encoded) => encoded.replace(regex, (match) => charMap.get(match) ?? match);
}
function encodeParam(key, params, decoder) {
	const value = params[key];
	if (typeof value !== "string") return value;
	if (key === "_splat") {
		if (/^[a-zA-Z0-9\-._~!/]*$/.test(value)) return value;
		return value.split("/").map((segment) => encodePathParam(segment, decoder)).join("/");
	} else return encodePathParam(value, decoder);
}
/**
* Interpolate params and wildcards into a route path template.
*
* - Encodes params safely (configurable allowed characters)
* - Supports `{-$optional}` segments, `{prefix{$id}suffix}` and `{$}` wildcards
*/
function interpolatePath({ path, params, decoder, ...rest }) {
	let isMissingParams = false;
	const usedParams = Object.create(null);
	if (!path || path === "/") return {
		interpolatedPath: "/",
		usedParams,
		isMissingParams
	};
	if (!path.includes("$")) return {
		interpolatedPath: path,
		usedParams,
		isMissingParams
	};
	const length = path.length;
	let cursor = 0;
	let segment;
	let joined = "";
	while (cursor < length) {
		const start = cursor;
		segment = parseSegment(path, start, segment);
		const end = segment[5];
		cursor = end + 1;
		if (start === end) continue;
		const kind = segment[0];
		if (kind === 0) {
			joined += "/" + path.substring(start, end);
			continue;
		}
		if (kind === 2) {
			const splat = params._splat;
			usedParams._splat = splat;
			usedParams["*"] = splat;
			const prefix = path.substring(start, segment[1]);
			const suffix = path.substring(segment[4], end);
			if (!splat) {
				isMissingParams = true;
				if (prefix || suffix) joined += "/" + prefix + suffix;
				continue;
			}
			const value = encodeParam("_splat", params, decoder);
			joined += "/" + prefix + value + suffix;
			continue;
		}
		if (kind === 1) {
			const key = path.substring(segment[2], segment[3]);
			if (!isMissingParams && !(key in params)) isMissingParams = true;
			usedParams[key] = params[key];
			const prefix = path.substring(start, segment[1]);
			const suffix = path.substring(segment[4], end);
			const value = encodeParam(key, params, decoder) ?? "undefined";
			joined += "/" + prefix + value + suffix;
			continue;
		}
		if (kind === 3) {
			const key = path.substring(segment[2], segment[3]);
			const valueRaw = params[key];
			if (valueRaw == null) continue;
			usedParams[key] = valueRaw;
			const prefix = path.substring(start, segment[1]);
			const suffix = path.substring(segment[4], end);
			const value = encodeParam(key, params, decoder) ?? "";
			joined += "/" + prefix + value + suffix;
			continue;
		}
	}
	if (path.endsWith("/")) joined += "/";
	return {
		usedParams,
		interpolatedPath: joined || "/",
		isMissingParams
	};
}
function encodePathParam(value, decoder) {
	const encoded = encodeURIComponent(value);
	return decoder?.(encoded) ?? encoded;
}
//#endregion
//#region node_modules/.bun/@tanstack+router-core@1.171.28/node_modules/@tanstack/router-core/dist/esm/not-found.js
/** Determine if a value is a TanStack Router not-found error. */
function isNotFound(obj) {
	return obj?.isNotFound === true;
}
//#endregion
//#region node_modules/.bun/@tanstack+router-core@1.171.28/node_modules/@tanstack/router-core/dist/esm/scroll-restoration.js
function getSafeSessionStorage() {
	try {
		return sessionStorage;
	} catch {
		return;
	}
}
var storageKey = "tsr-scroll-restoration-v1_3";
var safeSessionStorage = getSafeSessionStorage();
function createScrollRestorationCache() {
	try {
		return JSON.parse(safeSessionStorage?.getItem("tsr-scroll-restoration-v1_3") || "{}");
	} catch {
		return {};
	}
}
function persistScrollRestorationCache() {
	try {
		safeSessionStorage?.setItem(storageKey, JSON.stringify(scrollRestorationCache));
	} catch {}
}
var scrollRestorationCache = /* @__PURE__ */ createScrollRestorationCache();
var scrollRestorationIdAttribute = "data-scroll-restoration-id";
/**
* The default `getKey` function for `useScrollRestoration`.
* It returns the `key` from the location state or the `href` of the location.
*
* The `location.href` is used as a fallback to support the use case where the location state is not available like the initial render.
*/
var defaultGetScrollRestorationKey = (location) => {
	return location.state.__TSR_key || location.href;
};
function getScrollRestorationSelector(element) {
	const attrId = element.getAttribute(scrollRestorationIdAttribute);
	if (attrId) return `[${scrollRestorationIdAttribute}="${attrId}"]`;
	let selector = "";
	let el = element;
	let parent;
	while (parent = el.parentNode) {
		let index = 1;
		let sibling = el;
		while (sibling = sibling.previousElementSibling) index++;
		const part = `${el.localName}:nth-child(${index})`;
		selector = selector ? `${part} > ${selector}` : part;
		el = parent;
	}
	return selector;
}
var ignoreScroll = false;
var windowScrollTarget = "window";
function getElement(selector) {
	try {
		return typeof selector === "function" ? selector() : document.querySelector(selector);
	} catch {}
}
function getScrollToTopElements(scrollToTopSelectors) {
	const elements = /* @__PURE__ */ new Set();
	for (const selector of scrollToTopSelectors) {
		if (selector === windowScrollTarget) continue;
		const element = getElement(selector);
		if (element) elements.add(element);
	}
	return elements;
}
function setupScrollRestoration(router, force) {
	const shouldSetupScrollRestoration = force ?? router.options.scrollRestoration;
	const scroll = router._scroll;
	if (shouldSetupScrollRestoration) scroll.restoring = true;
	const getKey = router.options.getScrollRestorationKey || defaultGetScrollRestorationKey;
	const trackedScrollTargets = /* @__PURE__ */ new Set();
	const snapshotCurrentScrollTargets = (restoreKey) => {
		const keyEntry = scrollRestorationCache[restoreKey] ||= {};
		for (const target of trackedScrollTargets) if (target === document) keyEntry[windowScrollTarget] = {
			scrollX,
			scrollY
		};
		else if (target.isConnected) keyEntry[getScrollRestorationSelector(target)] = {
			scrollX: target.scrollLeft,
			scrollY: target.scrollTop
		};
	};
	if (shouldSetupScrollRestoration && !scroll.restoration) {
		scroll.restoration = true;
		ignoreScroll = false;
		history.scrollRestoration = "manual";
		document.addEventListener("scroll", (event) => {
			if (ignoreScroll) return;
			trackedScrollTargets.add(event.target);
		}, true);
		router.subscribe("onBeforeLoad", (event) => {
			if (event.fromLocation) snapshotCurrentScrollTargets(getKey(event.fromLocation));
			trackedScrollTargets.clear();
		});
		addEventListener("pagehide", () => {
			snapshotCurrentScrollTargets(getKey(router.stores.resolvedLocation.get() ?? router.stores.location.get()));
			persistScrollRestorationCache();
		});
	}
	if (scroll.reset) return;
	scroll.reset = true;
	router.subscribe("onRendered", (event) => {
		const behavior = router.options.scrollRestorationBehavior;
		const scrollToTopSelectors = router.options.scrollToTopSelectors;
		const shouldResetScroll = scroll.next;
		const hashNavigation = scroll.hash;
		let scrollToTopElements;
		trackedScrollTargets.clear();
		scroll.next = true;
		scroll.hash = false;
		if (typeof router.options.scrollRestoration === "function" && !router.options.scrollRestoration({ location: router.latestLocation })) return;
		const cacheKey = getKey(event.toLocation);
		const fromCacheKey = event.fromLocation && getKey(event.fromLocation);
		if (scroll.restoring && fromCacheKey && fromCacheKey !== cacheKey) {
			const fromElementEntries = scrollRestorationCache[fromCacheKey];
			if (fromElementEntries) {
				let toElementEntries = scrollRestorationCache[cacheKey];
				for (const elementSelector in fromElementEntries) {
					if (elementSelector === windowScrollTarget) {
						if (shouldResetScroll) continue;
					} else {
						const element = getElement(elementSelector);
						if (!element) continue;
						if (shouldResetScroll && scrollToTopSelectors) {
							scrollToTopElements ??= getScrollToTopElements(scrollToTopSelectors);
							if (scrollToTopElements.has(element)) continue;
						}
					}
					if (!toElementEntries) toElementEntries = scrollRestorationCache[cacheKey] = {};
					toElementEntries[elementSelector] ??= fromElementEntries[elementSelector];
				}
			}
		}
		ignoreScroll = true;
		try {
			const hash = event.toLocation.hash;
			const hashScrollIntoViewOptions = event.toLocation.state.__hashScrollIntoViewOptions ?? true;
			let windowRestored = false;
			if (shouldResetScroll) {
				if (!hash && scrollToTopSelectors) scrollToTopElements ??= getScrollToTopElements(scrollToTopSelectors);
				const skipWindowRestore = hash && hashScrollIntoViewOptions && hashNavigation;
				const elementEntries = scroll.restoring ? scrollRestorationCache[cacheKey] : void 0;
				if (elementEntries) for (const elementSelector in elementEntries) {
					const { scrollX, scrollY } = elementEntries[elementSelector];
					if (elementSelector === windowScrollTarget) {
						if (skipWindowRestore) continue;
						scrollTo({
							top: scrollY,
							left: scrollX,
							behavior
						});
						windowRestored = true;
					} else {
						const element = getElement(elementSelector);
						if (element) {
							element.scrollLeft = scrollX;
							element.scrollTop = scrollY;
							scrollToTopElements?.delete(element);
						}
					}
				}
				if (!hash) {
					const scrollOptions = {
						top: 0,
						left: 0,
						behavior
					};
					if (!windowRestored) scrollTo(scrollOptions);
					if (scrollToTopElements) for (const element of scrollToTopElements) element.scrollTo(scrollOptions);
				}
			}
			if (!windowRestored && hash && hashScrollIntoViewOptions) document.getElementById(hash)?.scrollIntoView(hashScrollIntoViewOptions);
		} finally {
			ignoreScroll = false;
		}
	});
}
//#endregion
//#region node_modules/.bun/@tanstack+router-core@1.171.28/node_modules/@tanstack/router-core/dist/esm/qss.js
/**
* Program is a reimplementation of the `qss` package:
* Copyright (c) Luke Edwards luke.edwards05@gmail.com, MIT License
* https://github.com/lukeed/qss/blob/master/license.md
*
* This reimplementation uses modern browser APIs
* (namely URLSearchParams) and TypeScript while still
* maintaining the original functionality and interface.
*
* Update: this implementation has also been mangled to
* fit exactly our use-case (single value per key in encoding).
*/
/**
* Encodes an object into a query string.
* @param obj - The object to encode into a query string.
* @param stringify - An optional custom stringify function.
* @returns The encoded query string.
* @example
* ```
* // Example input: encode({ token: 'foo', key: 'value' })
* // Expected output: "token=foo&key=value"
* ```
*/
function encode(obj, stringify = String) {
	const result = new URLSearchParams();
	for (const key in obj) {
		const val = obj[key];
		if (val !== void 0) result.set(key, stringify(val));
	}
	return result.toString();
}
/**
* Converts a string value to its appropriate type (string, number, boolean).
* @param mix - The string value to convert.
* @returns The converted value.
* @example
* // Example input: toValue("123")
* // Expected output: 123
*/
function toValue(str) {
	if (!str) return "";
	if (str === "false") return false;
	if (str === "true") return true;
	return +str * 0 === 0 && +str + "" === str ? +str : str;
}
/**
* Decodes a query string into an object.
* @param str - The query string to decode.
* @returns The decoded key-value pairs in an object format.
* @example
* // Example input: decode("token=foo&key=value")
* // Expected output: { "token": "foo", "key": "value" }
*/
function decode(str) {
	const searchParams = new URLSearchParams(str);
	const result = Object.create(null);
	for (const [key, value] of searchParams.entries()) {
		const previousValue = result[key];
		if (previousValue == null) result[key] = toValue(value);
		else if (Array.isArray(previousValue)) previousValue.push(toValue(value));
		else result[key] = [previousValue, toValue(value)];
	}
	return result;
}
//#endregion
//#region node_modules/.bun/@tanstack+router-core@1.171.28/node_modules/@tanstack/router-core/dist/esm/searchParams.js
var jsonStart = /^(?:\s|["[{\d-]|fa|nu|tr)/;
/** Default `parseSearch` that strips leading '?' and JSON-parses values. */
var defaultParseSearch = parseSearchWith(JSON.parse);
/** Default `stringifySearch` using JSON.stringify for complex values. */
var defaultStringifySearch = stringifySearchWith(JSON.stringify, JSON.parse);
/**
* Build a `parseSearch` function using a provided JSON-like parser.
*
* The returned function strips a leading `?`, decodes values, and attempts to
* JSON-parse string values using the given `parser`.
*
* @param parser Function to parse a string value (e.g. `JSON.parse`).
* @returns A `parseSearch` function compatible with `Router` options.
* @link https://tanstack.com/router/latest/docs/framework/react/guide/custom-search-param-serialization
*/
function parseSearchWith(parser) {
	const isJsonParser = parser === JSON.parse;
	return (searchStr) => {
		if (searchStr[0] === "?") searchStr = searchStr.substring(1);
		const query = decode(searchStr);
		for (const key in query) {
			const value = query[key];
			if (typeof value === "string") {
				if (isJsonParser && !jsonStart.test(value)) continue;
				try {
					query[key] = parser(value);
				} catch (_err) {}
			}
		}
		return query;
	};
}
/**
* Build a `stringifySearch` function using a provided serializer.
*
* Non-primitive values are serialized with `stringify`. If a `parser` is
* supplied, string values that are parseable are re-serialized to ensure
* symmetry with `parseSearch`.
*
* @param stringify Function to serialize a value (e.g. `JSON.stringify`).
* @param parser Optional parser to detect parseable strings.
* @returns A `stringifySearch` function compatible with `Router` options.
* @link https://tanstack.com/router/latest/docs/framework/react/guide/custom-search-param-serialization
*/
function stringifySearchWith(stringify, parser) {
	const isJsonParser = parser === JSON.parse;
	function stringifyValue(val) {
		if (val && typeof val === "object") try {
			return stringify(val);
		} catch (_err) {}
		else if (parser && typeof val === "string") {
			if (isJsonParser && !jsonStart.test(val)) return val;
			try {
				parser(val);
				return stringify(val);
			} catch (_err) {}
		}
		return val;
	}
	return (search) => {
		const searchStr = encode(search, stringifyValue);
		return searchStr ? `?${searchStr}` : "";
	};
}
//#endregion
//#region node_modules/.bun/@tanstack+router-core@1.171.28/node_modules/@tanstack/router-core/dist/esm/root.js
/** Stable identifier used for the root route in a route tree. */
var rootRouteId = "__root__";
//#endregion
//#region node_modules/.bun/@tanstack+router-core@1.171.28/node_modules/@tanstack/router-core/dist/esm/redirect.js
/**
* Create a redirect Response understood by TanStack Router.
*
* Use from route `loader`/`beforeLoad` or server functions to trigger a
* navigation. If `throw: true` is set, the redirect is thrown instead of
* returned. When an absolute `href` is supplied and `reloadDocument` is not
* set, a full-document navigation is inferred.
*
* @param opts Options for the redirect. Common fields:
* - `href`: absolute URL for external redirects; infers `reloadDocument`.
* - `statusCode`: HTTP status code to use (defaults to 307).
* - `headers`: additional headers to include on the Response.
* - Standard navigation options like `to`, `params`, `search`, `replace`,
*   and `reloadDocument` for internal redirects.
* @returns A Response augmented with router navigation options.
* @link https://tanstack.com/router/latest/docs/framework/react/api/router/redirectFunction
*/
function redirect(opts) {
	opts.statusCode = opts.statusCode || opts.code || 307;
	if (!opts.reloadDocument && typeof opts.href === "string" && isAbsoluteUrl(opts.href)) opts.reloadDocument = true;
	const headers = new Headers(opts.headers);
	if (opts.href && headers.get("Location") === null) headers.set("Location", opts.href);
	const response = new Response(null, {
		status: opts.statusCode,
		headers
	});
	response.options = opts;
	if (opts.throw) throw response;
	return response;
}
/** Check whether a value is a TanStack Router redirect Response. */
/** Check whether a value is a TanStack Router redirect Response. */
function isRedirect(obj) {
	return obj instanceof Response && !!obj.options;
}
//#endregion
//#region node_modules/.bun/@tanstack+router-core@1.171.28/node_modules/@tanstack/router-core/dist/esm/rewrite.js
/** Compose multiple rewrite pairs into a single in/out rewrite. */
function composeRewrites(rewrites) {
	return {
		input: ({ url }) => {
			for (const rewrite of rewrites) url = executeRewriteInput(rewrite, url);
			return url;
		},
		output: ({ url }) => {
			for (let i = rewrites.length - 1; i >= 0; i--) url = executeRewriteOutput(rewrites[i], url);
			return url;
		}
	};
}
/** Create a rewrite pair that strips/adds a basepath on input/output. */
function rewriteBasepath(opts) {
	const trimmedBasepath = trimPath(opts.basepath);
	const normalizedBasepath = `/${trimmedBasepath}`;
	const checkBasepath = opts.caseSensitive ? normalizedBasepath : normalizedBasepath.toLowerCase();
	const checkBasepathWithSlash = `${checkBasepath}/`;
	return {
		input: ({ url }) => {
			const pathname = opts.caseSensitive ? url.pathname : url.pathname.toLowerCase();
			if (pathname === checkBasepath) url.pathname = "/";
			else if (pathname.startsWith(checkBasepathWithSlash)) url.pathname = url.pathname.slice(normalizedBasepath.length);
			return url;
		},
		output: ({ url }) => {
			url.pathname = joinPaths([
				"/",
				trimmedBasepath,
				url.pathname
			]);
			return url;
		}
	};
}
/** Execute a location input rewrite if provided. */
function executeRewriteInput(rewrite, url) {
	const res = rewrite?.input?.({ url });
	if (res) {
		if (typeof res === "string") return new URL(res);
		else if (res instanceof URL) return res;
	}
	return url;
}
/** Execute a location output rewrite if provided. */
function executeRewriteOutput(rewrite, url) {
	const res = rewrite?.output?.({ url });
	if (res) {
		if (typeof res === "string") return new URL(res);
		else if (res instanceof URL) return res;
	}
	return url;
}
//#endregion
//#region node_modules/.bun/@tanstack+router-core@1.171.28/node_modules/@tanstack/router-core/dist/esm/stores.js
function createRouterStores(initialLocation, config) {
	const { createMutableStore, createReadonlyStore, batch } = config;
	const byRoute = /* @__PURE__ */ new Map();
	const status = createMutableStore("idle");
	const location = createMutableStore(initialLocation);
	const resolvedLocation = createMutableStore(void 0);
	const ids = createMutableStore([]);
	const matches = createReadonlyStore(() => ids.get().map((id) => byRoute.get(id).get()));
	const __store = createReadonlyStore(() => ({
		status: status.get(),
		isLoading: status.get() === "pending",
		matches: matches.get(),
		location: location.get(),
		resolvedLocation: resolvedLocation.get()
	}));
	function getMatchStore(routeId) {
		let matchStore = byRoute.get(routeId);
		if (!matchStore) {
			matchStore = createMutableStore(void 0);
			byRoute.set(routeId, matchStore);
		}
		return matchStore;
	}
	const store = {
		status,
		location,
		resolvedLocation,
		ids,
		matches,
		byRoute,
		__store,
		getMatchStore,
		setMatches
	};
	function setMatches(nextMatches) {
		const previousIds = ids.get();
		const nextIds = nextMatches.map((match) => match.routeId);
		batch(() => {
			if (!arraysEqual(previousIds, nextIds)) ids.set(nextIds);
			for (const id of previousIds) if (!nextIds.includes(id)) byRoute.get(id).set(() => void 0);
			for (const nextMatch of nextMatches) {
				const matchStore = getMatchStore(nextMatch.routeId);
				if (matchStore.get() !== nextMatch) matchStore.set(nextMatch);
			}
		});
	}
	return store;
}
//#endregion
//#region node_modules/.bun/@tanstack+history@1.162.2/node_modules/@tanstack/history/dist/esm/index.js
var stateIndexKey$1 = "__TSR_index";
var popStateEvent = "popstate";
var beforeUnloadEvent = "beforeunload";
function createHistory$1(opts) {
	let location = opts.getLocation();
	const subscribers = /* @__PURE__ */ new Set();
	const notify = (action) => {
		location = opts.getLocation();
		subscribers.forEach((subscriber) => subscriber({
			location,
			action
		}));
	};
	const handleIndexChange = (action) => {
		if (opts.notifyOnIndexChange ?? true) notify(action);
		else location = opts.getLocation();
	};
	const tryNavigation = async ({ task, navigateOpts, ...actionInfo }) => {
		if (navigateOpts?.ignoreBlocker ?? false) {
			task();
			return;
		}
		const blockers = opts.getBlockers?.() ?? [];
		const isPushOrReplace = actionInfo.type === "PUSH" || actionInfo.type === "REPLACE";
		if (typeof document !== "undefined" && blockers.length && isPushOrReplace) for (const blocker of blockers) {
			const nextLocation = parseHref$1(actionInfo.path, actionInfo.state);
			if (await blocker.blockerFn({
				currentLocation: location,
				nextLocation,
				action: actionInfo.type
			})) {
				opts.onBlocked?.();
				return;
			}
		}
		task();
	};
	return {
		get location() {
			return location;
		},
		get length() {
			return opts.getLength();
		},
		subscribers,
		subscribe: (cb) => {
			subscribers.add(cb);
			return () => {
				subscribers.delete(cb);
			};
		},
		push: (path, state, navigateOpts) => {
			const currentIndex = location.state[stateIndexKey$1];
			state = assignKeyAndIndex(currentIndex + 1, state);
			tryNavigation({
				task: () => {
					opts.pushState(path, state);
					notify({ type: "PUSH" });
				},
				navigateOpts,
				type: "PUSH",
				path,
				state
			});
		},
		replace: (path, state, navigateOpts) => {
			const currentIndex = location.state[stateIndexKey$1];
			state = assignKeyAndIndex(currentIndex, state);
			tryNavigation({
				task: () => {
					opts.replaceState(path, state);
					notify({ type: "REPLACE" });
				},
				navigateOpts,
				type: "REPLACE",
				path,
				state
			});
		},
		go: (index, navigateOpts) => {
			tryNavigation({
				task: () => {
					opts.go(index, navigateOpts?.ignoreBlocker ?? false);
					handleIndexChange({
						type: "GO",
						index
					});
				},
				navigateOpts,
				type: "GO"
			});
		},
		back: (navigateOpts) => {
			tryNavigation({
				task: () => {
					opts.back(navigateOpts?.ignoreBlocker ?? false);
					handleIndexChange({ type: "BACK" });
				},
				navigateOpts,
				type: "BACK"
			});
		},
		forward: (navigateOpts) => {
			tryNavigation({
				task: () => {
					opts.forward(navigateOpts?.ignoreBlocker ?? false);
					handleIndexChange({ type: "FORWARD" });
				},
				navigateOpts,
				type: "FORWARD"
			});
		},
		canGoBack: () => location.state[stateIndexKey$1] !== 0,
		createHref: (str) => opts.createHref(str),
		block: (blocker) => {
			if (!opts.setBlockers) return () => {};
			const blockers = opts.getBlockers?.() ?? [];
			opts.setBlockers([...blockers, blocker]);
			return () => {
				const blockers = opts.getBlockers?.() ?? [];
				opts.setBlockers?.(blockers.filter((b) => b !== blocker));
			};
		},
		flush: () => opts.flush?.(),
		destroy: () => opts.destroy?.(),
		notify
	};
}
function assignKeyAndIndex(index, state) {
	if (!state) state = {};
	const key = createRandomKey$1();
	return {
		...state,
		key,
		__TSR_key: key,
		[stateIndexKey$1]: index
	};
}
/**
* Creates a history object that can be used to interact with the browser's
* navigation. This is a lightweight API wrapping the browser's native methods.
* It is designed to work with TanStack Router, but could be used as a standalone API as well.
* IMPORTANT: This API implements history throttling via a microtask to prevent
* excessive calls to the history API. In some browsers, calling history.pushState or
* history.replaceState in quick succession can cause the browser to ignore subsequent
* calls. This API smooths out those differences and ensures that your application
* state will *eventually* match the browser state. In most cases, this is not a problem,
* but if you need to ensure that the browser state is up to date, you can use the
* `history.flush` method to immediately flush all pending state changes to the browser URL.
* @param opts
* @param opts.getHref A function that returns the current href (path + search + hash)
* @param opts.createHref A function that takes a path and returns a href (path + search + hash)
* @returns A history instance
*/
function createBrowserHistory(opts) {
	const win = opts?.window ?? (typeof document !== "undefined" ? window : void 0);
	const originalPushState = win.history.pushState;
	const originalReplaceState = win.history.replaceState;
	let blockers = [];
	const _getBlockers = () => blockers;
	const _setBlockers = (newBlockers) => blockers = newBlockers;
	const createHref = opts?.createHref ?? ((path) => path);
	const parseLocation = opts?.parseLocation ?? (() => parseHref$1(`${win.location.pathname}${win.location.search}${win.location.hash}`, win.history.state));
	if (!win.history.state?.__TSR_key && !win.history.state?.key) {
		const addedKey = createRandomKey$1();
		win.history.replaceState({
			[stateIndexKey$1]: 0,
			key: addedKey,
			__TSR_key: addedKey
		}, "");
	}
	let currentLocation = parseLocation();
	let rollbackLocation;
	let nextPopIsGo = false;
	let ignoreNextPop = false;
	let skipBlockerNextPop = false;
	let ignoreNextBeforeUnload = false;
	const getLocation = () => currentLocation;
	let next;
	const flush = () => {
		if (!next) return;
		history._ignoreSubscribers = true;
		(next[2] ? win.history.pushState : win.history.replaceState)(next[1], "", next[0]);
		history._ignoreSubscribers = false;
		next = void 0;
		rollbackLocation = void 0;
	};
	const queueHistoryAction = (isPush, destHref, state) => {
		const href = createHref(destHref);
		const hasPendingAction = !!next;
		if (!hasPendingAction) rollbackLocation = currentLocation;
		currentLocation = parseHref$1(destHref, state);
		next = [
			href,
			state,
			next?.[2] || isPush
		];
		if (!hasPendingAction) queueMicrotask(() => flush());
	};
	const onPushPop = (type) => {
		currentLocation = parseLocation();
		history.notify({ type });
	};
	const onPushPopEvent = async () => {
		ignoreNextBeforeUnload = false;
		if (ignoreNextPop) {
			ignoreNextPop = false;
			return;
		}
		const nextLocation = parseLocation();
		const delta = nextLocation.state[stateIndexKey$1] - currentLocation.state[stateIndexKey$1];
		const isForward = delta === 1;
		const isBack = delta === -1;
		const isGo = !isForward && !isBack || nextPopIsGo;
		nextPopIsGo = false;
		const action = isGo ? "GO" : isBack ? "BACK" : "FORWARD";
		const notify = isGo ? {
			type: "GO",
			index: delta
		} : { type: isBack ? "BACK" : "FORWARD" };
		if (skipBlockerNextPop) skipBlockerNextPop = false;
		else {
			const blockers = _getBlockers();
			if (typeof document !== "undefined" && blockers.length) {
				for (const blocker of blockers) if (await blocker.blockerFn({
					currentLocation,
					nextLocation,
					action
				})) {
					ignoreNextPop = true;
					win.history.go(-delta);
					history.notify(notify);
					return;
				}
			}
		}
		currentLocation = parseLocation();
		history.notify(notify);
	};
	const onBeforeUnload = (e) => {
		if (ignoreNextBeforeUnload) {
			ignoreNextBeforeUnload = false;
			return;
		}
		let shouldBlock = false;
		const blockers = _getBlockers();
		if (typeof document !== "undefined" && blockers.length) for (const blocker of blockers) {
			const shouldHaveBeforeUnload = blocker.enableBeforeUnload ?? true;
			if (shouldHaveBeforeUnload === true) {
				shouldBlock = true;
				break;
			}
			if (typeof shouldHaveBeforeUnload === "function" && shouldHaveBeforeUnload() === true) {
				shouldBlock = true;
				break;
			}
		}
		if (shouldBlock) {
			e.preventDefault();
			return e.returnValue = "";
		}
	};
	const history = createHistory$1({
		getLocation,
		getLength: () => win.history.length,
		pushState: (href, state) => queueHistoryAction(true, href, state),
		replaceState: (href, state) => queueHistoryAction(false, href, state),
		back: (ignoreBlocker) => {
			if (ignoreBlocker) {
				skipBlockerNextPop = true;
				ignoreNextBeforeUnload = true;
			}
			return win.history.back();
		},
		forward: (ignoreBlocker) => {
			if (ignoreBlocker) {
				skipBlockerNextPop = true;
				ignoreNextBeforeUnload = true;
			}
			win.history.forward();
		},
		go: (n, ignoreBlocker) => {
			nextPopIsGo = true;
			if (ignoreBlocker) {
				skipBlockerNextPop = true;
				ignoreNextBeforeUnload = true;
			}
			win.history.go(n);
		},
		createHref: (href) => createHref(href),
		flush,
		destroy: () => {
			win.history.pushState = originalPushState;
			win.history.replaceState = originalReplaceState;
			win.removeEventListener(beforeUnloadEvent, onBeforeUnload, { capture: true });
			win.removeEventListener(popStateEvent, onPushPopEvent);
		},
		onBlocked: () => {
			if (rollbackLocation && currentLocation !== rollbackLocation) currentLocation = rollbackLocation;
		},
		getBlockers: _getBlockers,
		setBlockers: _setBlockers,
		notifyOnIndexChange: false
	});
	win.addEventListener(beforeUnloadEvent, onBeforeUnload, { capture: true });
	win.addEventListener(popStateEvent, onPushPopEvent);
	win.history.pushState = function(...args) {
		const res = originalPushState.apply(win.history, args);
		if (!history._ignoreSubscribers) onPushPop("PUSH");
		return res;
	};
	win.history.replaceState = function(...args) {
		const res = originalReplaceState.apply(win.history, args);
		if (!history._ignoreSubscribers) onPushPop("REPLACE");
		return res;
	};
	return history;
}
/**
* Sanitize a path to prevent open redirect vulnerabilities.
* Removes control characters and collapses leading double slashes.
*/
function sanitizePath$1(path) {
	let sanitized = path.replace(/[\x00-\x1f\x7f]/g, "");
	if (sanitized.startsWith("//")) sanitized = "/" + sanitized.replace(/^\/+/, "");
	return sanitized;
}
function parseHref$1(href, state) {
	const sanitizedHref = sanitizePath$1(href);
	const hashIndex = sanitizedHref.indexOf("#");
	const searchIndex = sanitizedHref.indexOf("?");
	const addedKey = createRandomKey$1();
	return {
		href: sanitizedHref,
		pathname: sanitizedHref.substring(0, hashIndex > 0 ? searchIndex > 0 ? Math.min(hashIndex, searchIndex) : hashIndex : searchIndex > 0 ? searchIndex : sanitizedHref.length),
		hash: hashIndex > -1 ? sanitizedHref.substring(hashIndex) : "",
		search: searchIndex > -1 ? sanitizedHref.slice(searchIndex, hashIndex === -1 ? void 0 : hashIndex) : "",
		state: state || {
			[stateIndexKey$1]: 0,
			key: addedKey,
			__TSR_key: addedKey
		}
	};
}
function createRandomKey$1() {
	return (Math.random() + 1).toString(36).substring(7);
}
//#endregion
//#region node_modules/.bun/@tanstack+router-core@1.171.28/node_modules/@tanstack/router-core/dist/esm/router.js
function routeNeedsLoad(route) {
	return route.options.loader || route.options.beforeLoad || route.lazyFn || route.options.component?.preload || route.options.pendingComponent?.preload;
}
/**
* Compute whether path, href or hash changed between previous and current
* resolved locations.
*/
function getLocationChangeInfo(location, resolvedLocation) {
	return {
		fromLocation: resolvedLocation,
		toLocation: location,
		pathChanged: resolvedLocation?.pathname !== location.pathname,
		hrefChanged: resolvedLocation?.href !== location.href,
		hashChanged: resolvedLocation?.hash !== location.hash
	};
}
/**
* Return only state owned by the application, excluding volatile history
* bookkeeping. Mask payloads (`__tempLocation`/`__tempKey`) are kept: they
* distinguish otherwise-identical locations.
*/
function _getUserHistoryState({ key: _key, __TSR_key: _tsrKey, __TSR_index: _tsrIndex, __hashScrollIntoViewOptions: _hashScroll, ...state }) {
	return state;
}
function lifecycleEnd(matches) {
	return matches.findIndex((match) => match.status === "error" || match.status === "notFound" || match._notFound) + 1;
}
/** Run route lifecycle callbacks in leave/enter/stay phases. */
function runRouteLifecycle(router, previous, matches, previousEnd, nextEnd, owner) {
	if (previousEnd) previous = previous.slice(0, previousEnd);
	if (nextEnd) matches = matches.slice(0, nextEnd);
	for (const match of previous) {
		if (owner && router._tx !== owner) return;
		if (!matches.some((candidate) => candidate.routeId === match.routeId)) router.routesById[match.routeId].options.onLeave?.(match);
	}
	for (const match of matches) {
		if (owner && router._tx !== owner) return;
		router.routesById[match.routeId].options[previous.some((candidate) => candidate.routeId === match.routeId) ? "onStay" : "onEnter"]?.(match);
	}
}
/**
* Core, framework-agnostic router engine that powers TanStack Router.
*
* Provides navigation, matching, loading, preloading, caching and event APIs
* used by framework adapters (React/Solid). Prefer framework helpers like
* `createRouter` in app code.
*
* @link https://tanstack.com/router/latest/docs/framework/react/api/router/RouterType
*/
var RouterCore = class {
	/**
	* @deprecated Use the `createRouter` function instead
	*/
	constructor(options, getStoreConfig) {
		this.tempLocationKey = `${Math.round(Math.random() * 1e7)}`;
		this._scroll = { next: true };
		this.subscribers = /* @__PURE__ */ new Set();
		this._cache = /* @__PURE__ */ new Map();
		this._committed = [];
		this.routeBranchCache = /* @__PURE__ */ new WeakMap();
		this.lightweightCache = /* @__PURE__ */ new WeakMap();
		this.startTransition = async (fn) => {
			fn();
			return false;
		};
		this.update = (newOptions) => {
			const prevOptions = this.options;
			const prevBasepath = this.basepath ?? prevOptions?.basepath ?? "/";
			const basepathWasUnset = this.basepath === void 0;
			const prevRewriteOption = prevOptions?.rewrite;
			this.options = {
				...prevOptions,
				...newOptions
			};
			this.isServer = this.options.isServer ?? false ?? typeof document === "undefined";
			this.protocolAllowlist = new Set(this.options.protocolAllowlist);
			if (this.options.pathParamsAllowedCharacters) this.pathParamsDecoder = compileDecodeCharMap(this.options.pathParamsAllowedCharacters);
			if (!this.history || this.options.history && this.options.history !== this.history) if (!this.options.history) this.history = createBrowserHistory();
			else this.history = this.options.history;
			this.origin = this.options.origin;
			if (!this.origin) if (window?.origin && window.origin !== "null") this.origin = window.origin;
			else this.origin = "http://localhost";
			if (this.history) this.updateLatestLocation();
			if (this.options.routeTree !== this.routeTree) {
				this.routeTree = this.options.routeTree;
				let processRouteTreeResult;
				this.resolvePathCache = createSieveCache(1e3);
				processRouteTreeResult = this.buildRouteTree();
				this.setRoutes(processRouteTreeResult);
			}
			if (!this.stores && this.latestLocation) {
				const config = this.getStoreConfig(this);
				this.batch = config.batch;
				this.stores = createRouterStores(this.latestLocation, config);
				setupScrollRestoration(this);
			}
			const nextBasepath = this.options.basepath ?? "/";
			const nextRewriteOption = this.options.rewrite;
			if (basepathWasUnset || prevBasepath !== nextBasepath || prevRewriteOption !== nextRewriteOption) {
				this.basepath = nextBasepath;
				const rewrites = [];
				const trimmed = trimPath(nextBasepath);
				if (trimmed && trimmed !== "/") rewrites.push(rewriteBasepath({ basepath: nextBasepath }));
				if (nextRewriteOption) rewrites.push(nextRewriteOption);
				this.rewrite = rewrites.length === 0 ? void 0 : rewrites.length === 1 ? rewrites[0] : composeRewrites(rewrites);
				if (this.history) this.updateLatestLocation();
				if (this.stores) this.stores.location.set(this.latestLocation);
			}
		};
		this.updateLatestLocation = () => {
			this.latestLocation = this.parseLocation(this.history.location, this.latestLocation);
		};
		this.buildRouteTree = () => {
			const result = processRouteTree(this.routeTree, this.options.caseSensitive, (route, i) => {
				route.init({ originalIndex: i });
			});
			if (this.options.routeMasks) processRouteMasks(this.options.routeMasks, result.processedTree);
			return result;
		};
		this.subscribe = (eventType, fn) => {
			const listener = {
				eventType,
				fn
			};
			this.subscribers.add(listener);
			return () => {
				this.subscribers.delete(listener);
			};
		};
		this.emit = (routerEvent) => {
			for (const listener of this.subscribers) if (listener.eventType === routerEvent.type) try {
				listener.fn(routerEvent);
			} catch (e) {
				console.error(e);
			}
		};
		this.parseLocation = (locationToParse, previousLocation) => {
			const parse = ({ pathname, search, hash, href, state }) => {
				if (!this.rewrite && !/[ \x00-\x1f\x7f\u0080-\uffff]/.test(pathname)) {
					const parsedSearch = this.options.parseSearch(search);
					const searchStr = this.options.stringifySearch(parsedSearch);
					return {
						href: pathname + searchStr + hash,
						publicHref: pathname + searchStr + hash,
						pathname: decodePath(pathname).path,
						external: false,
						searchStr,
						search: nullReplaceEqualDeep(previousLocation?.search, parsedSearch),
						hash: decodePath(hash.slice(1)).path,
						state: replaceEqualDeep(previousLocation?.state, state)
					};
				}
				const fullUrl = new URL(href, this.origin);
				const url = executeRewriteInput(this.rewrite, fullUrl);
				const parsedSearch = this.options.parseSearch(url.search);
				const searchStr = this.options.stringifySearch(parsedSearch);
				url.search = searchStr;
				return {
					href: url.href.replace(url.origin, ""),
					publicHref: href,
					pathname: decodePath(url.pathname).path,
					external: !!this.rewrite && url.origin !== this.origin,
					searchStr,
					search: nullReplaceEqualDeep(previousLocation?.search, parsedSearch),
					hash: decodePath(url.hash.slice(1)).path,
					state: replaceEqualDeep(previousLocation?.state, state)
				};
			};
			const location = parse(locationToParse);
			const { __tempLocation, __tempKey } = location.state;
			if (__tempLocation && (!__tempKey || __tempKey === this.tempLocationKey)) {
				const parsedTempLocation = parse(__tempLocation);
				parsedTempLocation.state.key = location.state.key;
				parsedTempLocation.state.__TSR_key = location.state.__TSR_key;
				delete parsedTempLocation.state.__tempLocation;
				return {
					...parsedTempLocation,
					maskedLocation: location
				};
			}
			return location;
		};
		this.resolvePathWithBase = (from, path) => {
			return resolvePath({
				base: from,
				to: path,
				trailingSlash: this.options.trailingSlash,
				cache: this.resolvePathCache
			});
		};
		this.matchRoutes = (pathnameOrNext, locationSearchOrOpts, opts) => {
			if (typeof pathnameOrNext === "string") return this.matchRoutesInternal({
				pathname: pathnameOrNext,
				search: locationSearchOrOpts
			}, opts);
			return this.matchRoutesInternal(pathnameOrNext, locationSearchOrOpts);
		};
		this.getMatchedRoutes = (pathname) => {
			const rawParams = Object.create(null);
			const match = findRouteMatch(trimPathRight(pathname), this.processedTree, true);
			if (match) Object.assign(rawParams, match.rawParams);
			return [
				match?.branch || [this.routesById["__root__"]],
				rawParams,
				match?.route
			];
		};
		this.buildLocation = (opts) => {
			const build = (dest = {}) => {
				if (dest.href) {
					const parsed = parseHref$1(dest.href, {});
					dest = {
						...dest,
						to: executeRewriteInput(this.rewrite, new URL(parsed.pathname, this.origin)).pathname,
						search: this.options.parseSearch(parsed.search),
						hash: parsed.hash.slice(1)
					};
				}
				const currentLocation = dest._fromLocation || this._pendingLocation || this.latestLocation;
				const lightweightResult = this.matchRoutesLightweight(currentLocation);
				const defaultedFromPath = dest.unsafeRelative === "path" ? currentLocation.pathname : dest.from ?? lightweightResult[1];
				const fromSearch = lightweightResult[2];
				const fromParams = lightweightResult[3];
				const nextTo = this.resolvePathWithBase(defaultedFromPath, dest.to ? `${dest.to}` : ".");
				let nextParams = resolveNextParams(dest.params, fromParams);
				const destRoute = this.routesByPath[trimPathRight(nextTo)];
				let destRoutes;
				if (destRoute) destRoutes = this.getRouteBranch(destRoute);
				else if (nextTo.includes("$")) destRoutes = [];
				else {
					const [matchedRoutes, rawParams, foundRoute] = this.getMatchedRoutes(nextTo);
					destRoutes = matchedRoutes;
					if (this.options.notFoundRoute && (!foundRoute || foundRoute.path !== "/" && rawParams["**"])) destRoutes = [...destRoutes, this.options.notFoundRoute];
				}
				if (destRoutes.length && hasKeys(nextParams)) for (const route of destRoutes) {
					const fn = route.options.params?.stringify ?? route.options.stringifyParams;
					if (fn) {
						if (nextParams === fromParams) nextParams = Object.assign(Object.create(null), nextParams);
						try {
							Object.assign(nextParams, fn(nextParams));
						} catch {}
					}
				}
				const nextPathname = opts.leaveParams ? nextTo : decodePath(interpolatePath({
					path: nextTo,
					params: nextParams,
					decoder: this.pathParamsDecoder,
					server: this.isServer
				}).interpolatedPath).path;
				let nextSearch = fromSearch;
				if (opts._includeValidateSearch && this.options.search?.strict) {
					const validatedSearch = {};
					destRoutes.forEach((route) => {
						if (route.options.validateSearch) try {
							Object.assign(validatedSearch, validateSearch(route.options.validateSearch, {
								...validatedSearch,
								...nextSearch
							}));
						} catch {}
					});
					nextSearch = validatedSearch;
				}
				nextSearch = applySearchMiddleware(nextSearch, dest, destRoutes, opts._includeValidateSearch);
				nextSearch = nullReplaceEqualDeep(fromSearch, nextSearch);
				const searchStr = this.options.stringifySearch(nextSearch);
				const hash = dest.hash === true ? currentLocation.hash : dest.hash ? functionalUpdate(dest.hash, currentLocation.hash) : void 0;
				const hashStr = hash ? `#${hash}` : "";
				let nextState = dest.state === true ? currentLocation.state : dest.state ? functionalUpdate(dest.state, currentLocation.state) : {};
				if (dest.state) nextState = replaceEqualDeep(currentLocation.state, nextState);
				const fullPath = `${nextPathname}${searchStr}${hashStr}`;
				let href;
				let publicHref;
				let external = false;
				if (this.rewrite) {
					const url = new URL(fullPath, this.origin);
					const rewrittenUrl = executeRewriteOutput(this.rewrite, url);
					href = url.href.replace(url.origin, "");
					if (rewrittenUrl.origin !== this.origin) {
						publicHref = rewrittenUrl.href;
						external = true;
					} else publicHref = rewrittenUrl.pathname + rewrittenUrl.search + rewrittenUrl.hash;
				} else {
					href = encodePathLikeUrl(fullPath);
					publicHref = href;
				}
				return {
					publicHref,
					href,
					pathname: nextPathname,
					search: nextSearch,
					searchStr,
					state: nextState,
					hash: hash ?? "",
					external,
					unmaskOnReload: dest.unmaskOnReload
				};
			};
			const next = build(opts);
			if (opts.mask) next.maskedLocation = build({
				from: opts.from,
				...opts.mask
			});
			else if (this.options.routeMasks) {
				const match = findFlatMatch(next.pathname, this.processedTree);
				if (match) {
					const params = Object.assign(Object.create(null), match.rawParams);
					const { from: _from, params: maskParams, ...maskProps } = match.route;
					const nextParams = resolveNextParams(maskParams, params);
					next.maskedLocation = build({
						from: opts.from,
						...maskProps,
						params: nextParams
					});
				}
			}
			return next;
		};
		this.commitLocation = async ({ viewTransition, ignoreBlocker, ...next }) => {
			let historyAction;
			const isSameLocation = trimPathRight(this.latestLocation.href) === trimPathRight(next.href) && deepEqual(_getUserHistoryState(next.state), _getUserHistoryState(this.latestLocation.state));
			const previousCommitPromise = this._commitPromise;
			let resolve;
			const commitPromise = new Promise((done) => {
				resolve = done;
			});
			commitPromise.resolve = () => {
				resolve();
				previousCommitPromise?.resolve();
			};
			this._commitPromise = commitPromise;
			if (isSameLocation) this.load();
			else {
				let { maskedLocation, hashScrollIntoView, ...nextHistory } = next;
				if (maskedLocation) {
					nextHistory = {
						...maskedLocation,
						state: {
							...maskedLocation.state,
							__tempKey: void 0,
							__tempLocation: {
								...nextHistory,
								search: nextHistory.searchStr,
								state: {
									...nextHistory.state,
									__tempKey: void 0,
									__tempLocation: void 0,
									__TSR_key: void 0,
									key: void 0
								}
							}
						}
					};
					if (nextHistory.unmaskOnReload ?? this.options.unmaskOnReload ?? false) nextHistory.state.__tempKey = this.tempLocationKey;
				}
				nextHistory.state.__hashScrollIntoViewOptions = hashScrollIntoView ?? this.options.defaultHashScrollIntoView ?? true;
				this.shouldViewTransition = viewTransition;
				historyAction = next.replace ? "REPLACE" : "PUSH";
				this.history[historyAction === "REPLACE" ? "replace" : "push"](nextHistory.publicHref, nextHistory.state, { ignoreBlocker });
				if (!this.history.subscribers.size) this.load({ action: { type: historyAction } });
			}
			this._scroll.next = next.resetScroll ?? true;
			return this._commitPromise;
		};
		this.buildAndCommitLocation = ({ replace, resetScroll, hashScrollIntoView, viewTransition, ignoreBlocker, ...rest } = {}) => {
			const location = this.buildLocation({
				...rest,
				_includeValidateSearch: true
			});
			this._pendingLocation = location;
			const commitPromise = this.commitLocation({
				...location,
				viewTransition,
				replace,
				resetScroll,
				hashScrollIntoView,
				ignoreBlocker
			});
			queueMicrotask(() => {
				if (this._pendingLocation === location) this._pendingLocation = void 0;
			});
			return commitPromise;
		};
		this.navigate = async ({ to, reloadDocument, href, publicHref, ...rest }) => {
			const hrefIsUrl = !!href && isAbsoluteUrl(`${href}`);
			if (hrefIsUrl && !reloadDocument) reloadDocument = true;
			if (reloadDocument) {
				if (to !== void 0 || !href) {
					const location = this.buildLocation({
						to,
						...rest
					});
					href = href ?? location.publicHref;
					publicHref = publicHref ?? location.publicHref;
				}
				const reloadHref = !hrefIsUrl && publicHref ? publicHref : href;
				if (isDangerousProtocol(reloadHref, this.protocolAllowlist)) return;
				if (!rest.ignoreBlocker) {
					const blockers = this.history.getBlockers?.() ?? [];
					for (const blocker of blockers) if (blocker?.blockerFn) {
						if (await blocker.blockerFn({
							currentLocation: this.latestLocation,
							nextLocation: this.latestLocation,
							action: "PUSH"
						})) return;
					}
				}
				if (rest.replace) window.location.replace(reloadHref);
				else window.location.href = reloadHref;
				return;
			}
			return this.buildAndCommitLocation({
				...rest,
				href,
				to,
				_isNavigate: true
			});
		};
		this.load = async (opts) => {
			this.updateLatestLocation();
			if (opts?.action) this._scroll.hash = opts.action.type === "PUSH" || opts.action.type === "REPLACE";
			await loadClientRoute(this, opts);
		};
		this.startViewTransition = (fn) => {
			const shouldViewTransition = this.shouldViewTransition ?? this.options.defaultViewTransition;
			this.shouldViewTransition = void 0;
			if (shouldViewTransition && typeof document.startViewTransition === "function") {
				let startViewTransitionParams;
				if (typeof shouldViewTransition === "object" && window.CSS?.supports?.("selector(:active-view-transition-type(a))")) {
					const next = this.latestLocation;
					const prevLocation = this.stores.resolvedLocation.get();
					const resolvedViewTransitionTypes = typeof shouldViewTransition.types === "function" ? shouldViewTransition.types(getLocationChangeInfo(next, prevLocation)) : shouldViewTransition.types;
					if (resolvedViewTransitionTypes === false) return fn();
					startViewTransitionParams = {
						update: fn,
						types: resolvedViewTransitionTypes
					};
				} else startViewTransitionParams = fn;
				return document.startViewTransition(startViewTransitionParams).updateCallbackDone;
			}
			return fn();
		};
		this.invalidate = (opts) => {
			const committedMatches = this._committed;
			const filter = opts?.filter;
			const preloads = this._preloads;
			const invalidIds = new Set([
				...committedMatches,
				...this._cache.values(),
				...[...preloads?.values() ?? []].flat(),
				...this._tx?.[3] ?? []
			].filter((match) => !filter || filter(match)).map((match) => match.id));
			const discardedPreloads = [];
			for (const [controller, matches] of preloads ?? []) if (matches.some((match) => invalidIds.has(match.id))) {
				preloads.delete(controller);
				discardedPreloads.push(controller);
			}
			const invalidate = (d) => {
				if (invalidIds.has(d.id)) {
					const route = this.routesById[d.routeId];
					const next = {
						...d,
						invalid: true,
						...(opts?.forcePending || d.status === "error" || d.status === "notFound") && routeNeedsLoad(route) ? {
							status: "pending",
							error: void 0
						} : void 0
					};
					d._flight = void 0;
					return next;
				}
				return d;
			};
			this._committed = committedMatches.map(invalidate);
			for (const [id, match] of this._cache) if (invalidIds.has(id)) {
				match.invalid = true;
				if (opts?.forcePending) match.status = "pending";
			}
			for (const id of invalidIds) this._flights?.delete(id);
			for (const controller of discardedPreloads) controller.abort();
			this.shouldViewTransition = false;
			return this.load({ sync: opts?.sync });
		};
		this.resolveRedirect = (redirect) => {
			const locationHeader = redirect.headers.get("Location");
			if (!redirect.options.href) {
				const href = this.buildLocation(redirect.options).publicHref || "/";
				redirect.options.href = href;
				redirect.headers.set("Location", href);
			} else if (locationHeader) try {
				const url = new URL(locationHeader);
				if (this.origin && url.origin === this.origin) {
					const href = url.pathname + url.search + url.hash;
					redirect.options.href = href;
					redirect.headers.set("Location", href);
				}
			} catch {}
			if (redirect.options.href && isDangerousProtocol(redirect.options.href, this.protocolAllowlist)) throw new Error("Redirect blocked: unsafe protocol");
			if (!redirect.headers.get("Location")) redirect.headers.set("Location", redirect.options.href);
			return redirect;
		};
		this.clearCache = (opts) => {
			const cached = this._cache;
			const preloads = this._preloads;
			const filter = opts?.filter;
			const discarded = [];
			const discardedIds = [];
			for (const [id, match] of cached) if (!filter || filter(match)) {
				discardedIds.push(id);
				discarded.push(match);
			}
			const abort = [];
			for (const [controller, matches] of preloads ?? []) if (!filter || matches.some(filter)) {
				abort.push(controller);
				discarded.push(...matches);
			}
			for (const id of discardedIds) cached.delete(id);
			for (const controller of abort) preloads.delete(controller);
			for (const match of discarded) {
				const flight = match._flight;
				match._flight = void 0;
				if (flight && !--flight[2]) {
					if (this._flights?.get(match.id) === flight) this._flights.delete(match.id);
					abort.push(flight[1]);
				}
			}
			for (const controller of abort) controller.abort();
		};
		this.loadRouteChunk = loadRouteChunk;
		this.preloadRoute = (opts) => preloadClientRoute(this, opts);
		this.matchRoute = (location, opts) => {
			const matchLocation = {
				...location,
				to: location.to ? this.resolvePathWithBase(location.from || "", location.to) : void 0,
				params: location.params || {},
				leaveParams: true
			};
			const next = this.buildLocation(matchLocation);
			const isPending = this.stores.status.get() === "pending";
			if (opts?.pending && !isPending) return false;
			const baseLocation = opts?.pending ?? !isPending ? this.latestLocation : this.stores.resolvedLocation.get() || this.stores.location.get();
			const match = findSingleMatch(next.pathname, opts?.caseSensitive ?? false, opts?.fuzzy ?? false, baseLocation.pathname, this.processedTree);
			if (!match) return false;
			if (location.params) {
				if (!deepEqual(match.rawParams, location.params, { partial: true })) return false;
			}
			if (opts?.includeSearch ?? true) return deepEqual(baseLocation.search, next.search, { partial: true }) ? match.rawParams : false;
			return match.rawParams;
		};
		this.getStoreConfig = getStoreConfig;
		this.update({
			defaultPreloadDelay: 50,
			defaultPendingMs: 1e3,
			defaultPendingMinMs: 500,
			context: void 0,
			...options,
			caseSensitive: options.caseSensitive ?? false,
			notFoundMode: options.notFoundMode ?? "fuzzy",
			stringifySearch: options.stringifySearch ?? defaultStringifySearch,
			parseSearch: options.parseSearch ?? defaultParseSearch,
			protocolAllowlist: options.protocolAllowlist ?? DEFAULT_PROTOCOL_ALLOWLIST
		});
		self.__TSR_ROUTER__ = this;
	}
	isShell() {
		return !!this.options.isShell;
	}
	get state() {
		return this.stores.__store.get();
	}
	setRoutes({ routesById, routesByPath, processedTree }) {
		this.routesById = routesById;
		this.routesByPath = routesByPath;
		this.processedTree = processedTree;
		const notFoundRoute = this.options.notFoundRoute;
		if (notFoundRoute) {
			notFoundRoute.init({ originalIndex: 99999999999 });
			this.routesById[notFoundRoute.id] = notFoundRoute;
		}
	}
	getRouteBranch(route) {
		let branch = this.routeBranchCache.get(route);
		if (!branch) {
			branch = buildRouteBranch(route);
			this.routeBranchCache.set(route, branch);
		}
		return branch;
	}
	matchRoutesInternal(next, opts) {
		const [initialMatchedRoutes, rawParams, foundRoute] = this.getMatchedRoutes(next.pathname);
		let matchedRoutes = initialMatchedRoutes;
		let isGlobalNotFound = false;
		if (foundRoute ? foundRoute.path !== "/" && rawParams["**"] : trimPathRight(next.pathname)) if (this.options.notFoundRoute) matchedRoutes = [...matchedRoutes, this.options.notFoundRoute];
		else isGlobalNotFound = true;
		const _notFoundRouteId = isGlobalNotFound ? findGlobalNotFoundRouteId(this.options.notFoundMode, matchedRoutes) : void 0;
		const matches = new Array(matchedRoutes.length);
		const committed = this._committed;
		const previousAt = (route, index) => {
			const match = committed[index];
			return match?.routeId === route.id ? match : route === this.options.notFoundRoute ? committed.find((candidate) => candidate.routeId === route.id) : void 0;
		};
		let strictParams;
		for (let index = 0; index < matchedRoutes.length; index++) {
			const route = matchedRoutes[index];
			const parentMatch = matches[index - 1];
			let preMatchSearch;
			let strictMatchSearch;
			let searchError;
			{
				const parentSearch = parentMatch?.search ?? next.search;
				const parentStrictSearch = parentMatch?._strictSearch ?? void 0;
				try {
					const strictSearch = validateSearch(route.options.validateSearch, { ...parentSearch }) ?? void 0;
					preMatchSearch = {
						...parentSearch,
						...strictSearch
					};
					strictMatchSearch = {
						...parentStrictSearch,
						...strictSearch
					};
				} catch (err) {
					let searchParamError = err;
					if (!(err instanceof SearchParamError)) searchParamError = new SearchParamError(err.message, { cause: err });
					if (opts?.throwOnError) throw searchParamError;
					preMatchSearch = parentSearch;
					strictMatchSearch = {};
					searchError = searchParamError;
				}
			}
			let loaderDeps = "";
			let loaderDepsHash = "";
			try {
				loaderDeps = route.options.loaderDeps?.({ search: preMatchSearch }) ?? "";
				loaderDepsHash = loaderDeps ? JSON.stringify(loaderDeps) || "" : "";
			} catch (cause) {
				if (opts?.throwOnError) throw cause;
				searchError ??= cause;
			}
			const { interpolatedPath, usedParams } = interpolatePath({
				path: route.fullPath,
				params: rawParams,
				decoder: this.pathParamsDecoder,
				server: this.isServer
			});
			const matchId = route.id + interpolatedPath + loaderDepsHash;
			const previousMatch = previousAt(route, index);
			const existingMatch = this._cache.get(matchId) ?? (previousMatch?.id === matchId ? previousMatch : void 0);
			strictParams = existingMatch?._strictParams ?? Object.assign(usedParams, strictParams);
			let paramsError;
			if (!existingMatch) try {
				extractStrictParams(route, strictParams);
			} catch (err) {
				if (isNotFound(err) || isRedirect(err)) paramsError = err;
				else paramsError = new PathParamError(err.message, { cause: err });
				if (opts?.throwOnError) throw paramsError;
			}
			const cause = previousMatch ? "stay" : "enter";
			let match;
			if (existingMatch) match = {
				...existingMatch,
				cause,
				search: previousMatch ? nullReplaceEqualDeep(previousMatch.search, preMatchSearch) : nullReplaceEqualDeep(existingMatch.search, preMatchSearch),
				_strictSearch: strictMatchSearch,
				searchError
			};
			else {
				const status = routeNeedsLoad(route) ? "pending" : "success";
				match = {
					id: matchId,
					ssr: route.options.ssr,
					index,
					routeId: route.id,
					params: previousMatch?.params ?? strictParams,
					_strictParams: strictParams,
					pathname: interpolatedPath,
					updatedAt: Date.now(),
					search: previousMatch ? nullReplaceEqualDeep(previousMatch.search, preMatchSearch) : preMatchSearch,
					_strictSearch: strictMatchSearch,
					searchError,
					status,
					isFetching: false,
					error: void 0,
					paramsError,
					context: {},
					abortController: opts?._controller ?? new AbortController(),
					cause,
					loaderDeps: previousMatch ? replaceEqualDeep(previousMatch.loaderDeps, loaderDeps) : loaderDeps,
					invalid: false,
					preload: false,
					staticData: route.options.staticData || {},
					fullPath: route.fullPath
				};
			}
			const _notFound = _notFoundRouteId === route.id;
			if (match._notFound && !_notFound) match.error = void 0;
			match._notFound = _notFound;
			matches[index] = match;
		}
		for (let index = 0; index < matches.length; index++) {
			const match = matches[index];
			match.params = match.cause === "stay" ? nullReplaceEqualDeep(match.params, strictParams) : strictParams;
			if (opts?._controller) match.context = {};
		}
		return matches;
	}
	/**
	* Lightweight route matching for buildLocation.
	* Only computes fullPath, accumulated search, and params - skipping expensive
	* operations like AbortController, loaderDeps, and full match objects.
	*/
	matchRoutesLightweight(location) {
		const lastRouteId = last(this.stores.ids.get());
		const lastStateMatch = lastRouteId ? this.stores.byRoute.get(lastRouteId).get() : void 0;
		const lastStateMatchId = lastStateMatch?.id;
		const cached = this.lightweightCache.get(location);
		if (cached && cached[0] === lastStateMatchId) return cached[1];
		const [matchedRoutes, rawParams] = this.getMatchedRoutes(location.pathname);
		const lastRoute = last(matchedRoutes);
		const accumulatedSearch = { ...location.search };
		for (const route of matchedRoutes) try {
			Object.assign(accumulatedSearch, validateSearch(route.options.validateSearch, accumulatedSearch));
		} catch {}
		const canReuseParams = lastStateMatch && lastStateMatch.routeId === lastRoute.id && lastStateMatch.pathname === location.pathname;
		let params;
		if (canReuseParams) params = lastStateMatch.params;
		else {
			const strictParams = Object.assign(Object.create(null), rawParams);
			for (const route of matchedRoutes) try {
				extractStrictParams(route, strictParams);
			} catch {}
			params = strictParams;
		}
		const result = [
			matchedRoutes,
			lastRoute.fullPath,
			accumulatedSearch,
			params
		];
		this.lightweightCache.set(location, [lastStateMatchId, result]);
		return result;
	}
};
/** Error thrown when search parameter validation fails. */
var SearchParamError = class extends Error {};
/** Error thrown when path parameter parsing/validation fails. */
var PathParamError = class extends Error {};
function validateSearch(validateSearch, input) {
	if (validateSearch == null) return {};
	if ("~standard" in validateSearch) {
		const result = validateSearch["~standard"].validate(input);
		if (result instanceof Promise) throw new SearchParamError("Async validation not supported");
		if (result.issues) throw new SearchParamError(JSON.stringify(result.issues, void 0, 2), { cause: result });
		return result.value;
	}
	if ("parse" in validateSearch) return validateSearch.parse(input);
	if (typeof validateSearch === "function") return validateSearch(input);
	return {};
}
function applySearchMiddleware(search, dest, destRoutes, includeValidateSearch) {
	const middlewares = [];
	for (const route of destRoutes) {
		const routeOptions = route.options;
		if ("search" in routeOptions) {
			if (routeOptions.search?.middlewares) middlewares.push(...routeOptions.search.middlewares);
		} else if (routeOptions.preSearchFilters || routeOptions.postSearchFilters) {
			const legacyMiddleware = ({ search, next }) => {
				const result = next(routeOptions.preSearchFilters ? routeOptions.preSearchFilters.reduce((prev, next) => next(prev), search) : search);
				return routeOptions.postSearchFilters ? routeOptions.postSearchFilters.reduce((prev, next) => next(prev), result) : result;
			};
			middlewares.push(legacyMiddleware);
		}
		const routeValidateSearch = routeOptions.validateSearch;
		if (includeValidateSearch && routeValidateSearch) {
			const validate = ({ search, next, meta }) => {
				const result = next(search);
				try {
					const validated = validateSearch(routeValidateSearch, result);
					if (meta && validated) {
						for (const key in validated) if (!(key in result)) (meta.defaulted ||= /* @__PURE__ */ new Map()).set(key, validated[key]);
					}
					return {
						...result,
						...validated
					};
				} catch {}
				return result;
			};
			middlewares.push(validate);
		}
	}
	const applyNext = (index, currentSearch, meta) => {
		if (index >= middlewares.length) {
			if (!dest.search) return {};
			if (dest.search === true) return currentSearch;
			const result = functionalUpdate(dest.search, currentSearch);
			if (meta) meta.explicit = result;
			return result;
		}
		const next = (newSearch, collectMeta) => {
			if (collectMeta) {
				const nextMeta = meta || {};
				return {
					search: applyNext(index + 1, newSearch, nextMeta),
					meta: nextMeta
				};
			}
			return applyNext(index + 1, newSearch, meta);
		};
		return middlewares[index]({
			search: currentSearch,
			next,
			meta
		});
	};
	return applyNext(0, search);
}
function findGlobalNotFoundRouteId(notFoundMode, routes) {
	if (notFoundMode !== "root") {
		let fallback;
		for (let i = routes.length - 1; i >= 0; i--) {
			const route = routes[i];
			if (route.options.notFoundComponent) return route.id;
			fallback ||= route.children && route.id;
		}
		if (fallback) return fallback;
	}
	return rootRouteId;
}
function resolveNextParams(spec, base) {
	if (spec === false || spec === null) return Object.create(null);
	if ((spec ?? true) === true) return base;
	const next = Object.assign(Object.create(null), base);
	return Object.assign(next, functionalUpdate(spec, next));
}
function extractStrictParams(route, accumulatedParams) {
	const parseParams = route.options.params?.parse ?? route.options.parseParams;
	if (parseParams) Object.assign(accumulatedParams, parseParams(accumulatedParams));
}
//#endregion
//#region node_modules/.bun/@tanstack+router-core@1.171.28/node_modules/@tanstack/router-core/dist/esm/load-client.js
function preloadComponent(route, type) {
	return route.options[type]?.preload?.();
}
function loadComponents(route, onPendingReady) {
	const component = preloadComponent(route, "component");
	let pending = preloadComponent(route, "pendingComponent");
	if (onPendingReady) if (pending) pending = pending.then(onPendingReady);
	else onPendingReady();
	if (component && pending) return Promise.all([component, pending]).then(() => {});
	return component ?? pending;
}
function loadRouteChunk(route, componentType, onPendingReady) {
	const afterLazy = () => componentType === false ? void 0 : componentType ? preloadComponent(route, componentType) : loadComponents(route, onPendingReady);
	const current = route._lazy;
	if (current) return current === true ? afterLazy() : current.then(afterLazy);
	if (!route.lazyFn) return afterLazy();
	const promise = route.lazyFn().then((lazyRoute) => {
		{
			const { id: _id, ...options } = lazyRoute.options;
			Object.assign(route.options, options);
			route._lazy = true;
		}
	}, (error) => {
		route._lazy = void 0;
		throw error;
	});
	route._lazy = promise;
	return promise.then(afterLazy);
}
/** Return the structural lane through the first terminal render boundary. */
function _getRenderedMatches(matches) {
	const end = matches.findIndex((match) => match.status !== "success" || match._notFound) + 1;
	return end && end < matches.length ? matches.slice(0, end) : matches;
}
var SUCCESS = 0;
var ERROR = 1;
var NOT_FOUND = 2;
var REDIRECTED = 3;
var CANCELED_OUTCOME = [4];
function isControl(result) {
	return typeof result[0] === "number";
}
function waitFor(value, signal) {
	if (signal.aborted) return Promise.race([Promise.reject(signal), value]);
	return new Promise((resolve, reject) => {
		const abort = () => reject(signal);
		signal.addEventListener("abort", abort, { once: true });
		Promise.resolve(value).then(resolve, reject).then(() => signal.removeEventListener("abort", abort));
	});
}
function getRoute(router, match) {
	return router.routesById[match.routeId];
}
function normalize(value, rejected, routeId) {
	if (isRedirect(value)) return [REDIRECTED, value];
	if (isNotFound(value)) {
		value.routeId ||= routeId;
		return [NOT_FOUND, value];
	}
	if (!rejected) return [SUCCESS, value];
	if (typeof value?.then === "function") value = new Error("A Promise was thrown", { cause: value });
	return [ERROR, value];
}
function normalizeError(route, cause) {
	let outcome = normalize(cause, true, route.id);
	if (outcome[0] !== ERROR) return outcome;
	try {
		route.options.onError?.(outcome[1]);
	} catch (onErrorCause) {
		outcome = normalize(onErrorCause, true, route.id);
	}
	return outcome;
}
function normalizeLaneError(router, lane, route, cause, options) {
	if (options[0].signal.aborted) return CANCELED_OUTCOME;
	return materializeRedirect(router, lane, route, normalizeError(route, cause), options);
}
async function contextualize(router, lane, options, end, planSuccessfulLane, retainedEnd) {
	const [location, matches] = lane;
	const signal = options[0].signal;
	const preload = !!options[3];
	for (let index = options[6] ?? 0; index < end; index++) {
		const match = matches[index];
		const route = getRoute(router, match);
		match.abortController = options[0];
		const parentContext = matches[index - 1]?.context ?? router.options.context ?? {};
		const common = {
			params: match.params,
			location,
			navigate: (opts) => router.navigate({
				...opts,
				_fromLocation: location
			}),
			buildLocation: router.buildLocation,
			cause: preload ? "preload" : match.cause,
			abortController: options[0],
			preload,
			matches,
			routeId: route.id
		};
		try {
			const routeContext = match._ctx ||= route.options.context ? route.options.context({
				...common,
				deps: match.loaderDeps,
				context: parentContext
			}) || {} : void 0;
			match.context = {
				...parentContext,
				...routeContext
			};
		} catch (cause) {
			releaseFlight(router, match);
			return [index, normalizeLaneError(router, lane, route, cause, options)];
		}
		if (signal.aborted) return [index, CANCELED_OUTCOME];
		const validationError = match.paramsError ?? match.searchError;
		if (validationError !== void 0) {
			releaseFlight(router, match);
			return [index, normalizeLaneError(router, lane, route, validationError, options)];
		}
		const beforeLoad = route.options.beforeLoad;
		if (!beforeLoad) continue;
		const previousStatus = match.status;
		if (index >= retainedEnd) {
			match.status = "pending";
			options[7]?.();
		}
		try {
			setFetching(router, match, "beforeLoad", options[0]);
			const value = beforeLoad({
				...common,
				search: match.search,
				context: match.context,
				...router.options.additionalContext
			});
			const result = await (typeof value?.then === "function" ? waitFor(value, signal) : value);
			if (signal.aborted) return [index, CANCELED_OUTCOME];
			const outcome = materializeRedirect(router, lane, route, normalize(result, false, route.id), options);
			if (outcome[0] !== SUCCESS) {
				releaseFlight(router, match);
				return [index, outcome];
			}
			match.context = {
				...match.context,
				...result
			};
		} catch (cause) {
			releaseFlight(router, match);
			return [index, normalizeLaneError(router, lane, route, cause, options)];
		} finally {
			match.status = previousStatus;
			setFetching(router, match, false, options[0]);
		}
	}
	planSuccessfulLane();
}
function releaseOwnedFlight(router, match, flight) {
	if (!flight || --flight[2]) return;
	if (router._flights?.get(match.id) === flight) {
		const current = router._tx;
		if (current && !current[0].signal.aborted && !current[3].includes(match) && current[3].some((candidate) => candidate.id === match.id) && current[3].some((candidate) => candidate.isFetching === "beforeLoad")) return;
		router._flights.delete(match.id);
	}
	return flight[1];
}
function releaseFlight(router, match) {
	const flight = match._flight;
	match._flight = void 0;
	releaseOwnedFlight(router, match, flight)?.abort();
}
/**
* Not passing in a `next` ownership recipient
* is equivalent to discarding the match resources
*/
function transferMatchResources(router, previous, next, deferSameIdFlight) {
	const abort = [];
	for (const match of previous) if (!next?.includes(match)) {
		const flight = match._flight;
		match._flight = void 0;
		if (deferSameIdFlight && flight?.[2] === 1 && router._flights?.get(match.id) === flight && next?.some((candidate) => candidate.id === match.id)) flight[2] = 0;
		else {
			const controller = releaseOwnedFlight(router, match, flight);
			if (controller) abort.push(controller);
		}
	}
	for (const controller of abort) controller.abort();
}
function acquireMatchResources(matches) {
	for (const match of matches) {
		const flight = match._flight;
		if (flight) flight[2]++;
	}
}
function setFetching(router, match, value, owner) {
	match.isFetching = value;
	if (owner && router._tx?.[0] !== owner) return;
	const store = router.stores.byRoute.get(match.routeId);
	const presented = store?.get();
	if (presented?.id === match.id) store.set({
		...presented,
		isFetching: value
	});
}
function getLoaderContext(router, lane, match, route, controller, parentMatchPromise, preload) {
	const location = lane[0];
	return {
		params: match.params,
		location,
		navigate: (opts) => router.navigate({
			...opts,
			_fromLocation: location
		}),
		cause: preload ? "preload" : match.cause,
		abortController: controller,
		preload,
		deps: match.loaderDeps,
		parentMatchPromise,
		context: match.context,
		route,
		...router.options.additionalContext
	};
}
async function loadResource(router, lane, match, route, loader, parentMatchPromise, options) {
	const owner = options[0];
	const signal = owner.signal;
	if (signal.aborted) return CANCELED_OUTCOME;
	if (!loader) return [SUCCESS, void 0];
	let flight = match._flight;
	setFetching(router, match, "loader", owner);
	try {
		if (!flight) {
			const controller = new AbortController();
			flight = [
				Promise.resolve().then(() => loader(getLoaderContext(router, lane, match, route, controller, parentMatchPromise, !!options[3]))).then((value) => normalize(value, false, route.id), (cause) => normalize(cause, true, route.id)).then((result) => {
					if (result[0] !== SUCCESS && router._flights?.get(match.id) === flight) {
						router._flights.delete(match.id);
						if (!flight[2]) controller.abort();
					}
					return result[0] === ERROR && flight[2] ? normalizeError(route, result[1]) : result;
				}),
				controller,
				1
			];
			(router._flights ??= /* @__PURE__ */ new Map()).set(match.id, flight);
		}
		match._flight = flight;
		match.abortController = flight[1];
		return materializeRedirect(router, lane, route, await waitFor(flight[0], signal), options);
	} catch (cause) {
		if (cause !== signal || !signal.aborted) throw cause;
		releaseFlight(router, match);
		return CANCELED_OUTCOME;
	} finally {
		setFetching(router, match, false, owner);
	}
}
function settleInto(match, result, preload) {
	if (result[0] === REDIRECTED) return;
	match.status = "success";
	match.error = void 0;
	if (result[0] === SUCCESS) {
		match.loaderData = result[1];
		match.invalid = false;
		match.updatedAt = Date.now();
		match.preload = preload;
	} else match.invalid = true;
}
function cacheLoaderMatch(router, match, planned) {
	const current = router._cache.get(match.id);
	if (current !== planned || router._committed.some((candidate) => candidate.id === match.id && candidate._flight === match._flight)) return;
	const cached = {
		...match,
		_notFound: void 0,
		context: {}
	};
	if (cached._flight) cached._flight[2]++;
	router._cache.set(match.id, cached);
	if (current) releaseFlight(router, current);
}
function getParentSnapshot(match, outcome) {
	if (outcome[0] === ERROR || outcome[0] === NOT_FOUND) return {
		...match,
		status: outcome[0] === ERROR ? "error" : "notFound",
		error: outcome[1],
		_flight: void 0
	};
	return match;
}
function createLoaderTask(router, lane, index, tasks, semanticParent, options, retainedEnd) {
	const match = lane[1][index];
	const route = getRoute(router, match);
	const preload = !!options[3];
	const plannedCacheMatch = router._cache.get(match.id);
	let configured;
	let reload = false;
	let reloadFailure;
	try {
		if (match.status === "success") {
			configured = route.options.shouldReload;
			if (typeof configured === "function") configured = configured(getLoaderContext(router, lane, match, route, options[0], semanticParent, preload));
			if (options[0].signal.aborted) reloadFailure = CANCELED_OUTCOME;
		}
		if (!reloadFailure) if (match.status !== "success") reload = true;
		else {
			const staleAge = preload || match.preload ? route.options.preloadStaleTime ?? router.options.defaultPreloadStaleTime ?? 3e4 : route.options.staleTime ?? router.options.defaultStaleTime ?? 0;
			reload = !!(match.invalid || configured || configured === void 0 && Date.now() - match.updatedAt >= staleAge && (options[5] || match.cause === "enter" || options[2].some((candidate) => candidate.routeId === match.routeId && candidate.id !== match.id)));
		}
	} catch (cause) {
		match.invalid = true;
		releaseFlight(router, match);
		reloadFailure = normalizeLaneError(router, lane, route, cause, options);
	}
	const routeLoader = route.options.loader;
	const isLoaderFn = typeof routeLoader === "function";
	const loader = isLoaderFn ? routeLoader : routeLoader?.handler;
	const preloadable = !preload || route.options.preload !== false;
	let donor = preloadable && routeLoader && true ? router._flights?.get(match.id) : void 0;
	if (donor === match._flight || reloadFailure) donor = void 0;
	else if (donor && !reload && !preload && configured === void 0) reload = true;
	else if (!reload) donor = void 0;
	const background = !!(routeLoader && reload && match.status === "success" && !preload && !options[4] && ((isLoaderFn ? void 0 : routeLoader.staleReloadMode) ?? router.options.defaultStaleReloadMode) !== "blocking");
	const loaded = reload && preloadable;
	const blocking = loaded && !background && (match.status !== "success" || !!routeLoader);
	const onReady = index >= retainedEnd ? options[7] : void 0;
	const onLazyReady = route.lazyFn && route._lazy !== true ? onReady : void 0;
	if (loaded && !routeLoader) {
		match.invalid = false;
		match.updatedAt = Date.now();
	}
	if (donor) donor[2]++;
	if (blocking) {
		const acceptedFlight = match._flight;
		match._flight = donor;
		releaseOwnedFlight(router, match, acceptedFlight)?.abort();
		if (index >= retainedEnd) match.status = "pending";
		onReady?.();
	}
	if (!loaded) match.isFetching = false;
	const outcome = !reloadFailure && blocking ? loadResource(router, lane, match, route, loader, semanticParent, options).then((result) => {
		settleInto(match, result, preload);
		if (result[0] === SUCCESS) {
			if (routeLoader && !options[0].signal.aborted) cacheLoaderMatch(router, match, plannedCacheMatch);
			if (index >= retainedEnd) match.status = "pending";
		}
		return result;
	}) : Promise.resolve(reloadFailure ?? [SUCCESS, match.loaderData]);
	const chunkFailure = (async () => {
		try {
			const chunk = loadRouteChunk(route, void 0, onLazyReady);
			if (chunk) await waitFor(chunk, options[0].signal);
		} catch (cause) {
			if (!lane[1].some((candidate, candidateIndex) => candidateIndex <= index && (candidate.status === "error" || candidate.status === "notFound" || candidate._notFound))) return [index, normalizeLaneError(router, lane, route, cause, options)];
		}
		const result = await outcome;
		if (blocking && result[0] === SUCCESS && match.status === "pending" && !options[0].signal.aborted) {
			match.status = "success";
			onReady?.();
		}
	})();
	tasks.push([
		index,
		outcome,
		chunkFailure
	]);
	if (!background) return outcome.then((result) => getParentSnapshot(match, result));
	const candidate = {
		...match,
		status: "pending",
		preload: false,
		_flight: donor
	};
	match.invalid = false;
	match.isFetching = "loader";
	const backgroundOutcome = loadResource(router, lane, candidate, route, loader, semanticParent, options).then((result) => {
		match.isFetching = false;
		settleInto(candidate, result, false);
		return result;
	});
	(lane[2] ??= []).push([
		index,
		backgroundOutcome,
		chunkFailure,
		candidate
	]);
	return backgroundOutcome.then((result) => getParentSnapshot(candidate, result));
}
async function getNotFoundBoundary(router, matches, indexed, signal, fallback = 0) {
	const cause = indexed?.[1][1];
	let index = cause?.routeId ? matches.findIndex((match) => match.routeId === cause.routeId) : indexed?.[0] ?? matches.length - 1;
	if (index < 0) index = 0;
	for (let i = index; i >= 0; i--) {
		const route = getRoute(router, matches[i]);
		try {
			const loading = loadRouteChunk(route, false);
			if (loading) await waitFor(loading, signal);
		} catch (cause) {
			if (cause === signal && signal.aborted) throw cause;
		}
		if (route.options.notFoundComponent) return i;
	}
	return cause?.routeId ? index : fallback;
}
function discardBackground(router, lane) {
	if (lane[2]) {
		transferMatchResources(router, lane[2].map((task) => task[3]));
		lane[2] = void 0;
	}
}
async function settleTasks(tasks, serialFailure, redirectTasks, gate) {
	let loaderFailure;
	try {
		await Promise.all(tasks.map((task) => task[1].then(async (outcome) => {
			const taskIndex = task[0];
			if (gate && taskIndex >= await gate) return;
			if (outcome[0] >= REDIRECTED) throw [taskIndex, outcome];
			if (!loaderFailure && outcome[0] !== SUCCESS) {
				loaderFailure = [taskIndex, outcome];
				await Promise.all((redirectTasks ?? []).map((nextTask) => {
					if (nextTask[0] <= taskIndex) return;
					return nextTask[1].then((nextOutcome) => {
						if (nextOutcome[0] === REDIRECTED) throw [nextTask[0], nextOutcome];
					});
				}));
			}
		})));
	} catch (cause) {
		return cause;
	}
	return serialFailure ?? loaderFailure;
}
function materializeRedirect(router, lane, route, outcome, options, failed) {
	while (outcome[0] === REDIRECTED) {
		const redirect = outcome[1];
		const redirectOptions = redirect.options;
		if (redirectOptions.reloadDocument ? options[3] : options[1] >= 20) return outcome;
		try {
			if (redirectOptions.href && redirectOptions.reloadDocument) {
				router.resolveRedirect(redirect);
				return outcome;
			}
			return [
				REDIRECTED,
				redirect,
				router.buildLocation({
					...redirectOptions,
					_fromLocation: lane[0],
					_includeValidateSearch: true
				})
			];
		} catch (cause) {
			outcome = failed ? [ERROR, cause] : normalizeError(route, cause);
			failed = true;
		}
	}
	return outcome;
}
async function reduceLane(router, lane, tasks, controller, settlement, onReady) {
	const matches = lane[1];
	let failure = await settlement;
	let redirectLimitExceeded = false;
	const plannedBoundary = matches.findIndex((match) => match._notFound);
	const boundaryOf = (found) => found[1][0] === NOT_FOUND ? getNotFoundBoundary(router, matches, found, controller.signal) : found[0];
	let readinessEnd = plannedBoundary < 0 ? matches.length : plannedBoundary;
	if ((failure?.[1][0] ?? 0) >= REDIRECTED) readinessEnd = 0;
	else if (failure) {
		readinessEnd = failure[2] ??= await boundaryOf(failure);
		for (const task of tasks) {
			if (task[0] >= readinessEnd) break;
			const outcome = await task[1];
			if (outcome[0] !== SUCCESS && outcome[0] < REDIRECTED && !("loaderData" in matches[task[0]])) {
				failure = [task[0], outcome];
				readinessEnd = failure[2] = await boundaryOf(failure);
				break;
			}
		}
	}
	for (const task of tasks) {
		if (task[0] >= readinessEnd) break;
		const chunkFailure = await task[2];
		if (!chunkFailure) continue;
		failure = chunkFailure;
		break;
	}
	if ((failure?.[1][0] ?? 0) >= REDIRECTED) {
		const outcome = failure[1];
		if (outcome[0] !== REDIRECTED || outcome[1].options.reloadDocument || outcome[2]) {
			discardBackground(router, lane);
			return outcome;
		}
		redirectLimitExceeded = true;
		failure = [0, [ERROR, /* @__PURE__ */ new Error("Too many redirects")]];
	}
	const boundary = failure ? failure[2] ?? await boundaryOf(failure) : plannedBoundary;
	if (boundary >= 0) {
		const outcome = failure?.[1];
		const kind = outcome?.[0];
		const match = matches[boundary];
		const cause = outcome?.[1];
		const install = () => {
			if (outcome) {
				match._notFound = void 0;
				if (kind === ERROR) match.status = "error";
				else {
					cause.routeId = match.routeId;
					if (match.routeId === router.routeTree.id) {
						match.status = "success";
						match._notFound = true;
					} else match.status = "notFound";
				}
				match.error = cause;
				match.isFetching = false;
			}
		};
		install();
		if (!outcome) onReady?.();
		const route = getRoute(router, match);
		try {
			await waitFor(outcome ? Promise.resolve().then(() => loadRouteChunk(route, kind === ERROR ? "errorComponent" : "notFoundComponent")) : Promise.all([loadRouteChunk(route), loadRouteChunk(route, "notFoundComponent")]), controller.signal);
		} catch (cause) {
			if (cause === controller.signal && controller.signal.aborted) {
				discardBackground(router, lane);
				return CANCELED_OUTCOME;
			}
		}
		if (!outcome) match.status = "success";
		else if (redirectLimitExceeded) {
			controller.abort();
			await Promise.all([
				...tasks.map((task) => task[1]),
				...tasks.map((task) => task[2]),
				...(lane[2] ?? []).map((task) => task[1])
			]);
			discardBackground(router, lane);
			transferMatchResources(router, matches);
			install();
		}
	}
	return lane;
}
async function projectLane(router, lane, signal, start = 0, end = lane[1].length) {
	const matches = lane[1];
	for (let index = start; index < end; index++) {
		const match = matches[index];
		const routeOptions = getRoute(router, match).options;
		if (routeOptions.head || routeOptions.scripts) try {
			const context = {
				ssr: router.options.ssr,
				matches,
				match,
				params: match.params,
				loaderData: match.loaderData
			};
			const [head, scripts] = await waitFor(Promise.all([routeOptions.head?.(context), routeOptions.scripts?.(context)]), signal);
			match.meta = head?.meta;
			match.links = head?.links;
			match.headScripts = head?.scripts;
			match.styles = head?.styles;
			match.scripts = scripts;
		} catch (cause) {
			if (cause === signal && signal.aborted) break;
			console.error(cause);
		}
		if (match.status !== "success" || match._notFound) break;
	}
	return lane;
}
async function executeClientLane(router, location, matches, options) {
	const matched = [location, matches];
	const signal = options[0].signal;
	let reduced;
	try {
		const presented = router.stores.matches.get();
		let plannedBoundary = matches.findIndex((match) => match._notFound);
		if (router.options.notFoundMode !== "root" && plannedBoundary >= 0) {
			const boundary = await getNotFoundBoundary(router, matches, void 0, signal, plannedBoundary);
			matches[plannedBoundary]._notFound = void 0;
			matches[boundary]._notFound = true;
			plannedBoundary = boundary;
		}
		let end = plannedBoundary < 0 ? matches.length : plannedBoundary + 1;
		let retainedEnd = 0;
		while (retainedEnd < end && retainedEnd !== plannedBoundary) {
			const match = matches[retainedEnd];
			const committed = options[2][retainedEnd];
			const visible = presented[retainedEnd];
			if (committed?.id !== match.id || committed.status !== "success" || match.preload || visible?.id !== match.id || visible.status !== "success") break;
			retainedEnd++;
			if (committed._notFound || visible._notFound) break;
		}
		const tasks = [];
		const start = options[6] ?? 0;
		let semanticParent = start ? Promise.resolve(matches[start - 1]) : void 0;
		const planSuccessfulLane = () => {
			for (let index = start; index < end; index++) {
				if (signal.aborted) break;
				semanticParent = createLoaderTask(router, matched, index, tasks, semanticParent, options, retainedEnd);
			}
		};
		const failure = await contextualize(router, matched, options, end, planSuccessfulLane, retainedEnd);
		if (failure) {
			options[4] = true;
			end = failure[0];
			if (failure[1][0] === NOT_FOUND) {
				const boundary = await getNotFoundBoundary(router, matches, failure, signal);
				failure[2] = boundary;
				end = Math.min(end, boundary + 1);
			} else if (failure[1][0] >= REDIRECTED) end = 0;
			planSuccessfulLane();
		}
		if (!signal.aborted && !options[3]) {
			const abort = [];
			for (const [id, flight] of router._flights ?? []) if (!flight[2]) {
				router._flights.delete(id);
				abort.push(flight[1]);
			}
			for (const controller of abort) controller.abort();
		}
		const reduction = reduceLane(router, matched, tasks, options[0], settleTasks(tasks, failure, matched[2]), options[7]);
		if (matched[2]?.length) matched[3] = settleTasks(matched[2], void 0, void 0, reduction.then((foreground) => isControl(foreground) ? 0 : _getRenderedMatches(matches).length, () => 0));
		reduced = await reduction;
	} catch (cause) {
		discardBackground(router, matched);
		if (cause === signal && signal.aborted) return CANCELED_OUTCOME;
		throw cause;
	}
	if (isControl(reduced)) return reduced;
	return projectLane(router, reduced, signal, options[6] === matches.length ? options[6] : 0);
}
/**
* Waits for `pendingMs`, then presents the complete lane. Rendering applies the
* selected boundary cutoff while retaining every match's structural state.
* A replacement load for the same match keeps the timer; choosing a different
* match resets it. `pendingMinMs` starts after the fallback renders.
*/
function offerPending(router, tx) {
	if (router._tx !== tx) return;
	const matches = tx[3];
	const presented = router.stores.matches.get();
	let session = router._pending;
	for (let index = 0; index < matches.length; index++) {
		const match = matches[index];
		const success = match.status === "success" && !match._notFound;
		const presentedPending = presented[index]?.id === match.id && presented[index]?.status === "pending";
		if (success && !presentedPending) continue;
		const route = getRoute(router, match);
		const delay = success || match.invalid ? 0 : route.options.pendingMs ?? router.options.defaultPendingMs;
		const component = route.options.pendingComponent ?? router.options.defaultPendingComponent;
		if (!component || typeof delay !== "number" || delay === Infinity) {
			if (session) {
				session[0] = tx;
				session[2] = 0;
				session[4] = true;
			}
			return;
		}
		const min = route.options.pendingMinMs ?? router.options.defaultPendingMinMs ?? 0;
		let tookOver = false;
		if (session?.[1] === match.id) {
			tookOver = session[0] !== tx;
			session[0] = tx;
		} else {
			clearTimeout(session?.[3]);
			router._pending = session = void 0;
		}
		if (!session) router._pending = session = [
			tx,
			match.id,
			presentedPending ? Date.now() + min : tx[4] + delay,
			void 0,
			presentedPending || void 0,
			component
		];
		if (session[4] && !tookOver && session[5] === component) return;
		session[5] = component;
		if (!session[4]) {
			clearTimeout(session[3]);
			const remaining = session[2] - Date.now();
			if (remaining > 0) {
				session[3] = setTimeout(() => offerPending(router, tx), remaining);
				return;
			}
			session[2] = 0;
		}
		const offered = matches.map((match) => ({
			...match,
			_flight: void 0
		}));
		offered[index].status = "pending";
		const ack = session[4] = router.startTransition(() => router.stores.setMatches(offered), offered).then((rendered) => {
			if (rendered && router._pending === session && session[4] === ack && !session[2]) session[2] = Date.now() + min;
			return rendered;
		});
		return;
	}
}
/**
* Cancels pending UI timing unless the current successor can take over the
* same boundary that remains painted.
*/
function finishPending(router, tx) {
	const session = router._pending;
	if (router._tx === tx || !router._tx?.[3].some((match) => match.id === session?.[1])) {
		clearTimeout(session?.[3]);
		router._pending = void 0;
	}
}
async function awaitPendingMinimum(router, tx) {
	const session = router._pending;
	if (!session) return;
	clearTimeout(session[3]);
	const remaining = session[2] - Date.now();
	if (!session[4] || remaining <= 0 || !_getRenderedMatches(tx[3]).some((match) => match.id === session[1])) return;
	let timer;
	try {
		await waitFor(new Promise((resolve) => {
			timer = setTimeout(resolve, remaining);
		}), tx[0].signal);
	} catch {}
	clearTimeout(timer);
}
function publishMatches(router, matches) {
	router._committed = matches;
	router.stores.setMatches(matches);
}
function commitMatches(router, tx, matches, resolvedPrefix) {
	const previous = router._committed;
	const previousEnd = router._lifecycleEnd;
	const previousCached = router._cache;
	for (const match of matches) {
		match.preload = false;
		if (resolvedPrefix) match._assetEnd = void 0;
	}
	const cut = _getRenderedMatches(matches).length;
	const cached = /* @__PURE__ */ new Map();
	{
		const now = Date.now();
		for (const match of [...previous, ...previousCached.values()]) {
			if (match.status !== "success" || matches.some((candidate, index) => candidate.id === match.id && (index < cut || candidate.status === "success"))) continue;
			const route = getRoute(router, match);
			if (!route.options.loader || now - match.updatedAt >= (match.preload ? route.options.preloadGcTime ?? router.options.defaultPreloadGcTime ?? 3e5 : route.options.gcTime ?? router.options.defaultGcTime ?? 3e5)) continue;
			cached.set(match.id, previousCached.get(match.id) === match ? match : {
				...match,
				_flight: void 0,
				isFetching: false,
				context: {}
			});
		}
	}
	tx[3] = [];
	router._cache = cached;
	const nextEnd = router._lifecycleEnd = lifecycleEnd(matches);
	publishMatches(router, matches);
	transferMatchResources(router, [...previousCached.values(), ...previous].filter((match) => match._flight && cached.get(match.id) !== match), matches);
	runRouteLifecycle(router, previous, matches, previousEnd, nextEnd, tx);
}
/**
* Resolve once the router has settled on a transaction other than `owner`.
* Each transaction's completion follows its current successor, so all waiters
* share the same chain instead of polling every successor independently.
*/
async function awaitCurrent(router, owner) {
	let current = router._tx;
	while (current && current !== owner) {
		owner = current;
		await current[5];
		current = router._tx;
	}
}
function followRedirect(router, tx, outcome) {
	const options = outcome[1].options;
	const location = outcome[2];
	if (!location) return router.navigate({
		...options,
		replace: true,
		ignoreBlocker: true
	});
	if (options.reloadDocument) return router.navigate({
		href: location.publicHref,
		reloadDocument: true,
		replace: true,
		ignoreBlocker: true
	});
	location._redirects = tx[1] + 1;
	router._pendingLocation = location;
	const committed = router.commitLocation({
		...location,
		viewTransition: options.viewTransition,
		replace: true,
		resetScroll: options.resetScroll,
		hashScrollIntoView: options.hashScrollIntoView,
		ignoreBlocker: true
	});
	queueMicrotask(() => {
		if (router._pendingLocation === location) router._pendingLocation = void 0;
	});
	return committed;
}
async function runBackground(router, tx, base, tasks, settlement) {
	const next = base.map((match) => ({ ...match }));
	acquireMatchResources(next);
	for (const task of tasks) {
		releaseFlight(router, next[task[0]]);
		next[task[0]] = task[3];
	}
	const lane = [tx[2], next];
	let reduced;
	try {
		reduced = await reduceLane(router, lane, tasks, tx[0], settlement);
	} catch (cause) {
		transferMatchResources(router, next);
		throw cause;
	}
	if (isControl(reduced)) {
		transferMatchResources(router, next);
		if (reduced[0] === REDIRECTED && router._tx === tx && router._committed === base) await followRedirect(router, tx, reduced);
		return;
	}
	await projectLane(router, reduced, tx[0].signal);
	if (router._tx !== tx || router._committed !== base) {
		transferMatchResources(router, next);
		return;
	}
	for (const match of next) {
		const cached = router._cache.get(match.id);
		if (cached?._flight && cached._flight === match._flight) {
			router._cache.delete(match.id);
			releaseFlight(router, cached);
		}
	}
	publishMatches(router, next);
	transferMatchResources(router, base, next);
}
async function runClientTransaction(router, tx, forceStaleReload, onReady, sync, resolvedPrefix) {
	const result = await executeClientLane(router, tx[2], tx[3], [
		tx[0],
		tx[1],
		router._committed,
		void 0,
		sync,
		forceStaleReload,
		resolvedPrefix,
		onReady
	]);
	if (isControl(result)) {
		const follow = result[0] === REDIRECTED && router._tx === tx;
		if (!follow || result[1].options.reloadDocument) finishPending(router, tx);
		transferMatchResources(router, tx[3]);
		tx[3] = [];
		if (!follow) return;
		if (router._tx !== tx) {
			finishPending(router, tx);
			return;
		}
		await followRedirect(router, tx, result);
		return;
	}
	const matches = result[1];
	if (router._tx === tx) await awaitPendingMinimum(router, tx);
	if (router._tx !== tx) {
		finishPending(router, tx);
		transferMatchResources(router, matches);
		discardBackground(router, result);
		return;
	}
	const toLocation = tx[2];
	const changeInfo = getLocationChangeInfo(toLocation, router.stores.resolvedLocation.get());
	const background = result[2];
	await router.startViewTransition(async () => {
		if (router._tx === tx) await awaitPendingMinimum(router, tx);
		if (router._tx !== tx) {
			finishPending(router, tx);
			transferMatchResources(router, matches);
			discardBackground(router, result);
			return;
		}
		const commit = () => {
			finishPending(router, tx);
			commitMatches(router, tx, matches, resolvedPrefix);
			if (router._tx !== tx) return;
			router.emit({
				type: "onLoad",
				...changeInfo
			});
			if (router._tx === tx) router.emit({
				type: "onBeforeRouteMount",
				...changeInfo
			});
		};
		const rendered = await router.startTransition(commit, matches);
		if (router._tx !== tx) {
			discardBackground(router, result);
			return;
		}
		if (background?.length) runBackground(router, tx, matches, background, result[3]).catch(console.error);
		router.batch(() => {
			router.stores.resolvedLocation.set(toLocation);
			router.stores.status.set("idle");
			if (router._tx === tx) router.emit({
				type: "onResolved",
				...changeInfo
			});
			if (rendered && router._tx === tx) router.emit({
				type: "onRendered",
				...changeInfo
			});
		});
		if (router._tx !== tx) return;
		router._commitPromise?.resolve();
		router._commitPromise = void 0;
	});
}
async function loadClientRoute(router, opts) {
	const previousOwner = router._tx;
	const resolvedLocation = router.stores.resolvedLocation.get();
	const previousLocation = resolvedLocation ?? router.stores.location.get();
	const location = router.latestLocation;
	const pendingLocation = router._pendingLocation;
	const redirects = pendingLocation?.href === location.href ? pendingLocation._redirects ?? 0 : 0;
	const handoff = router._handoff;
	const hydrationController = handoff?.[0]();
	const preflight = new AbortController();
	const previousPreflight = router._preflight;
	router._preflight = preflight;
	if (!hydrationController) handoff?.[1]();
	previousPreflight?.abort();
	if (!preflight.signal.aborted) {
		const changeInfo = getLocationChangeInfo(location, resolvedLocation);
		router.emit({
			type: "onBeforeNavigate",
			...changeInfo
		});
		if (!preflight.signal.aborted) router.emit({
			type: "onBeforeLoad",
			...changeInfo
		});
	}
	if (preflight.signal.aborted) {
		await awaitCurrent(router, previousOwner);
		return;
	}
	const sameHref = previousLocation.href === location.href;
	let controller = preflight;
	const matches = router.matchRoutes(location, { _controller: preflight });
	acquireMatchResources(matches);
	const resolvedPrefix = hydrationController ? handoff[1](matches) : void 0;
	if (resolvedPrefix) controller = hydrationController;
	else hydrationController?.abort();
	if (preflight.signal.aborted) {
		transferMatchResources(router, matches);
		await awaitCurrent(router, previousOwner);
		return;
	}
	router._preflight = void 0;
	let settle;
	const run = () => runClientTransaction(router, tx, sameHref, () => offerPending(router, tx), opts?.sync, resolvedPrefix);
	const done = opts?.sync ? new Promise((resolve) => settle = resolve) : Promise.resolve().then(run);
	const tx = [
		controller,
		redirects,
		location,
		matches,
		Date.now(),
		done.then(() => awaitCurrent(router, tx))
	];
	router._tx = tx;
	if (previousOwner) {
		for (const match of router.stores.matches.get()) {
			if (router._tx !== tx) break;
			if (match.isFetching) setFetching(router, match, false);
		}
		previousOwner[0].abort();
		transferMatchResources(router, previousOwner[3], tx[3], true);
	}
	if (router._tx !== tx) {
		transferMatchResources(router, tx[3]);
		tx[3] = [];
		settle?.();
		await awaitCurrent(router, tx);
		return;
	}
	router.batch(() => {
		router.stores.status.set("pending");
		router.stores.location.set(location);
	});
	if (resolvedPrefix || !router._committed.length && matches[0]?.status !== "success" && !matches.some((match) => match._notFound)) offerPending(router, tx);
	settle?.(run());
	await tx[5];
}
async function preloadClientRoute(router, opts) {
	let location = router.buildLocation(opts);
	for (let redirects = 0;; redirects++) {
		const base = router._committed;
		const controller = new AbortController();
		let matches;
		let active;
		let result;
		try {
			try {
				matches = router.matchRoutes(location, { _controller: controller });
				acquireMatchResources(matches);
				active = (router._preloads ??= /* @__PURE__ */ new Map()).set(controller, matches);
				result = await executeClientLane(router, location, matches, [
					controller,
					redirects,
					base,
					true
				]);
			} finally {
				if (active) {
					active = active.delete(controller);
					transferMatchResources(router, matches);
				}
				controller.abort();
			}
			if (!isControl(result)) return result[1];
			if (!active || result.length < 3 || false) return;
			location = result[2];
		} catch (cause) {
			if (!isNotFound(cause)) console.error(cause);
			return;
		}
	}
}
//#endregion
//#region node_modules/.bun/@tanstack+router-core@1.171.28/node_modules/@tanstack/router-core/dist/esm/route.js
var BaseRoute = class {
	get to() {
		return this._to;
	}
	get id() {
		return this._id;
	}
	get path() {
		return this._path;
	}
	get fullPath() {
		return this._fullPath;
	}
	constructor(options) {
		this.init = (opts) => {
			this.originalIndex = opts.originalIndex;
			const options = this.options;
			const isRoot = !options?.path && !options?.id;
			this.parentRoute = this.options.getParentRoute?.();
			if (isRoot) this._path = rootRouteId;
			else if (!this.parentRoute) invariant();
			let path = isRoot ? rootRouteId : options?.path;
			if (path && path !== "/") path = trimPathLeft(path);
			const customId = options?.id || path;
			let id = isRoot ? rootRouteId : joinPaths([this.parentRoute.id === "__root__" ? "" : this.parentRoute.id, customId]);
			if (path === "__root__") path = "/";
			if (id !== "__root__") id = joinPaths(["/", id]);
			const fullPath = id === "__root__" ? "/" : joinPaths([this.parentRoute.fullPath, path]);
			this._path = path;
			this._id = id;
			this._fullPath = fullPath;
			this._to = trimPathRight(fullPath);
		};
		this.addChildren = (children) => {
			return this._addFileChildren(children);
		};
		this._addFileChildren = (children) => {
			if (Array.isArray(children)) this.children = children;
			if (typeof children === "object" && children !== null) this.children = Object.values(children);
			return this;
		};
		this._addFileTypes = () => {
			return this;
		};
		this.updateLoader = (options) => {
			Object.assign(this.options, options);
			return this;
		};
		this.update = (options) => {
			Object.assign(this.options, options);
			return this;
		};
		this.lazy = (lazyFn) => {
			this.lazyFn = lazyFn;
			return this;
		};
		this.redirect = (opts) => redirect({
			from: this.fullPath,
			...opts
		});
		this.options = options || {};
		this.isRoot = !options?.getParentRoute;
		if (options?.id && options?.path) throw new Error(`Route cannot have both an 'id' and a 'path' option.`);
	}
};
var BaseRootRoute = class extends BaseRoute {
	constructor(options) {
		super(options);
	}
};
//#endregion
//#region node_modules/.bun/@tanstack+history@1.162.1/node_modules/@tanstack/history/dist/esm/index.js
var stateIndexKey = "__TSR_index";
/**
* Sanitize a path to prevent open redirect vulnerabilities.
* Removes control characters and collapses leading double slashes.
*/
function sanitizePath(path) {
	let sanitized = path.replace(/[\x00-\x1f\x7f]/g, "");
	if (sanitized.startsWith("//")) sanitized = "/" + sanitized.replace(/^\/+/, "");
	return sanitized;
}
function parseHref(href, state) {
	const sanitizedHref = sanitizePath(href);
	const hashIndex = sanitizedHref.indexOf("#");
	const searchIndex = sanitizedHref.indexOf("?");
	const addedKey = createRandomKey();
	return {
		href: sanitizedHref,
		pathname: sanitizedHref.substring(0, hashIndex > 0 ? searchIndex > 0 ? Math.min(hashIndex, searchIndex) : hashIndex : searchIndex > 0 ? searchIndex : sanitizedHref.length),
		hash: hashIndex > -1 ? sanitizedHref.substring(hashIndex) : "",
		search: searchIndex > -1 ? sanitizedHref.slice(searchIndex, hashIndex === -1 ? void 0 : hashIndex) : "",
		state: state || {
			[stateIndexKey]: 0,
			key: addedKey,
			__TSR_key: addedKey
		}
	};
}
function createRandomKey() {
	return (Math.random() + 1).toString(36).substring(7);
}
//#endregion
//#region packages/solid-gpui-router/src/history.ts
/** Native history owns the stack and checks every transition before publishing it. */
var NativeHistory = class {
	entries;
	index;
	registrations = /* @__PURE__ */ new Set();
	pending;
	destroyed = false;
	subscribers = /* @__PURE__ */ new Set();
	constructor(initialEntries) {
		if (initialEntries.length === 0) throw new TypeError("initialEntries must contain at least one route");
		this.entries = initialEntries.map((href, index) => this.createLocation(href, void 0, index));
		this.index = this.entries.length - 1;
	}
	get location() {
		return this.entries[this.index];
	}
	get length() {
		return this.entries.length;
	}
	subscribe = (subscriber) => {
		this.subscribers.add(subscriber);
		return () => {
			this.subscribers.delete(subscriber);
		};
	};
	block = (blocker) => {
		this.assertActive();
		const registration = { blocker };
		this.registrations.add(registration);
		return () => {
			this.registrations.delete(registration);
			if (this.pending?.registrations.has(registration)) this.pending.cancel();
		};
	};
	assertActive() {
		if (this.destroyed) throw new Error("Cannot navigate a destroyed native history");
	}
	createLocation(href, state, index) {
		const location = parseHref(href, void 0);
		location.state = {
			...state,
			...location.state,
			__TSR_index: index
		};
		return location;
	}
	/** A newer request or removed registration invalidates an outstanding decision. */
	transition(nextLocation, action, options, commit) {
		this.assertActive();
		this.pending?.cancel();
		const registrations = options?.ignoreBlocker ? [] : [...this.registrations];
		if (registrations.length === 0) return Promise.resolve(commit());
		const currentLocation = this.location;
		return new Promise((resolve, reject) => {
			const pending = {
				registrations: new Set(registrations),
				cancel: () => {
					if (this.pending !== pending) return;
					this.pending = void 0;
					resolve();
				}
			};
			this.pending = pending;
			const check = async () => {
				for (const { blocker } of registrations) {
					if (this.pending !== pending) return;
					const blocked = await blocker.blockerFn({
						currentLocation,
						nextLocation,
						action: action.type
					});
					if (this.pending !== pending) return;
					if (blocked) {
						pending.cancel();
						return;
					}
				}
				this.pending = void 0;
				await commit();
				resolve();
			};
			check().catch((error) => {
				if (this.pending === pending) this.pending = void 0;
				reject(error);
			});
		});
	}
	/** RouterCore's synchronous commit starts only after the native decision settles. */
	commitRouterLocation(href, state, replace, options, commit) {
		const next = this.createLocation(href, state, this.index + (replace ? 0 : 1));
		return this.transition(next, { type: replace ? "REPLACE" : "PUSH" }, options, commit);
	}
	push = (href, state, options) => {
		const next = this.createLocation(href, state, this.index + 1);
		return this.transition(next, { type: "PUSH" }, options, () => {
			this.entries.splice(this.index + 1, this.entries.length, next);
			this.index++;
			this.notify({ type: "PUSH" });
		});
	};
	replace = (href, state, options) => {
		const next = this.createLocation(href, state, this.index);
		return this.transition(next, { type: "REPLACE" }, options, () => {
			this.entries[this.index] = next;
			this.notify({ type: "REPLACE" });
		});
	};
	move(delta, action, options) {
		if (!Number.isInteger(delta)) throw new TypeError("History delta must be an integer");
		const index = Math.min(Math.max(this.index + delta, 0), this.entries.length - 1);
		if (index === this.index) {
			this.assertActive();
			this.pending?.cancel();
			return Promise.resolve();
		}
		return this.transition(this.entries[index], action, options, () => {
			this.index = index;
			this.notify(action);
		});
	}
	go = (delta, options) => this.move(delta, {
		type: "GO",
		index: delta
	}, options);
	back = (options) => this.move(-1, { type: "BACK" }, options);
	forward = (options) => this.move(1, { type: "FORWARD" }, options);
	canGoBack = () => this.index > 0;
	createHref = (href) => href;
	flush = () => {};
	notify = (action) => {
		const location = this.location;
		for (const subscriber of this.subscribers) subscriber({
			location,
			action
		});
	};
	destroy = () => {
		this.destroyed = true;
		this.pending?.cancel();
		this.registrations.clear();
		this.subscribers.clear();
	};
};
//#endregion
//#region packages/solid-gpui-router/src/router-core.ts
/** Adapt the asynchronous native navigation boundary before RouterCore starts loaders. */
var NativeRouterCore = class extends RouterCore {
	constructor(options, storeFactory) {
		super(options, storeFactory);
		const commitLocation = this.commitLocation;
		this.commitLocation = (next) => {
			const destination = next.maskedLocation ?? next;
			return this.history.commitRouterLocation(destination.publicHref, destination.state, next.replace ?? false, { ignoreBlocker: next.ignoreBlocker }, () => commitLocation({
				...next,
				ignoreBlocker: true
			}));
		};
	}
};
//#endregion
//#region packages/solid-gpui-router/src/index.ts
var PENDING_STYLE = { padding: 16 };
var ROUTER_ROOT_STYLE = {
	flexDirection: "column",
	flexGrow: 1,
	minWidth: 0,
	minHeight: 0
};
var MESSAGE_STYLE = {
	gap: 10,
	padding: 16
};
var TITLE_STYLE = {
	fontSize: 16,
	fontWeight: "semibold"
};
var RETRY_STYLE = {
	padding: 8,
	borderWidth: 1,
	borderColor: "#808080",
	borderRadius: 6
};
function DefaultPendingComponent() {
	return createComponent(View, {
		style: PENDING_STYLE,
		children: createComponent(Text, { children: "Loading…" })
	});
}
function DefaultErrorComponent(props) {
	const message = props.error instanceof Error ? props.error.message : String(props.error);
	return createComponent(View, {
		style: MESSAGE_STYLE,
		children: [
			createComponent(Text, {
				style: TITLE_STYLE,
				children: "Navigation failed"
			}),
			createComponent(Text, { children: message }),
			createComponent(Pressable, {
				accessibilityRole: "button",
				accessibilityLabel: "Retry navigation",
				style: RETRY_STYLE,
				onPress: () => props.reset(),
				children: createComponent(Text, { children: "Retry" })
			})
		]
	});
}
function DefaultNotFoundComponent(props) {
	return createComponent(View, {
		style: MESSAGE_STYLE,
		children: [createComponent(Text, {
			style: TITLE_STYLE,
			children: "Route not found"
		}), createComponent(Text, { children: () => String(props.routeId) })]
	});
}
function createRoute(options) {
	return new BaseRoute(options);
}
function createRootRoute(options) {
	return new BaseRootRoute(options);
}
function createNativeMutableStore(initialValue) {
	const [get, set] = createSignal(initialValue);
	return {
		get,
		set
	};
}
function createNativeReadonlyStore(read) {
	return { get: createRoot$1(() => createMemo(read)) };
}
var nativeStoreFactory = () => ({
	createMutableStore: createNativeMutableStore,
	createReadonlyStore: createNativeReadonlyStore,
	batch
});
var nativeRouters = /* @__PURE__ */ new WeakSet();
var mountedRouters = /* @__PURE__ */ new WeakSet();
var disableNativeScrollRestoration = () => false;
function createRouter(options) {
	const { initialEntries = ["/"], ...routerOptions } = options;
	const history = new NativeHistory(initialEntries);
	const router = new NativeRouterCore({
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
		defaultNotFoundComponent: routerOptions.defaultNotFoundComponent ?? DefaultNotFoundComponent
	}, nativeStoreFactory);
	const nativeOptions = {
		...router.options,
		scrollRestoration: disableNativeScrollRestoration
	};
	router.update(nativeOptions);
	nativeRouters.add(router);
	return router;
}
var RouterContext = createContext();
var MatchContext = createContext();
var UniversalRouterContextProvider = RouterContext.Provider;
var UniversalMatchContextProvider = MatchContext.Provider;
function useRouter() {
	const router = useContext(RouterContext);
	if (router === void 0) throw new Error("useRouter must be used under RouterProvider");
	return router;
}
function useLocation(options) {
	const router = useRouter();
	return createMemo(() => {
		const location = router.stores.__store.get().location;
		return options?.select ? options.select(location) : location;
	});
}
function RouteError(props) {
	return createComponent(props.component, {
		error: props.error,
		reset: () => {
			props.router.invalidate();
		}
	});
}
function NativeMatch(props) {
	const match = () => props.router.stores.getMatchStore(props.routeId).get();
	const route = props.router.routesById[props.routeId];
	const status = createMemo(() => match()?.status);
	const render = () => {
		const currentStatus = status();
		if (currentStatus === void 0 || currentStatus === "pending") return createComponent(route.options.pendingComponent ?? props.router.options.defaultPendingComponent ?? DefaultPendingComponent, {});
		if (currentStatus === "error") return createComponent(RouteError, {
			component: route.options.errorComponent ?? props.router.options.defaultErrorComponent ?? DefaultErrorComponent,
			get error() {
				return match()?.error;
			},
			router: props.router
		});
		if (currentStatus === "notFound") return createComponent(route.options.notFoundComponent ?? props.router.options.defaultNotFoundComponent ?? DefaultNotFoundComponent, {
			get data() {
				const error = match()?.error;
				return isNotFound(error) ? error.data : void 0;
			},
			isNotFound: true,
			routeId: props.routeId
		});
		return createComponent(route.options.component ?? props.router.options.defaultComponent ?? Outlet, {});
	};
	return createComponent(UniversalMatchContextProvider, {
		value: match,
		get children() {
			return createMemo(render);
		}
	});
}
function Outlet() {
	const router = useRouter();
	const routeId = useContext(MatchContext)?.()?.routeId;
	if (routeId === void 0) throw new Error("Outlet must be rendered by a route component");
	const childRouteId = createMemo(() => {
		const routeIds = router.stores.ids.get();
		return routeIds[routeIds.indexOf(routeId) + 1];
	});
	return (() => {
		const id = childRouteId();
		return id === void 0 ? null : createComponent(NativeMatch, {
			routeId: id,
			router
		});
	});
}
function RouterLifecycle(props) {
	const [failure, setFailure] = createSignal();
	let settleCurrent;
	props.router.startTransition = (publish) => {
		settleCurrent?.(false);
		return new Promise((resolve, reject) => {
			const settle = (rendered) => {
				if (settleCurrent !== settle) return;
				settleCurrent = void 0;
				resolve(rendered);
			};
			const fail = (error) => {
				if (settleCurrent !== settle) return;
				settleCurrent = void 0;
				reject(error);
			};
			settleCurrent = settle;
			startTransition(() => publish()).then(() => settle(true), fail);
		});
	};
	const load = () => {
		props.router.load().catch((error) => setFailure(() => error));
	};
	createEffect(() => {
		const error = failure();
		if (error !== void 0) throw error;
	});
	onMount(() => {
		if (mountedRouters.has(props.router)) throw new Error("a native router instance can only be mounted in one Solid GPUI root");
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
function RouterProvider(props) {
	if (!nativeRouters.has(props.router)) throw new TypeError("RouterProvider requires a router created by @solid-gpui/router");
	return createComponent(UniversalRouterContextProvider, {
		value: props.router,
		get children() {
			createComponent(RouterLifecycle, { router: props.router });
			const rootRouteId = createMemo(() => props.router.stores.__store.get().matches[0]?.routeId);
			return createComponent(View, {
				style: ROUTER_ROOT_STYLE,
				children: () => {
					const routeId = rootRouteId();
					return routeId === void 0 ? createComponent(DefaultPendingComponent, {}) : createComponent(NativeMatch, {
						routeId,
						router: props.router
					});
				}
			});
		}
	});
}
//#endregion
//#region examples/website/src/showcase/reference/model.ts
function createStudioProject(count = 240) {
	return {
		duration: 120,
		tracks: Array.from({ length: count }, (_, index) => ({
			id: `track-${index}`,
			name: `${[
				"Picture",
				"Dialogue",
				"Music",
				"Captions"
			][index % 4]} ${index + 1}`,
			clips: [{
				id: `clip-${index}-a`,
				label: `Opening ${index + 1}`,
				start: 4,
				duration: 12
			}, {
				id: `clip-${index}-b`,
				label: `Take ${index + 1}`,
				start: 24 + index % 6 * 4,
				duration: 10
			}]
		}))
	};
}
function locate(project, id) {
	for (const track of project.tracks) {
		const clip = track.clips.find((item) => item.id === id);
		if (clip) return {
			track,
			clip
		};
	}
	throw new Error(`Unknown clip: ${id}`);
}
function editClip(project, id, edit) {
	locate(project, id);
	if (!edit.label.trim() || !Number.isFinite(edit.start) || !Number.isFinite(edit.duration) || edit.duration < 1) throw new Error("Enter a title, a finite start, and a duration of at least one second.");
	const duration = Math.min(project.duration, edit.duration);
	const clip = {
		id,
		label: edit.label,
		start: Math.max(0, Math.min(project.duration - duration, edit.start)),
		duration
	};
	return {
		...project,
		tracks: project.tracks.map((track) => track.clips.some((item) => item.id === id) ? {
			...track,
			clips: track.clips.map((item) => item.id === id ? clip : item)
		} : track)
	};
}
function moveClip(project, id, destination, start) {
	const { clip } = locate(project, id);
	if (!project.tracks.some((track) => track.id === destination)) throw new Error(`Unknown track: ${destination}`);
	const edited = editClip(project, id, {
		...clip,
		start
	});
	const next = locate(edited, id).clip;
	return {
		...edited,
		tracks: edited.tracks.map((track) => {
			if (track.id === destination) return {
				...track,
				clips: [...track.clips.filter((item) => item.id !== id), next]
			};
			return track.clips.some((item) => item.id === id) ? {
				...track,
				clips: track.clips.filter((item) => item.id !== id)
			} : track;
		})
	};
}
function reorderTrack(project, id, before) {
	if (id === before) return project;
	const source = project.tracks.find((track) => track.id === id);
	if (!source || !project.tracks.some((track) => track.id === before)) throw new Error("Unknown reorder target");
	const tracks = project.tracks.filter((track) => track.id !== id);
	tracks.splice(tracks.findIndex((track) => track.id === before), 0, source);
	return {
		...project,
		tracks
	};
}
function createHistory(count = 1e4) {
	return Array.from({ length: count }, (_, index) => ({
		id: `history-${index}`,
		author: [
			"Maya",
			"Noah",
			"Ari",
			"Lin"
		][index % 4],
		subject: `Review ${index + 1}: ${[
			"Opening cut",
			"Sound mix",
			"Caption pass",
			"Delivery notes"
		][index % 4]}`,
		paragraphs: [`Review ${index + 1} — The opening cut is ready for another look.`, ...Array.from({ length: index % 3 + 1 }, (_, paragraph) => `Note ${paragraph + 1}: Keep the dialogue clear and the transition gentle. 字幕 😀 é`)]
	}));
}
//#endregion
//#region examples/website/src/showcase/reference/state.ts
function createReferenceStudioState() {
	const [project, setProject] = createSignal(createStudioProject());
	const [history, setHistory] = createSignal(createHistory());
	const [selectedClipId, setSelectedClipId] = createSignal("clip-0-a");
	const [selectedHistoryId, setSelectedHistoryId] = createSignal("history-0");
	const [title, setTitle] = createSignal("Opening 1");
	const [start, setStart] = createSignal("4");
	const [duration, setDuration] = createSignal("12");
	const [draft, setDraft] = createSignal("Review 字幕 😀 é");
	const [query, setQuery] = createSignal("");
	const [status, setStatus] = createSignal("Ready for review");
	const [inspectorOpen, setInspectorOpen] = createSignal(false);
	const [zoom, setZoom] = createSignal(1);
	const [pan, setPan] = createSignal(0);
	const selectedClip = createMemo(() => project().tracks.flatMap((track) => track.clips).find((clip) => clip.id === selectedClipId()));
	const selectedHistory = createMemo(() => history().find((entry) => entry.id === selectedHistoryId()));
	const visibleHistory = createMemo(() => {
		const needle = query().trim().toLowerCase();
		return needle ? history().filter((entry) => `${entry.author} ${entry.subject} ${entry.paragraphs.join(" ")}`.toLowerCase().includes(needle)) : history();
	});
	const selectClip = (id) => {
		const clip = project().tracks.flatMap((track) => track.clips).find((item) => item.id === id);
		if (!clip) throw new Error(`Unknown clip: ${id}`);
		batch(() => {
			setSelectedClipId(id);
			setTitle(clip.label);
			setStart(String(clip.start));
			setDuration(String(clip.duration));
		});
	};
	const apply = () => {
		try {
			setProject(editClip(project(), selectedClipId(), {
				label: title(),
				start: Number(start()),
				duration: Number(duration())
			}));
			setStatus(`Saved ${selectedClip().label}`);
		} catch (error) {
			setStatus(String(error));
		}
	};
	const drop = (type, target) => {
		if (type.startsWith("studio-track:")) {
			setProject(reorderTrack(project(), type.slice(13), target));
			setStatus(`Moved ${type.slice(13)} before ${target}`);
		} else if (type.startsWith("studio-clip:")) {
			const id = type.slice(12);
			const clip = project().tracks.flatMap((track) => track.clips).find((item) => item.id === id);
			if (!clip) throw new Error(`Unknown clip: ${id}`);
			batch(() => {
				setProject(moveClip(project(), id, target, clip.start));
				selectClip(id);
				setStatus(`Moved ${id} to ${target}`);
			});
		}
	};
	let nextNote = 0;
	const post = () => {
		if (!draft().trim()) return;
		const entry = {
			id: `note-${nextNote++}`,
			author: "You",
			subject: "New review note",
			paragraphs: [draft()]
		};
		batch(() => {
			setHistory((entries) => [entry, ...entries]);
			setSelectedHistoryId(entry.id);
			setDraft("");
			setQuery("");
			setStatus("Review note posted");
		});
	};
	return {
		project,
		history,
		selectedClip,
		selectedHistory,
		selectedClipId,
		selectedHistoryId,
		setSelectedHistoryId,
		title,
		setTitle,
		start,
		setStart,
		duration,
		setDuration,
		draft,
		setDraft,
		query,
		setQuery,
		visibleHistory,
		status,
		setStatus,
		inspectorOpen,
		setInspectorOpen,
		zoom,
		setZoom,
		pan,
		setPan,
		selectClip,
		apply,
		drop,
		post
	};
}
//#endregion
//#region examples/website/src/showcase/ReferenceStudio.tsx
var studioPalette = {
	background: "#131217",
	panel: "#1B1A20",
	raised: "#222127",
	border: "#3C3944",
	text: "#ECEAF1",
	muted: "#B5B1BF",
	accent: "#D4688C"
};
function Action(props) {
	return createComponent(Pressable, {
		get accessibilityLabel() {
			return props.label;
		},
		accessibilityRole: "button",
		focusable: true,
		get disabled() {
			return props.disabled;
		},
		get onPress() {
			return props.onPress;
		},
		get style() {
			return {
				padding: 7,
				minHeight: 32,
				flexShrink: 0,
				borderRadius: 5,
				borderWidth: 1,
				borderColor: props.active ? props.palette.accent : props.palette.border,
				backgroundColor: props.palette.raised
			};
		},
		get children() {
			return createComponent(Text, {
				get style() {
					return {
						fontSize: 12,
						lineHeight: 18,
						color: props.palette.text
					};
				},
				get children() {
					return props.text;
				}
			});
		}
	});
}
function ReferenceStudio(props) {
	const s = props.state;
	const p = () => props.palette ?? studioPalette;
	const height = () => props.height ?? 620;
	const narrow = () => props.width < 720;
	const inspectorWidth = () => narrow() ? s.inspectorOpen() ? Math.min(240, props.width - 120) : 0 : 220;
	const contentWidth = () => Math.max(120, props.width - inspectorWidth() - 2);
	const timelineWidth = () => narrow() ? props.view === "timeline" ? contentWidth() : 0 : Math.floor(contentWidth() * (props.view === "timeline" ? .62 : .38));
	const historyWidth = () => contentWidth() - timelineWidth();
	let tracks;
	let history;
	let active = true;
	onCleanup(() => {
		active = false;
		tracks = void 0;
		history = void 0;
	});
	const command = (pending) => {
		pending.catch((error) => {
			if (active) s.setStatus(String(error));
		});
	};
	const inputStyle = () => ({
		height: 34,
		padding: 6,
		color: p().text,
		backgroundColor: p().background,
		borderWidth: 1,
		borderColor: p().border,
		fontSize: 13,
		lineHeight: 20
	});
	const paneStyle = (width) => ({
		width,
		height: height() - 96,
		minWidth: 0,
		minHeight: 0,
		flexShrink: 0,
		overflow: "hidden",
		flexDirection: "column",
		backgroundColor: p().panel
	});
	function TrackRow(row) {
		props.onRowLifetime?.("track", row.track.id, true);
		onCleanup(() => props.onRowLifetime?.("track", row.track.id, false));
		const gridWidth = () => Math.max(80, timelineWidth() - 100);
		const scale = () => gridWidth() / 120 * s.zoom();
		return createComponent(View, {
			get accessibilityLabel() {
				return `studio.track.${row.track.id}`;
			},
			accessibilityRole: "listitem",
			onDrop: (type) => s.drop(type, row.track.id),
			get style() {
				return {
					height: 64,
					flexShrink: 0,
					flexDirection: "row",
					borderBottomWidth: 1,
					borderColor: p().border
				};
			},
			get children() {
				return [createComponent(Pressable, {
					get accessibilityLabel() {
						return `studio.reorder.${row.track.id}`;
					},
					get draggable() {
						return { type: `studio-track:${row.track.id}` };
					},
					get style() {
						return {
							width: 100,
							flexShrink: 0,
							padding: 8,
							backgroundColor: p().raised,
							cursor: "grab"
						};
					},
					get children() {
						return [createComponent(Text, {
							get style() {
								return {
									color: p().text,
									fontSize: 11,
									lineHeight: 16
								};
							},
							get children() {
								return row.track.name;
							}
						}), createComponent(Text, {
							get style() {
								return {
									color: p().muted,
									fontSize: 10,
									lineHeight: 14
								};
							},
							children: "Drag to reorder"
						})];
					}
				}), createComponent(View, {
					get style() {
						return {
							width: gridWidth(),
							height: 64,
							position: "relative",
							overflow: "hidden"
						};
					},
					get children() {
						return createComponent(For, {
							get each() {
								return row.track.clips;
							},
							children: (clip) => createComponent(Pressable, {
								get accessibilityLabel() {
									return `studio.clip.${clip.id}`;
								},
								get accessibilitySelected() {
									return s.selectedClipId() === clip.id;
								},
								get draggable() {
									return { type: `studio-clip:${clip.id}` };
								},
								onPress: () => s.selectClip(clip.id),
								get style() {
									return {
										position: "absolute",
										left: (clip.start - s.pan()) * scale(),
										top: 10,
										width: Math.max(24, clip.duration * scale()),
										height: 42,
										padding: 4,
										borderRadius: 4,
										borderWidth: s.selectedClipId() === clip.id ? 2 : 1,
										borderColor: s.selectedClipId() === clip.id ? p().accent : p().border,
										backgroundColor: p().raised,
										overflow: "hidden",
										cursor: "grab"
									};
								},
								get children() {
									return createComponent(Text, {
										get style() {
											return {
												color: p().text,
												fontSize: 11,
												lineHeight: 15
											};
										},
										get children() {
											return clip.label;
										}
									});
								}
							})
						});
					}
				})];
			}
		});
	}
	function HistoryRow(row) {
		props.onRowLifetime?.("history", row.entry.id, true);
		onCleanup(() => props.onRowLifetime?.("history", row.entry.id, false));
		return createComponent(View, {
			get accessibilityLabel() {
				return `studio.history.${row.entry.id}`;
			},
			accessibilityRole: "listitem",
			get style() {
				return {
					padding: 10,
					gap: 5,
					flexShrink: 0,
					borderBottomWidth: 1,
					borderColor: p().border,
					backgroundColor: s.selectedHistoryId() === row.entry.id ? p().raised : p().panel
				};
			},
			get children() {
				return [createComponent(Pressable, {
					get accessibilityLabel() {
						return `studio.open.${row.entry.id}`;
					},
					onPress: () => s.setSelectedHistoryId(row.entry.id),
					get children() {
						return createComponent(Text, {
							get style() {
								return {
									color: p().accent,
									fontSize: 12,
									lineHeight: 18
								};
							},
							get children() {
								return [
									memo(() => {
										return row.entry.author;
									}),
									" · ",
									memo(() => {
										return row.entry.subject;
									})
								];
							}
						});
					}
				}), createComponent(For, {
					get each() {
						return row.entry.paragraphs;
					},
					children: (paragraph, index) => createComponent(Text, {
						selectable: true,
						get accessibilityLabel() {
							return `studio.text.${row.entry.id}.${index()}`;
						},
						get style() {
							return {
								color: p().text,
								fontSize: 12,
								lineHeight: 18
							};
						},
						children: paragraph
					})
				})];
			}
		});
	}
	return createComponent(View, {
		accessibilityLabel: "studio.shell",
		get style() {
			return {
				width: props.width,
				height: height(),
				minWidth: 0,
				minHeight: 0,
				flexDirection: "column",
				backgroundColor: p().background,
				color: p().text
			};
		},
		get children() {
			return [
				createComponent(View, {
					accessibilityLabel: "studio.navigation",
					style: {
						height: 54,
						padding: 8,
						flexDirection: "row",
						alignItems: "center",
						gap: 8,
						flexShrink: 0
					},
					get children() {
						return [
							createComponent(Action, {
								label: "studio.nav.timeline",
								text: "Timeline",
								get active() {
									return props.view === "timeline";
								},
								onPress: () => props.navigate("timeline"),
								get palette() {
									return p();
								}
							}),
							createComponent(Action, {
								label: "studio.nav.history",
								text: "History",
								get active() {
									return props.view === "history";
								},
								onPress: () => props.navigate("history"),
								get palette() {
									return p();
								}
							}),
							createComponent(Text, {
								get style() {
									return {
										flexGrow: 1,
										minWidth: 0,
										color: p().muted,
										fontSize: 12
									};
								},
								children: "Cut review"
							}),
							createComponent(Action, {
								label: "studio.inspector.toggle",
								text: "Inspector",
								onPress: () => s.setInspectorOpen(!s.inspectorOpen()),
								get palette() {
									return p();
								}
							})
						];
					}
				}),
				createComponent(View, {
					get style() {
						return {
							flexDirection: "row",
							height: height() - 96,
							minHeight: 0,
							gap: 1
						};
					},
					get children() {
						return [
							createComponent(View, {
								accessibilityLabel: "studio.timeline.pane",
								get style() {
									return paneStyle(timelineWidth());
								},
								get children() {
									return [
										createComponent(View, {
											style: {
												height: 44,
												padding: 6,
												flexDirection: "row",
												gap: 6,
												overflow: "hidden",
												flexShrink: 0
											},
											get children() {
												return [
													createComponent(Action, {
														label: "studio.tracks.first",
														text: "First",
														onPress: () => tracks && command(tracks.scrollToIndex(0)),
														get palette() {
															return p();
														}
													}),
													createComponent(Action, {
														label: "studio.tracks.last",
														text: "Last",
														onPress: () => tracks && command(tracks.scrollToEnd()),
														get palette() {
															return p();
														}
													}),
													createComponent(Action, {
														label: "studio.zoom",
														get text() {
															return `${s.zoom()}×`;
														},
														onPress: () => s.setZoom(s.zoom() === 1 ? 2 : 1),
														get palette() {
															return p();
														}
													}),
													createComponent(Action, {
														label: "studio.pan",
														get text() {
															return s.pan() ? "0s" : "30s";
														},
														onPress: () => s.setPan(s.pan() ? 0 : 30),
														get palette() {
															return p();
														}
													})
												];
											}
										}),
										createComponent(View, {
											style: {
												height: 28,
												flexShrink: 0,
												paddingLeft: 100,
												flexDirection: "row",
												justifyContent: "space-between",
												overflow: "hidden"
											},
											get children() {
												return [
													0,
													30,
													60,
													90
												].map((second) => createComponent(Text, {
													get style() {
														return {
															color: p().muted,
															fontSize: 10
														};
													},
													get children() {
														return [memo(() => {
															return second / s.zoom() + s.pan();
														}), "s"];
													}
												}));
											}
										}),
										createComponent(VirtualList, {
											accessibilityLabel: "studio.tracks",
											ref: (handle) => {
												tracks = handle;
											},
											get data() {
												return s.project().tracks;
											},
											itemKey: (track) => track.id,
											estimatedItemSize: 64,
											initialNumToRender: 8,
											overscan: 2,
											get style() {
												return {
													height: height() - 168,
													minHeight: 0,
													flexDirection: "column"
												};
											},
											renderItem: (track) => createComponent(TrackRow, { track })
										})
									];
								}
							}),
							createComponent(View, {
								accessibilityLabel: "studio.history.pane",
								get style() {
									return paneStyle(historyWidth());
								},
								get children() {
									return [
										createComponent(View, {
											style: {
												padding: 6,
												gap: 6,
												flexShrink: 0
											},
											get children() {
												return [createComponent(TextInput, {
													accessibilityLabel: "studio.history.search",
													placeholder: "Search reviews",
													get value() {
														return s.query();
													},
													get onChangeText() {
														return s.setQuery;
													},
													get style() {
														return inputStyle();
													}
												}), createComponent(View, {
													style: {
														flexDirection: "row",
														gap: 6
													},
													get children() {
														return [
															createComponent(Action, {
																label: "studio.history.first",
																text: "First",
																onPress: () => history && command(history.scrollToIndex(0)),
																get palette() {
																	return p();
																}
															}),
															createComponent(Action, {
																label: "studio.history.last",
																text: "Last",
																onPress: () => history && command(history.scrollToEnd()),
																get palette() {
																	return p();
																}
															}),
															createComponent(Text, {
																accessibilityLabel: "studio.history.count",
																get style() {
																	return {
																		color: p().muted,
																		fontSize: 11
																	};
																},
																get children() {
																	return [memo(() => {
																		return s.visibleHistory().length;
																	}), " reviews"];
																}
															})
														];
													}
												})];
											}
										}),
										createComponent(VirtualList, {
											accessibilityLabel: "studio.history.list",
											ref: (handle) => {
												history = handle;
											},
											get data() {
												return s.visibleHistory();
											},
											itemKey: (entry) => entry.id,
											estimatedItemSize: 140,
											initialNumToRender: 5,
											overscan: 2,
											get style() {
												return {
													height: height() - 300,
													minHeight: 0,
													flexDirection: "column"
												};
											},
											renderItem: (entry) => createComponent(HistoryRow, { entry }),
											get emptyState() {
												return createComponent(Text, {
													get style() {
														return {
															color: p().muted,
															padding: 10
														};
													},
													children: "No reviews match this search."
												});
											}
										}),
										createComponent(View, {
											style: {
												padding: 6,
												gap: 6,
												flexShrink: 0
											},
											get children() {
												return [createComponent(TextInput, {
													accessibilityLabel: "studio.note.draft",
													multiline: true,
													get value() {
														return s.draft();
													},
													get onChangeText() {
														return s.setDraft;
													},
													get style() {
														return {
															...inputStyle(),
															height: 70
														};
													}
												}), createComponent(Action, {
													label: "studio.note.post",
													text: "Post review",
													get disabled() {
														return !s.draft().trim();
													},
													onPress: () => {
														s.post();
														if (history) command(history.scrollToIndex(0));
													},
													get palette() {
														return p();
													}
												})];
											}
										})
									];
								}
							}),
							createComponent(View, {
								accessibilityLabel: "studio.inspector",
								get style() {
									return {
										...paneStyle(inspectorWidth()),
										overflow: "scroll"
									};
								},
								get children() {
									return createComponent(View, {
										style: {
											padding: 10,
											gap: 8,
											flexShrink: 0
										},
										get children() {
											return [
												createComponent(Text, {
													get style() {
														return {
															color: p().text,
															fontSize: 14
														};
													},
													children: "Clip inspector"
												}),
												createComponent(Text, {
													get style() {
														return {
															color: p().muted,
															fontSize: 11
														};
													},
													get children() {
														return s.selectedClipId();
													}
												}),
												createComponent(TextInput, {
													accessibilityLabel: "studio.clip.title",
													get value() {
														return s.title();
													},
													get onChangeText() {
														return s.setTitle;
													},
													get style() {
														return inputStyle();
													}
												}),
												createComponent(Text, {
													get style() {
														return {
															color: p().muted,
															fontSize: 11
														};
													},
													children: "Start / duration (seconds)"
												}),
												createComponent(TextInput, {
													accessibilityLabel: "studio.clip.start",
													get value() {
														return s.start();
													},
													get onChangeText() {
														return s.setStart;
													},
													get style() {
														return inputStyle();
													}
												}),
												createComponent(TextInput, {
													accessibilityLabel: "studio.clip.duration",
													get value() {
														return s.duration();
													},
													get onChangeText() {
														return s.setDuration;
													},
													get style() {
														return inputStyle();
													}
												}),
												createComponent(Action, {
													label: "studio.clip.save",
													text: "Save clip",
													get onPress() {
														return s.apply;
													},
													get palette() {
														return p();
													}
												}),
												memo(() => {
													return props.preview?.();
												}),
												createComponent(Text, {
													get style() {
														return {
															color: p().text,
															fontSize: 14
														};
													},
													children: "Selected review"
												}),
												createComponent(Text, {
													selectable: true,
													accessibilityLabel: "studio.review.subject",
													get style() {
														return {
															color: p().text,
															fontSize: 12
														};
													},
													get children() {
														return s.selectedHistory().subject;
													}
												}),
												createComponent(For, {
													get each() {
														return s.selectedHistory().paragraphs;
													},
													children: (paragraph, index) => createComponent(Text, {
														selectable: true,
														get accessibilityLabel() {
															return `studio.review.text.${index()}`;
														},
														get style() {
															return {
																color: p().text,
																fontSize: 12,
																lineHeight: 18
															};
														},
														children: paragraph
													})
												}),
												createComponent(Action, {
													label: "studio.review.copy",
													text: "Copy review",
													onPress: () => command(props.copyText(s.selectedHistory().paragraphs.join("\n"))),
													get palette() {
														return p();
													}
												})
											];
										}
									});
								}
							})
						];
					}
				}),
				createComponent(View, {
					style: {
						height: 42,
						padding: 8,
						flexShrink: 0,
						overflow: "hidden"
					},
					get children() {
						return createComponent(Text, {
							accessibilityLabel: "studio.status",
							accessibilityRole: "status",
							get style() {
								return {
									color: p().muted,
									fontSize: 12
								};
							},
							get children() {
								return s.status();
							}
						});
					}
				})
			];
		}
	});
}
//#endregion
//#region examples/website/tests/reference-studio.application.tsx
/** Production controls and router, isolated from documentation virtualization for native qualification. */
function mountReferenceStudio(transport, surfaceId = 1) {
	const size = createWindowSizeStore({
		width: 1280,
		height: 720
	});
	const live = /* @__PURE__ */ new Set();
	let peak = 0;
	let root;
	const route = createRootRoute({ component: () => {
		const dimensions = useWindowSize(size);
		const location = useLocation();
		const router = useRouter();
		return createComponent(ReferenceStudio, {
			state: createReferenceStudioState(),
			get width() {
				return dimensions().width;
			},
			get height() {
				return dimensions().height;
			},
			get view() {
				return location().pathname === "/studio/history" ? "history" : "timeline";
			},
			navigate: (view) => void router.navigate({ to: `/studio/${view}` }),
			copyText: (text) => root.setClipboardText(text),
			onRowLifetime: (kind, id, mounted) => {
				const key = `${kind}:${id}`;
				if (mounted) live.add(key);
				else live.delete(key);
				peak = Math.max(peak, live.size);
			}
		});
	} });
	const timeline = createRoute({
		getParentRoute: () => route,
		path: "/studio/timeline",
		component: () => null
	});
	const history = createRoute({
		getParentRoute: () => route,
		path: "/studio/history",
		component: () => null
	});
	const router = createRouter({
		routeTree: route.addChildren([timeline, history]),
		initialEntries: ["/studio/timeline"]
	});
	root = createRoot(transport, {
		surfaceId,
		onWindowResize: (width, height, scale) => size.set(width, height, scale)
	});
	root.render(() => createComponent(RouterProvider, { router }));
	return {
		root,
		router,
		rowOwners: () => ({
			live: live.size,
			peak
		}),
		dispose: () => {
			root.unmount();
			router.history.destroy();
		}
	};
}
//#endregion
//#region examples/website/tests/reference-studio.entry.tsx
var transport = new StdioTransport();
var app = mountReferenceStudio(transport);
await app.root.setTitle("Reference Studio — native acceptance");
await app.root.resize(1280, 720);
var closed = false;
var close = () => {
	if (closed) return;
	closed = true;
	app.dispose();
	transport.dispose();
};
process.once("SIGTERM", close);
process.once("SIGINT", close);
process.once("beforeExit", close);
//#endregion
export {};
