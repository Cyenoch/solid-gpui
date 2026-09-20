// Generated from protocol.bop; do not edit.

#[derive(Clone, Copy, Debug)]
pub(crate) enum TypeSpec {
    Scalar(&'static str),
    Array { element: usize },
    Definition { definition: usize },
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Field {
    pub(crate) id: u8,
    pub(crate) name: &'static str,
    pub(crate) type_id: usize,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Branch {
    pub(crate) id: u8,
    pub(crate) definition: usize,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Definition {
    Message {
        name: &'static str,
        fields: &'static [Field],
    },
    Union {
        branches: &'static [Branch],
    },
    Enum {
        name: &'static str,
        base: &'static str,
        values: &'static [i64],
    },
}

const FIELDS_5: &[Field] = &[
    Field {
        id: 1,
        name: "surfaceId",
        type_id: 0,
    },
    Field {
        id: 2,
        name: "epoch",
        type_id: 1,
    },
    Field {
        id: 3,
        name: "baseRevision",
        type_id: 2,
    },
    Field {
        id: 4,
        name: "revision",
        type_id: 3,
    },
    Field {
        id: 5,
        name: "nodes",
        type_id: 5,
    },
];
const FIELDS_6: &[Field] = &[
    Field {
        id: 1,
        name: "surfaceId",
        type_id: 6,
    },
    Field {
        id: 2,
        name: "epoch",
        type_id: 7,
    },
    Field {
        id: 3,
        name: "revision",
        type_id: 8,
    },
    Field {
        id: 4,
        name: "sequence",
        type_id: 9,
    },
    Field {
        id: 5,
        name: "nodeId",
        type_id: 10,
    },
    Field {
        id: 6,
        name: "listenerId",
        type_id: 11,
    },
    Field {
        id: 7,
        name: "eventType",
        type_id: 12,
    },
    Field {
        id: 8,
        name: "payload",
        type_id: 13,
    },
];
const FIELDS_7: &[Field] = &[
    Field {
        id: 1,
        name: "surfaceId",
        type_id: 14,
    },
    Field {
        id: 2,
        name: "epoch",
        type_id: 15,
    },
    Field {
        id: 3,
        name: "baseRevision",
        type_id: 16,
    },
    Field {
        id: 4,
        name: "revision",
        type_id: 17,
    },
    Field {
        id: 5,
        name: "operations",
        type_id: 19,
    },
];
const FIELDS_8: &[Field] = &[
    Field {
        id: 1,
        name: "surfaceId",
        type_id: 20,
    },
    Field {
        id: 2,
        name: "epoch",
        type_id: 21,
    },
    Field {
        id: 3,
        name: "afterRevision",
        type_id: 22,
    },
    Field {
        id: 4,
        name: "requestId",
        type_id: 23,
    },
    Field {
        id: 5,
        name: "nodeId",
        type_id: 24,
    },
    Field {
        id: 6,
        name: "kind",
        type_id: 25,
    },
    Field {
        id: 7,
        name: "payload",
        type_id: 26,
    },
];
const FIELDS_10: &[Field] = &[Field {
    id: 1,
    name: "value",
    type_id: 27,
}];
const FIELDS_11: &[Field] = &[Field {
    id: 1,
    name: "value",
    type_id: 28,
}];
const FIELDS_12: &[Field] = &[Field {
    id: 1,
    name: "value",
    type_id: 29,
}];
const FIELDS_13: &[Field] = &[Field {
    id: 1,
    name: "value",
    type_id: 30,
}];
const FIELDS_14: &[Field] = &[Field {
    id: 1,
    name: "value",
    type_id: 31,
}];
const FIELDS_15: &[Field] = &[Field {
    id: 1,
    name: "value",
    type_id: 33,
}];
const FIELDS_16: &[Field] = &[
    Field {
        id: 1,
        name: "id",
        type_id: 34,
    },
    Field {
        id: 2,
        name: "value",
        type_id: 35,
    },
];
const FIELDS_18: &[Field] = &[
    Field {
        id: 1,
        name: "value",
        type_id: 36,
    },
    Field {
        id: 2,
        name: "placeholder",
        type_id: 37,
    },
    Field {
        id: 3,
        name: "multiline",
        type_id: 38,
    },
    Field {
        id: 4,
        name: "disabled",
        type_id: 39,
    },
    Field {
        id: 5,
        name: "controlled",
        type_id: 40,
    },
    Field {
        id: 6,
        name: "ackEditSeq",
        type_id: 41,
    },
    Field {
        id: 7,
        name: "selectionStart",
        type_id: 42,
    },
    Field {
        id: 8,
        name: "selectionEnd",
        type_id: 43,
    },
    Field {
        id: 9,
        name: "markedStart",
        type_id: 44,
    },
    Field {
        id: 10,
        name: "markedEnd",
        type_id: 45,
    },
    Field {
        id: 11,
        name: "maxLength",
        type_id: 46,
    },
    Field {
        id: 12,
        name: "selectionReversed",
        type_id: 47,
    },
];
const FIELDS_19: &[Field] = &[
    Field {
        id: 1,
        name: "itemCount",
        type_id: 48,
    },
    Field {
        id: 2,
        name: "rangeStart",
        type_id: 49,
    },
    Field {
        id: 3,
        name: "rangeEnd",
        type_id: 50,
    },
    Field {
        id: 4,
        name: "estimatedItemSize",
        type_id: 51,
    },
    Field {
        id: 5,
        name: "overscan",
        type_id: 52,
    },
    Field {
        id: 6,
        name: "dataRevision",
        type_id: 53,
    },
    Field {
        id: 7,
        name: "dataEdit",
        type_id: 54,
    },
];
const FIELDS_20: &[Field] = &[
    Field {
        id: 1,
        name: "source",
        type_id: 55,
    },
    Field {
        id: 2,
        name: "objectFit",
        type_id: 56,
    },
    Field {
        id: 3,
        name: "fallbackSource",
        type_id: 57,
    },
];
const FIELDS_21: &[Field] = &[
    Field {
        id: 1,
        name: "dragType",
        type_id: 58,
    },
    Field {
        id: 2,
        name: "exportFiles",
        type_id: 60,
    },
    Field {
        id: 3,
        name: "acceptsDragOver",
        type_id: 61,
    },
    Field {
        id: 4,
        name: "acceptsDrop",
        type_id: 62,
    },
];
const FIELDS_22: &[Field] = &[
    Field {
        id: 1,
        name: "providerId",
        type_id: 64,
    },
    Field {
        id: 2,
        name: "catalogDigest",
        type_id: 66,
    },
    Field {
        id: 3,
        name: "entryId",
        type_id: 67,
    },
    Field {
        id: 4,
        name: "entryVersion",
        type_id: 68,
    },
    Field {
        id: 5,
        name: "fields",
        type_id: 70,
    },
    Field {
        id: 6,
        name: "eventIds",
        type_id: 72,
    },
];
const FIELDS_23: &[Field] = &[
    Field {
        id: 1,
        name: "name",
        type_id: 73,
    },
    Field {
        id: 2,
        name: "size",
        type_id: 74,
    },
    Field {
        id: 3,
        name: "color",
        type_id: 75,
    },
];
const FIELDS_24: &[Field] = &[
    Field {
        id: 1,
        name: "baseRevision",
        type_id: 76,
    },
    Field {
        id: 2,
        name: "start",
        type_id: 77,
    },
    Field {
        id: 3,
        name: "oldCount",
        type_id: 78,
    },
    Field {
        id: 4,
        name: "newCount",
        type_id: 79,
    },
];
const FIELDS_25: &[Field] = &[
    Field {
        id: 1,
        name: "role",
        type_id: 80,
    },
    Field {
        id: 2,
        name: "label",
        type_id: 81,
    },
    Field {
        id: 3,
        name: "description",
        type_id: 82,
    },
    Field {
        id: 4,
        name: "disabled",
        type_id: 83,
    },
    Field {
        id: 5,
        name: "checked",
        type_id: 84,
    },
    Field {
        id: 6,
        name: "selected",
        type_id: 85,
    },
    Field {
        id: 7,
        name: "value",
        type_id: 86,
    },
    Field {
        id: 8,
        name: "expanded",
        type_id: 87,
    },
    Field {
        id: 9,
        name: "level",
        type_id: 88,
    },
    Field {
        id: 10,
        name: "live",
        type_id: 89,
    },
];
const FIELDS_26: &[Field] = &[
    Field {
        id: 1,
        name: "durationMs",
        type_id: 90,
    },
    Field {
        id: 2,
        name: "delayMs",
        type_id: 91,
    },
    Field {
        id: 3,
        name: "easing",
        type_id: 92,
    },
    Field {
        id: 4,
        name: "propertyMask",
        type_id: 93,
    },
];
const FIELDS_27: &[Field] = &[
    Field {
        id: 1,
        name: "offsetX",
        type_id: 94,
    },
    Field {
        id: 2,
        name: "offsetY",
        type_id: 95,
    },
    Field {
        id: 3,
        name: "blurRadius",
        type_id: 96,
    },
    Field {
        id: 4,
        name: "spreadRadius",
        type_id: 97,
    },
    Field {
        id: 5,
        name: "color",
        type_id: 98,
    },
    Field {
        id: 6,
        name: "inset",
        type_id: 99,
    },
];
const FIELDS_28: &[Field] = &[Field {
    id: 1,
    name: "values",
    type_id: 101,
}];
const FIELDS_29: &[Field] = &[
    Field {
        id: 1,
        name: "angle",
        type_id: 102,
    },
    Field {
        id: 2,
        name: "startColor",
        type_id: 103,
    },
    Field {
        id: 3,
        name: "startPosition",
        type_id: 104,
    },
    Field {
        id: 4,
        name: "endColor",
        type_id: 105,
    },
    Field {
        id: 5,
        name: "endPosition",
        type_id: 106,
    },
];
const FIELDS_30: &[Field] = &[
    Field {
        id: 1,
        name: "width",
        type_id: 107,
    },
    Field {
        id: 2,
        name: "height",
        type_id: 108,
    },
    Field {
        id: 3,
        name: "flexDirection",
        type_id: 109,
    },
    Field {
        id: 4,
        name: "flexGrow",
        type_id: 110,
    },
    Field {
        id: 5,
        name: "padding",
        type_id: 111,
    },
    Field {
        id: 6,
        name: "gap",
        type_id: 112,
    },
    Field {
        id: 7,
        name: "backgroundColor",
        type_id: 113,
    },
    Field {
        id: 8,
        name: "color",
        type_id: 114,
    },
    Field {
        id: 9,
        name: "opacity",
        type_id: 115,
    },
    Field {
        id: 10,
        name: "transition",
        type_id: 116,
    },
    Field {
        id: 11,
        name: "justifyContent",
        type_id: 117,
    },
    Field {
        id: 12,
        name: "alignItems",
        type_id: 118,
    },
    Field {
        id: 13,
        name: "borderRadius",
        type_id: 119,
    },
    Field {
        id: 14,
        name: "borderWidth",
        type_id: 120,
    },
    Field {
        id: 15,
        name: "borderColor",
        type_id: 121,
    },
    Field {
        id: 16,
        name: "fontSize",
        type_id: 122,
    },
    Field {
        id: 17,
        name: "fontWeight",
        type_id: 123,
    },
    Field {
        id: 18,
        name: "overflow",
        type_id: 124,
    },
    Field {
        id: 19,
        name: "lineClamp",
        type_id: 125,
    },
    Field {
        id: 20,
        name: "textOverflow",
        type_id: 126,
    },
    Field {
        id: 21,
        name: "marginTop",
        type_id: 127,
    },
    Field {
        id: 22,
        name: "marginRight",
        type_id: 128,
    },
    Field {
        id: 23,
        name: "marginBottom",
        type_id: 129,
    },
    Field {
        id: 24,
        name: "marginLeft",
        type_id: 130,
    },
    Field {
        id: 25,
        name: "fontStyle",
        type_id: 131,
    },
    Field {
        id: 26,
        name: "textDecoration",
        type_id: 132,
    },
    Field {
        id: 27,
        name: "lineHeight",
        type_id: 133,
    },
    Field {
        id: 28,
        name: "minWidth",
        type_id: 134,
    },
    Field {
        id: 29,
        name: "maxWidth",
        type_id: 135,
    },
    Field {
        id: 30,
        name: "minHeight",
        type_id: 136,
    },
    Field {
        id: 31,
        name: "maxHeight",
        type_id: 137,
    },
    Field {
        id: 32,
        name: "flexShrink",
        type_id: 138,
    },
    Field {
        id: 33,
        name: "alignSelf",
        type_id: 139,
    },
    Field {
        id: 34,
        name: "position",
        type_id: 140,
    },
    Field {
        id: 35,
        name: "left",
        type_id: 141,
    },
    Field {
        id: 36,
        name: "top",
        type_id: 142,
    },
    Field {
        id: 37,
        name: "right",
        type_id: 143,
    },
    Field {
        id: 38,
        name: "bottom",
        type_id: 144,
    },
    Field {
        id: 39,
        name: "cursor",
        type_id: 145,
    },
    Field {
        id: 40,
        name: "textAlign",
        type_id: 146,
    },
    Field {
        id: 41,
        name: "boxShadow",
        type_id: 147,
    },
    Field {
        id: 42,
        name: "fontFamily",
        type_id: 148,
    },
    Field {
        id: 43,
        name: "paddingTop",
        type_id: 149,
    },
    Field {
        id: 44,
        name: "paddingRight",
        type_id: 150,
    },
    Field {
        id: 45,
        name: "paddingBottom",
        type_id: 151,
    },
    Field {
        id: 46,
        name: "paddingLeft",
        type_id: 152,
    },
    Field {
        id: 47,
        name: "borderTopWidth",
        type_id: 153,
    },
    Field {
        id: 48,
        name: "borderRightWidth",
        type_id: 154,
    },
    Field {
        id: 49,
        name: "borderBottomWidth",
        type_id: 155,
    },
    Field {
        id: 50,
        name: "borderLeftWidth",
        type_id: 156,
    },
    Field {
        id: 51,
        name: "borderTopLeftRadius",
        type_id: 157,
    },
    Field {
        id: 52,
        name: "borderTopRightRadius",
        type_id: 158,
    },
    Field {
        id: 53,
        name: "borderBottomRightRadius",
        type_id: 159,
    },
    Field {
        id: 54,
        name: "borderBottomLeftRadius",
        type_id: 160,
    },
    Field {
        id: 55,
        name: "widthPercent",
        type_id: 161,
    },
    Field {
        id: 56,
        name: "heightPercent",
        type_id: 162,
    },
    Field {
        id: 57,
        name: "flexWrap",
        type_id: 163,
    },
    Field {
        id: 58,
        name: "linearGradient",
        type_id: 164,
    },
    Field {
        id: 59,
        name: "borderTopColor",
        type_id: 165,
    },
    Field {
        id: 60,
        name: "borderRightColor",
        type_id: 166,
    },
    Field {
        id: 61,
        name: "borderBottomColor",
        type_id: 167,
    },
    Field {
        id: 62,
        name: "borderLeftColor",
        type_id: 168,
    },
    Field {
        id: 63,
        name: "gridColumns",
        type_id: 169,
    },
    Field {
        id: 64,
        name: "gridRows",
        type_id: 170,
    },
    Field {
        id: 65,
        name: "gridColumnSpan",
        type_id: 171,
    },
    Field {
        id: 66,
        name: "gridRowSpan",
        type_id: 172,
    },
];
const FIELDS_31: &[Field] = &[
    Field {
        id: 1,
        name: "id",
        type_id: 173,
    },
    Field {
        id: 2,
        name: "parentId",
        type_id: 174,
    },
    Field {
        id: 3,
        name: "index",
        type_id: 175,
    },
    Field {
        id: 4,
        name: "kind",
        type_id: 176,
    },
    Field {
        id: 5,
        name: "style",
        type_id: 177,
    },
    Field {
        id: 6,
        name: "text",
        type_id: 178,
    },
    Field {
        id: 7,
        name: "listenerId",
        type_id: 179,
    },
    Field {
        id: 8,
        name: "hostProperties",
        type_id: 180,
    },
    Field {
        id: 9,
        name: "accessibility",
        type_id: 181,
    },
    Field {
        id: 10,
        name: "focusable",
        type_id: 182,
    },
    Field {
        id: 11,
        name: "selectable",
        type_id: 183,
    },
    Field {
        id: 12,
        name: "tooltip",
        type_id: 184,
    },
    Field {
        id: 13,
        name: "acceptsPointerMove",
        type_id: 185,
    },
    Field {
        id: 14,
        name: "observesLayout",
        type_id: 186,
    },
];
const FIELDS_32: &[Field] = &[];
const FIELDS_34: &[Field] = &[Field {
    id: 1,
    name: "node",
    type_id: 187,
}];
const FIELDS_35: &[Field] = &[
    Field {
        id: 1,
        name: "id",
        type_id: 188,
    },
    Field {
        id: 2,
        name: "mask",
        type_id: 189,
    },
    Field {
        id: 3,
        name: "style",
        type_id: 190,
    },
    Field {
        id: 4,
        name: "clearStyle",
        type_id: 191,
    },
    Field {
        id: 5,
        name: "text",
        type_id: 192,
    },
    Field {
        id: 6,
        name: "listenerId",
        type_id: 193,
    },
    Field {
        id: 7,
        name: "hostProperties",
        type_id: 194,
    },
    Field {
        id: 8,
        name: "accessibility",
        type_id: 195,
    },
    Field {
        id: 9,
        name: "focusable",
        type_id: 196,
    },
    Field {
        id: 10,
        name: "selectable",
        type_id: 197,
    },
    Field {
        id: 11,
        name: "tooltip",
        type_id: 198,
    },
    Field {
        id: 12,
        name: "acceptsPointerMove",
        type_id: 199,
    },
    Field {
        id: 13,
        name: "observesLayout",
        type_id: 200,
    },
];
const FIELDS_36: &[Field] = &[
    Field {
        id: 1,
        name: "id",
        type_id: 201,
    },
    Field {
        id: 2,
        name: "parentId",
        type_id: 202,
    },
    Field {
        id: 3,
        name: "index",
        type_id: 203,
    },
];
const FIELDS_37: &[Field] = &[Field {
    id: 1,
    name: "id",
    type_id: 204,
}];
const FIELDS_38: &[Field] = &[Field {
    id: 1,
    name: "operation",
    type_id: 205,
}];
const FIELDS_39: &[Field] = &[
    Field {
        id: 1,
        name: "kind",
        type_id: 206,
    },
    Field {
        id: 2,
        name: "resizable",
        type_id: 207,
    },
    Field {
        id: 3,
        name: "minWidth",
        type_id: 208,
    },
    Field {
        id: 4,
        name: "minHeight",
        type_id: 209,
    },
];
const FIELDS_40: &[Field] = &[
    Field {
        id: 1,
        name: "id",
        type_id: 210,
    },
    Field {
        id: 2,
        name: "label",
        type_id: 211,
    },
];
const FIELDS_41: &[Field] = &[
    Field {
        id: 1,
        name: "title",
        type_id: 212,
    },
    Field {
        id: 2,
        name: "items",
        type_id: 214,
    },
];
const FIELDS_43: &[Field] = &[];
const FIELDS_44: &[Field] = &[
    Field {
        id: 1,
        name: "name",
        type_id: 215,
    },
    Field {
        id: 2,
        name: "disabled",
        type_id: 216,
    },
    Field {
        id: 3,
        name: "checked",
        type_id: 217,
    },
];
const FIELDS_45: &[Field] = &[Field {
    id: 1,
    name: "menu",
    type_id: 218,
}];
const FIELDS_46: &[Field] = &[Field {
    id: 1,
    name: "value",
    type_id: 219,
}];
const FIELDS_47: &[Field] = &[
    Field {
        id: 1,
        name: "keystrokes",
        type_id: 220,
    },
    Field {
        id: 2,
        name: "actionName",
        type_id: 221,
    },
];
const FIELDS_49: &[Field] = &[
    Field {
        id: 1,
        name: "first",
        type_id: 222,
    },
    Field {
        id: 2,
        name: "second",
        type_id: 223,
    },
];
const FIELDS_50: &[Field] = &[Field {
    id: 1,
    name: "value",
    type_id: 224,
}];
const FIELDS_51: &[Field] = &[Field {
    id: 1,
    name: "value",
    type_id: 225,
}];
const FIELDS_52: &[Field] = &[
    Field {
        id: 1,
        name: "path",
        type_id: 226,
    },
    Field {
        id: 2,
        name: "content",
        type_id: 227,
    },
];
const FIELDS_53: &[Field] = &[
    Field {
        id: 1,
        name: "title",
        type_id: 228,
    },
    Field {
        id: 2,
        name: "width",
        type_id: 229,
    },
    Field {
        id: 3,
        name: "height",
        type_id: 230,
    },
    Field {
        id: 4,
        name: "options",
        type_id: 231,
    },
];
const FIELDS_54: &[Field] = &[
    Field {
        id: 1,
        name: "title",
        type_id: 232,
    },
    Field {
        id: 2,
        name: "directories",
        type_id: 233,
    },
    Field {
        id: 3,
        name: "multiple",
        type_id: 234,
    },
];
const FIELDS_55: &[Field] = &[
    Field {
        id: 1,
        name: "title",
        type_id: 235,
    },
    Field {
        id: 2,
        name: "body",
        type_id: 236,
    },
    Field {
        id: 3,
        name: "actions",
        type_id: 238,
    },
];
const FIELDS_56: &[Field] = &[Field {
    id: 1,
    name: "menus",
    type_id: 240,
}];
const FIELDS_57: &[Field] = &[Field {
    id: 1,
    name: "bindings",
    type_id: 242,
}];
const FIELDS_58: &[Field] = &[
    Field {
        id: 1,
        name: "format",
        type_id: 243,
    },
    Field {
        id: 2,
        name: "bytes",
        type_id: 245,
    },
];
const FIELDS_59: &[Field] = &[
    Field {
        id: 1,
        name: "requestId",
        type_id: 246,
    },
    Field {
        id: 2,
        name: "allow",
        type_id: 247,
    },
];
const FIELDS_60: &[Field] = &[
    Field {
        id: 1,
        name: "moduleId",
        type_id: 249,
    },
    Field {
        id: 2,
        name: "moduleDigest",
        type_id: 251,
    },
    Field {
        id: 3,
        name: "functionId",
        type_id: 252,
    },
    Field {
        id: 4,
        name: "args",
        type_id: 254,
    },
];
const FIELDS_61: &[Field] = &[Field {
    id: 1,
    name: "requestId",
    type_id: 255,
}];
const FIELDS_62: &[Field] = &[
    Field {
        id: 1,
        name: "keepAlive",
        type_id: 256,
    },
    Field {
        id: 2,
        name: "quit",
        type_id: 257,
    },
    Field {
        id: 3,
        name: "acknowledgedSequence",
        type_id: 258,
    },
];
const FIELDS_63: &[Field] = &[
    Field {
        id: 1,
        name: "anchorNodeId",
        type_id: 259,
    },
    Field {
        id: 2,
        name: "width",
        type_id: 260,
    },
    Field {
        id: 3,
        name: "height",
        type_id: 261,
    },
    Field {
        id: 4,
        name: "placement",
        type_id: 262,
    },
    Field {
        id: 5,
        name: "gap",
        type_id: 263,
    },
];
const FIELDS_64: &[Field] = &[Field {
    id: 1,
    name: "requestId",
    type_id: 264,
}];
const FIELDS_66: &[Field] = &[Field {
    id: 1,
    name: "value",
    type_id: 265,
}];
const FIELDS_67: &[Field] = &[
    Field {
        id: 1,
        name: "width",
        type_id: 266,
    },
    Field {
        id: 2,
        name: "height",
        type_id: 267,
    },
];
const FIELDS_68: &[Field] = &[Field {
    id: 1,
    name: "value",
    type_id: 268,
}];
const FIELDS_69: &[Field] = &[Field {
    id: 1,
    name: "value",
    type_id: 269,
}];
const FIELDS_70: &[Field] = &[Field {
    id: 1,
    name: "paths",
    type_id: 271,
}];
const FIELDS_71: &[Field] = &[Field {
    id: 1,
    name: "value",
    type_id: 272,
}];
const FIELDS_72: &[Field] = &[
    Field {
        id: 1,
        name: "format",
        type_id: 273,
    },
    Field {
        id: 2,
        name: "bytes",
        type_id: 275,
    },
];
const FIELDS_73: &[Field] = &[
    Field {
        id: 1,
        name: "x",
        type_id: 276,
    },
    Field {
        id: 2,
        name: "y",
        type_id: 277,
    },
    Field {
        id: 3,
        name: "width",
        type_id: 278,
    },
    Field {
        id: 4,
        name: "height",
        type_id: 279,
    },
];
const FIELDS_74: &[Field] = &[
    Field {
        id: 1,
        name: "fullscreen",
        type_id: 280,
    },
    Field {
        id: 2,
        name: "maximized",
        type_id: 281,
    },
];
const FIELDS_75: &[Field] = &[Field {
    id: 1,
    name: "value",
    type_id: 282,
}];
const FIELDS_76: &[Field] = &[Field {
    id: 1,
    name: "value",
    type_id: 284,
}];
const FIELDS_78: &[Field] = &[
    Field {
        id: 1,
        name: "text",
        type_id: 285,
    },
    Field {
        id: 2,
        name: "selectionStart",
        type_id: 286,
    },
    Field {
        id: 3,
        name: "selectionEnd",
        type_id: 287,
    },
    Field {
        id: 4,
        name: "markedStart",
        type_id: 288,
    },
    Field {
        id: 5,
        name: "markedEnd",
        type_id: 289,
    },
    Field {
        id: 6,
        name: "editSeq",
        type_id: 290,
    },
    Field {
        id: 7,
        name: "reversed",
        type_id: 291,
    },
];
const FIELDS_79: &[Field] = &[
    Field {
        id: 1,
        name: "requestId",
        type_id: 292,
    },
    Field {
        id: 2,
        name: "command",
        type_id: 293,
    },
    Field {
        id: 3,
        name: "nodeId",
        type_id: 294,
    },
    Field {
        id: 4,
        name: "success",
        type_id: 295,
    },
    Field {
        id: 5,
        name: "error",
        type_id: 296,
    },
    Field {
        id: 6,
        name: "value",
        type_id: 297,
    },
];
const FIELDS_80: &[Field] = &[
    Field {
        id: 1,
        name: "start",
        type_id: 298,
    },
    Field {
        id: 2,
        name: "end",
        type_id: 299,
    },
];
const FIELDS_81: &[Field] = &[Field {
    id: 1,
    name: "generation",
    type_id: 300,
}];
const FIELDS_82: &[Field] = &[
    Field {
        id: 1,
        name: "key",
        type_id: 301,
    },
    Field {
        id: 2,
        name: "modifiers",
        type_id: 303,
    },
    Field {
        id: 3,
        name: "action",
        type_id: 304,
    },
];
const FIELDS_83: &[Field] = &[
    Field {
        id: 1,
        name: "button",
        type_id: 305,
    },
    Field {
        id: 2,
        name: "modifiers",
        type_id: 307,
    },
    Field {
        id: 3,
        name: "action",
        type_id: 308,
    },
    Field {
        id: 4,
        name: "clickCount",
        type_id: 309,
    },
    Field {
        id: 5,
        name: "x",
        type_id: 310,
    },
    Field {
        id: 6,
        name: "y",
        type_id: 311,
    },
];
const FIELDS_84: &[Field] = &[
    Field {
        id: 1,
        name: "modifiers",
        type_id: 313,
    },
    Field {
        id: 2,
        name: "x",
        type_id: 314,
    },
    Field {
        id: 3,
        name: "y",
        type_id: 315,
    },
];
const FIELDS_85: &[Field] = &[
    Field {
        id: 1,
        name: "deltaKind",
        type_id: 316,
    },
    Field {
        id: 2,
        name: "dx",
        type_id: 317,
    },
    Field {
        id: 3,
        name: "dy",
        type_id: 318,
    },
    Field {
        id: 4,
        name: "x",
        type_id: 319,
    },
    Field {
        id: 5,
        name: "y",
        type_id: 320,
    },
    Field {
        id: 6,
        name: "modifiers",
        type_id: 322,
    },
];
const FIELDS_86: &[Field] = &[Field {
    id: 1,
    name: "text",
    type_id: 323,
}];
const FIELDS_87: &[Field] = &[
    Field {
        id: 1,
        name: "width",
        type_id: 324,
    },
    Field {
        id: 2,
        name: "height",
        type_id: 325,
    },
    Field {
        id: 3,
        name: "scaleFactor",
        type_id: 326,
    },
];
const FIELDS_88: &[Field] = &[Field {
    id: 1,
    name: "active",
    type_id: 327,
}];
const FIELDS_89: &[Field] = &[Field {
    id: 1,
    name: "action",
    type_id: 328,
}];
const FIELDS_90: &[Field] = &[Field {
    id: 1,
    name: "appearance",
    type_id: 329,
}];
const FIELDS_91: &[Field] = &[
    Field {
        id: 1,
        name: "x",
        type_id: 330,
    },
    Field {
        id: 2,
        name: "y",
        type_id: 331,
    },
    Field {
        id: 3,
        name: "width",
        type_id: 332,
    },
    Field {
        id: 4,
        name: "height",
        type_id: 333,
    },
];
const FIELDS_92: &[Field] = &[Field {
    id: 1,
    name: "dragType",
    type_id: 334,
}];
const FIELDS_93: &[Field] = &[Field {
    id: 1,
    name: "dragType",
    type_id: 335,
}];
const FIELDS_94: &[Field] = &[Field {
    id: 1,
    name: "paths",
    type_id: 337,
}];
const FIELDS_95: &[Field] = &[
    Field {
        id: 1,
        name: "tag",
        type_id: 338,
    },
    Field {
        id: 2,
        name: "actionId",
        type_id: 339,
    },
];
const FIELDS_96: &[Field] = &[
    Field {
        id: 1,
        name: "x",
        type_id: 340,
    },
    Field {
        id: 2,
        name: "y",
        type_id: 341,
    },
];
const FIELDS_97: &[Field] = &[Field {
    id: 1,
    name: "requestId",
    type_id: 342,
}];
const FIELDS_98: &[Field] = &[
    Field {
        id: 1,
        name: "eventId",
        type_id: 343,
    },
    Field {
        id: 2,
        name: "fields",
        type_id: 345,
    },
];
const FIELDS_99: &[Field] = &[
    Field {
        id: 1,
        name: "targetSurfaceId",
        type_id: 346,
    },
    Field {
        id: 2,
        name: "reason",
        type_id: 347,
    },
    Field {
        id: 3,
        name: "urls",
        type_id: 349,
    },
];
const FIELDS_100: &[Field] = &[
    Field {
        id: 1,
        name: "protocolVersion",
        type_id: 350,
    },
    Field {
        id: 2,
        name: "body",
        type_id: 351,
    },
];

const BRANCHES_4: &[Branch] = &[
    Branch {
        id: 1,
        definition: 5,
    },
    Branch {
        id: 2,
        definition: 6,
    },
    Branch {
        id: 3,
        definition: 7,
    },
    Branch {
        id: 4,
        definition: 8,
    },
];
const BRANCHES_9: &[Branch] = &[
    Branch {
        id: 1,
        definition: 10,
    },
    Branch {
        id: 2,
        definition: 11,
    },
    Branch {
        id: 3,
        definition: 12,
    },
    Branch {
        id: 4,
        definition: 13,
    },
    Branch {
        id: 5,
        definition: 14,
    },
    Branch {
        id: 6,
        definition: 15,
    },
];
const BRANCHES_17: &[Branch] = &[
    Branch {
        id: 1,
        definition: 18,
    },
    Branch {
        id: 2,
        definition: 19,
    },
    Branch {
        id: 3,
        definition: 20,
    },
    Branch {
        id: 4,
        definition: 21,
    },
    Branch {
        id: 5,
        definition: 22,
    },
    Branch {
        id: 6,
        definition: 23,
    },
];
const BRANCHES_33: &[Branch] = &[
    Branch {
        id: 1,
        definition: 34,
    },
    Branch {
        id: 2,
        definition: 35,
    },
    Branch {
        id: 3,
        definition: 36,
    },
    Branch {
        id: 4,
        definition: 37,
    },
];
const BRANCHES_42: &[Branch] = &[
    Branch {
        id: 1,
        definition: 43,
    },
    Branch {
        id: 2,
        definition: 44,
    },
    Branch {
        id: 3,
        definition: 45,
    },
];
const BRANCHES_48: &[Branch] = &[
    Branch {
        id: 1,
        definition: 49,
    },
    Branch {
        id: 2,
        definition: 50,
    },
    Branch {
        id: 3,
        definition: 51,
    },
    Branch {
        id: 4,
        definition: 52,
    },
    Branch {
        id: 5,
        definition: 53,
    },
    Branch {
        id: 6,
        definition: 54,
    },
    Branch {
        id: 7,
        definition: 55,
    },
    Branch {
        id: 8,
        definition: 56,
    },
    Branch {
        id: 9,
        definition: 57,
    },
    Branch {
        id: 10,
        definition: 58,
    },
    Branch {
        id: 11,
        definition: 59,
    },
    Branch {
        id: 12,
        definition: 60,
    },
    Branch {
        id: 13,
        definition: 61,
    },
    Branch {
        id: 14,
        definition: 62,
    },
    Branch {
        id: 15,
        definition: 63,
    },
    Branch {
        id: 16,
        definition: 64,
    },
];
const BRANCHES_65: &[Branch] = &[
    Branch {
        id: 1,
        definition: 66,
    },
    Branch {
        id: 2,
        definition: 67,
    },
    Branch {
        id: 3,
        definition: 68,
    },
    Branch {
        id: 4,
        definition: 69,
    },
    Branch {
        id: 5,
        definition: 70,
    },
    Branch {
        id: 6,
        definition: 71,
    },
    Branch {
        id: 7,
        definition: 72,
    },
    Branch {
        id: 8,
        definition: 73,
    },
    Branch {
        id: 9,
        definition: 74,
    },
    Branch {
        id: 10,
        definition: 75,
    },
    Branch {
        id: 11,
        definition: 76,
    },
];
const BRANCHES_77: &[Branch] = &[
    Branch {
        id: 1,
        definition: 78,
    },
    Branch {
        id: 2,
        definition: 79,
    },
    Branch {
        id: 3,
        definition: 80,
    },
    Branch {
        id: 4,
        definition: 81,
    },
    Branch {
        id: 5,
        definition: 82,
    },
    Branch {
        id: 6,
        definition: 83,
    },
    Branch {
        id: 7,
        definition: 84,
    },
    Branch {
        id: 8,
        definition: 85,
    },
    Branch {
        id: 9,
        definition: 86,
    },
    Branch {
        id: 10,
        definition: 87,
    },
    Branch {
        id: 11,
        definition: 88,
    },
    Branch {
        id: 12,
        definition: 89,
    },
    Branch {
        id: 13,
        definition: 90,
    },
    Branch {
        id: 14,
        definition: 91,
    },
    Branch {
        id: 15,
        definition: 92,
    },
    Branch {
        id: 16,
        definition: 93,
    },
    Branch {
        id: 17,
        definition: 94,
    },
    Branch {
        id: 18,
        definition: 95,
    },
    Branch {
        id: 19,
        definition: 96,
    },
    Branch {
        id: 20,
        definition: 97,
    },
    Branch {
        id: 21,
        definition: 98,
    },
    Branch {
        id: 22,
        definition: 99,
    },
];

pub(crate) static TYPES: &[TypeSpec] = &[
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Definition { definition: 31 },
    TypeSpec::Array { element: 4 },
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Definition { definition: 2 },
    TypeSpec::Definition { definition: 77 },
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Definition { definition: 38 },
    TypeSpec::Array { element: 18 },
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Definition { definition: 3 },
    TypeSpec::Definition { definition: 48 },
    TypeSpec::Scalar("bool"),
    TypeSpec::Scalar("int32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("byte"),
    TypeSpec::Array { element: 32 },
    TypeSpec::Scalar("uint32"),
    TypeSpec::Definition { definition: 9 },
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("bool"),
    TypeSpec::Scalar("bool"),
    TypeSpec::Scalar("bool"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("bool"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Definition { definition: 24 },
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("string"),
    TypeSpec::Array { element: 59 },
    TypeSpec::Scalar("bool"),
    TypeSpec::Scalar("bool"),
    TypeSpec::Scalar("byte"),
    TypeSpec::Array { element: 63 },
    TypeSpec::Scalar("byte"),
    TypeSpec::Array { element: 65 },
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Definition { definition: 16 },
    TypeSpec::Array { element: 69 },
    TypeSpec::Scalar("uint32"),
    TypeSpec::Array { element: 71 },
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("bool"),
    TypeSpec::Scalar("bool"),
    TypeSpec::Scalar("bool"),
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("bool"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("bool"),
    TypeSpec::Definition { definition: 27 },
    TypeSpec::Array { element: 100 },
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Definition { definition: 26 },
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Definition { definition: 28 },
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Definition { definition: 29 },
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Definition { definition: 0 },
    TypeSpec::Definition { definition: 30 },
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Definition { definition: 17 },
    TypeSpec::Definition { definition: 25 },
    TypeSpec::Scalar("bool"),
    TypeSpec::Scalar("bool"),
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("bool"),
    TypeSpec::Scalar("bool"),
    TypeSpec::Definition { definition: 31 },
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Definition { definition: 30 },
    TypeSpec::Definition { definition: 32 },
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Definition { definition: 17 },
    TypeSpec::Definition { definition: 25 },
    TypeSpec::Scalar("bool"),
    TypeSpec::Scalar("bool"),
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("bool"),
    TypeSpec::Scalar("bool"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Definition { definition: 33 },
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("bool"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("string"),
    TypeSpec::Definition { definition: 46 },
    TypeSpec::Array { element: 213 },
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("bool"),
    TypeSpec::Scalar("bool"),
    TypeSpec::Definition { definition: 41 },
    TypeSpec::Definition { definition: 42 },
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Definition { definition: 39 },
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("bool"),
    TypeSpec::Scalar("bool"),
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("string"),
    TypeSpec::Definition { definition: 40 },
    TypeSpec::Array { element: 237 },
    TypeSpec::Definition { definition: 41 },
    TypeSpec::Array { element: 239 },
    TypeSpec::Definition { definition: 47 },
    TypeSpec::Array { element: 241 },
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("byte"),
    TypeSpec::Array { element: 244 },
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("bool"),
    TypeSpec::Scalar("byte"),
    TypeSpec::Array { element: 248 },
    TypeSpec::Scalar("byte"),
    TypeSpec::Array { element: 250 },
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("byte"),
    TypeSpec::Array { element: 253 },
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("bool"),
    TypeSpec::Scalar("bool"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("bool"),
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("string"),
    TypeSpec::Array { element: 270 },
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("byte"),
    TypeSpec::Array { element: 274 },
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("bool"),
    TypeSpec::Scalar("bool"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("byte"),
    TypeSpec::Array { element: 283 },
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("bool"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("bool"),
    TypeSpec::Scalar("string"),
    TypeSpec::Definition { definition: 65 },
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("string"),
    TypeSpec::Array { element: 302 },
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("string"),
    TypeSpec::Array { element: 306 },
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("string"),
    TypeSpec::Array { element: 312 },
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("string"),
    TypeSpec::Array { element: 321 },
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("bool"),
    TypeSpec::Scalar("string"),
    TypeSpec::Definition { definition: 1 },
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("string"),
    TypeSpec::Array { element: 336 },
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("float32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("uint32"),
    TypeSpec::Definition { definition: 16 },
    TypeSpec::Array { element: 344 },
    TypeSpec::Scalar("uint32"),
    TypeSpec::Scalar("string"),
    TypeSpec::Scalar("string"),
    TypeSpec::Array { element: 348 },
    TypeSpec::Scalar("uint32"),
    TypeSpec::Definition { definition: 4 },
];

pub(crate) static DEFINITIONS: &[Definition] = &[
    Definition::Enum {
        name: "NodeKind",
        base: "uint8",
        values: &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
    },
    Definition::Enum {
        name: "WindowAppearance",
        base: "uint8",
        values: &[0, 1, 2],
    },
    Definition::Enum {
        name: "EventKind",
        base: "uint8",
        values: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25,
        ],
    },
    Definition::Enum {
        name: "CommandKind",
        base: "uint8",
        values: &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
        ],
    },
    Definition::Union {
        branches: BRANCHES_4,
    },
    Definition::Message {
        name: "Snapshot",
        fields: FIELDS_5,
    },
    Definition::Message {
        name: "Event",
        fields: FIELDS_6,
    },
    Definition::Message {
        name: "Patch",
        fields: FIELDS_7,
    },
    Definition::Message {
        name: "Command",
        fields: FIELDS_8,
    },
    Definition::Union {
        branches: BRANCHES_9,
    },
    Definition::Message {
        name: "ExtensionBoolValue",
        fields: FIELDS_10,
    },
    Definition::Message {
        name: "ExtensionInt32Value",
        fields: FIELDS_11,
    },
    Definition::Message {
        name: "ExtensionU32Value",
        fields: FIELDS_12,
    },
    Definition::Message {
        name: "ExtensionF32Value",
        fields: FIELDS_13,
    },
    Definition::Message {
        name: "ExtensionTextValue",
        fields: FIELDS_14,
    },
    Definition::Message {
        name: "ExtensionBytesValue",
        fields: FIELDS_15,
    },
    Definition::Message {
        name: "ExtensionField",
        fields: FIELDS_16,
    },
    Definition::Union {
        branches: BRANCHES_17,
    },
    Definition::Message {
        name: "TextInputProperties",
        fields: FIELDS_18,
    },
    Definition::Message {
        name: "VirtualListProperties",
        fields: FIELDS_19,
    },
    Definition::Message {
        name: "ImageProperties",
        fields: FIELDS_20,
    },
    Definition::Message {
        name: "DragProperties",
        fields: FIELDS_21,
    },
    Definition::Message {
        name: "ExtensionProperties",
        fields: FIELDS_22,
    },
    Definition::Message {
        name: "IconProperties",
        fields: FIELDS_23,
    },
    Definition::Message {
        name: "VirtualListDataEdit",
        fields: FIELDS_24,
    },
    Definition::Message {
        name: "AccessibilityProperties",
        fields: FIELDS_25,
    },
    Definition::Message {
        name: "Transition",
        fields: FIELDS_26,
    },
    Definition::Message {
        name: "BoxShadowValue",
        fields: FIELDS_27,
    },
    Definition::Message {
        name: "BoxShadowSet",
        fields: FIELDS_28,
    },
    Definition::Message {
        name: "LinearGradient",
        fields: FIELDS_29,
    },
    Definition::Message {
        name: "Style",
        fields: FIELDS_30,
    },
    Definition::Message {
        name: "Node",
        fields: FIELDS_31,
    },
    Definition::Message {
        name: "ClearStyle",
        fields: FIELDS_32,
    },
    Definition::Union {
        branches: BRANCHES_33,
    },
    Definition::Message {
        name: "PatchCreate",
        fields: FIELDS_34,
    },
    Definition::Message {
        name: "PatchUpdate",
        fields: FIELDS_35,
    },
    Definition::Message {
        name: "PatchMove",
        fields: FIELDS_36,
    },
    Definition::Message {
        name: "PatchDelete",
        fields: FIELDS_37,
    },
    Definition::Message {
        name: "PatchOperation",
        fields: FIELDS_38,
    },
    Definition::Message {
        name: "WindowOpenOptions",
        fields: FIELDS_39,
    },
    Definition::Message {
        name: "NotificationActionDefinition",
        fields: FIELDS_40,
    },
    Definition::Message {
        name: "MenuDefinition",
        fields: FIELDS_41,
    },
    Definition::Union {
        branches: BRANCHES_42,
    },
    Definition::Message {
        name: "MenuSeparator",
        fields: FIELDS_43,
    },
    Definition::Message {
        name: "MenuAction",
        fields: FIELDS_44,
    },
    Definition::Message {
        name: "MenuSubmenu",
        fields: FIELDS_45,
    },
    Definition::Message {
        name: "MenuItem",
        fields: FIELDS_46,
    },
    Definition::Message {
        name: "KeybindingDefinition",
        fields: FIELDS_47,
    },
    Definition::Union {
        branches: BRANCHES_48,
    },
    Definition::Message {
        name: "U32PairCommand",
        fields: FIELDS_49,
    },
    Definition::Message {
        name: "FloatCommand",
        fields: FIELDS_50,
    },
    Definition::Message {
        name: "TextCommand",
        fields: FIELDS_51,
    },
    Definition::Message {
        name: "StringPairCommand",
        fields: FIELDS_52,
    },
    Definition::Message {
        name: "OpenSurfaceCommand",
        fields: FIELDS_53,
    },
    Definition::Message {
        name: "FileDialogOpenCommand",
        fields: FIELDS_54,
    },
    Definition::Message {
        name: "NotificationCommand",
        fields: FIELDS_55,
    },
    Definition::Message {
        name: "MenusCommand",
        fields: FIELDS_56,
    },
    Definition::Message {
        name: "KeybindingsCommand",
        fields: FIELDS_57,
    },
    Definition::Message {
        name: "ClipboardImageCommand",
        fields: FIELDS_58,
    },
    Definition::Message {
        name: "CloseResolutionCommand",
        fields: FIELDS_59,
    },
    Definition::Message {
        name: "InvokeNativeCommand",
        fields: FIELDS_60,
    },
    Definition::Message {
        name: "CancelNativeCommand",
        fields: FIELDS_61,
    },
    Definition::Message {
        name: "ConfigureApplicationCommand",
        fields: FIELDS_62,
    },
    Definition::Message {
        name: "OpenPopupCommand",
        fields: FIELDS_63,
    },
    Definition::Message {
        name: "ClosePopupCommand",
        fields: FIELDS_64,
    },
    Definition::Union {
        branches: BRANCHES_65,
    },
    Definition::Message {
        name: "NumberValue",
        fields: FIELDS_66,
    },
    Definition::Message {
        name: "PairValue",
        fields: FIELDS_67,
    },
    Definition::Message {
        name: "BoolValue",
        fields: FIELDS_68,
    },
    Definition::Message {
        name: "TextValue",
        fields: FIELDS_69,
    },
    Definition::Message {
        name: "PathsValue",
        fields: FIELDS_70,
    },
    Definition::Message {
        name: "FileTextValue",
        fields: FIELDS_71,
    },
    Definition::Message {
        name: "ImageValue",
        fields: FIELDS_72,
    },
    Definition::Message {
        name: "BoundsValue",
        fields: FIELDS_73,
    },
    Definition::Message {
        name: "WindowStateValue",
        fields: FIELDS_74,
    },
    Definition::Message {
        name: "ScrollOffsetValue",
        fields: FIELDS_75,
    },
    Definition::Message {
        name: "BytesValue",
        fields: FIELDS_76,
    },
    Definition::Union {
        branches: BRANCHES_77,
    },
    Definition::Message {
        name: "TextInputEventData",
        fields: FIELDS_78,
    },
    Definition::Message {
        name: "CommandResult",
        fields: FIELDS_79,
    },
    Definition::Message {
        name: "VisibleRangeEvent",
        fields: FIELDS_80,
    },
    Definition::Message {
        name: "AnimationCompleteEvent",
        fields: FIELDS_81,
    },
    Definition::Message {
        name: "KeyEvent",
        fields: FIELDS_82,
    },
    Definition::Message {
        name: "PointerEvent",
        fields: FIELDS_83,
    },
    Definition::Message {
        name: "PointerMoveEvent",
        fields: FIELDS_84,
    },
    Definition::Message {
        name: "ScrollEvent",
        fields: FIELDS_85,
    },
    Definition::Message {
        name: "SubmitEvent",
        fields: FIELDS_86,
    },
    Definition::Message {
        name: "WindowResizeEvent",
        fields: FIELDS_87,
    },
    Definition::Message {
        name: "WindowActivationEvent",
        fields: FIELDS_88,
    },
    Definition::Message {
        name: "ActionEvent",
        fields: FIELDS_89,
    },
    Definition::Message {
        name: "WindowAppearanceEvent",
        fields: FIELDS_90,
    },
    Definition::Message {
        name: "LayoutEvent",
        fields: FIELDS_91,
    },
    Definition::Message {
        name: "DragOverEvent",
        fields: FIELDS_92,
    },
    Definition::Message {
        name: "DragDropEvent",
        fields: FIELDS_93,
    },
    Definition::Message {
        name: "ExternalFileDropEvent",
        fields: FIELDS_94,
    },
    Definition::Message {
        name: "NotificationResponseEvent",
        fields: FIELDS_95,
    },
    Definition::Message {
        name: "PointerDownOutsideEvent",
        fields: FIELDS_96,
    },
    Definition::Message {
        name: "CloseRequestedEvent",
        fields: FIELDS_97,
    },
    Definition::Message {
        name: "ExtensionEvent",
        fields: FIELDS_98,
    },
    Definition::Message {
        name: "ApplicationActivationEvent",
        fields: FIELDS_99,
    },
    Definition::Message {
        name: "Envelope",
        fields: FIELDS_100,
    },
];
pub(crate) const ROOT_DEFINITION: usize = 100;
