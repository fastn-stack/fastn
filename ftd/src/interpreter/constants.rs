pub static FTD_HIGHLIGHTER: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| regex::Regex::new(r"((;;)( *)(<hl>))( *)(\n?)$").unwrap());

pub const FTD_BREAKPOINT_WIDTH: &str = "ftd#breakpoint-width";
pub const FTD_BREAKPOINT_WIDTH_DATA: &str = "ftd#breakpoint-width-data";

pub const FTD_DEVICE: &str = "ftd#device";
pub const FTD_DEVICE_DATA: &str = "ftd#device-data";
pub const FTD_DEVICE_DATA_MOBILE: &str = "ftd#device-data.mobile";
pub const FTD_DEVICE_DATA_DESKTOP: &str = "ftd#device-data.desktop";

pub const FTD_LENGTH: &str = "ftd#length";
pub const FTD_LENGTH_PX: &str = "ftd#length.px";
pub const FTD_LENGTH_PERCENT: &str = "ftd#length.percent";
pub const FTD_LENGTH_CALC: &str = "ftd#length.calc";
pub const FTD_LENGTH_VH: &str = "ftd#length.vh";
pub const FTD_LENGTH_VW: &str = "ftd#length.vw";
pub const FTD_LENGTH_VMIN: &str = "ftd#length.vmin";
pub const FTD_LENGTH_VMAX: &str = "ftd#length.vmax";
pub const FTD_LENGTH_EM: &str = "ftd#length.em";
pub const FTD_LENGTH_REM: &str = "ftd#length.rem";
pub const FTD_LENGTH_RESPONSIVE: &str = "ftd#length.responsive";

pub const FTD_RESPONSIVE_LENGTH: &str = "ftd#responsive-length";
pub const FTD_RESPONSIVE_LENGTH_DESKTOP: &str = "ftd#responsive-length.desktop";

pub const FTD_ALIGN: &str = "ftd#align";
pub const FTD_ALIGN_TOP_LEFT: &str = "ftd#align.top-left";
pub const FTD_ALIGN_TOP_CENTER: &str = "ftd#align.top-center";
pub const FTD_ALIGN_TOP_RIGHT: &str = "ftd#align.top-right";
pub const FTD_ALIGN_RIGHT: &str = "ftd#align.right";
pub const FTD_ALIGN_LEFT: &str = "ftd#align.left";
pub const FTD_ALIGN_CENTER: &str = "ftd#align.center";
pub const FTD_ALIGN_BOTTOM_LEFT: &str = "ftd#align.bottom-left";
pub const FTD_ALIGN_BOTTOM_CENTER: &str = "ftd#align.bottom-center";
pub const FTD_ALIGN_BOTTOM_RIGHT: &str = "ftd#align.bottom-right";

pub const FTD_RESIZING: &str = "ftd#resizing";
pub const FTD_RESIZING_HUG_CONTENT: &str = "ftd#resizing.hug-content";
pub const FTD_RESIZING_FILL_CONTAINER: &str = "ftd#resizing.fill-container";
pub const FTD_RESIZING_AUTO: &str = "ftd#resizing.auto";
pub const FTD_RESIZING_FIXED: &str = "ftd#resizing.fixed";

pub const FTD_COLOR: &str = "ftd#color";
pub const FTD_COLOR_LIGHT: &str = "ftd#color.light";

pub const FTD_BACKGROUND: &str = "ftd#background";
pub const FTD_BACKGROUND_SOLID: &str = "ftd#background.solid";
pub const FTD_BACKGROUND_IMAGE: &str = "ftd#background.image";
pub const FTD_BACKGROUND_LINEAR_GRADIENT: &str = "ftd#background.linear-gradient";

pub const FTD_LENGTH_PAIR: &str = "ftd#length-pair";
pub const FTD_LENGTH_PAIR_X: &str = "ftd#length-pair.x";
pub const FTD_LENGTH_PAIR_Y: &str = "ftd#length-pair.y";

pub const FTD_BG_IMAGE: &str = "ftd#background-image";
pub const FTD_BG_IMAGE_SRC: &str = "ftd#background-image.src";
pub const FTD_BG_IMAGE_REPEAT: &str = "ftd#background-image.repeat";

pub const FTD_LINEAR_GRADIENT: &str = "ftd#linear-gradient";
pub const FTD_LINEAR_GRADIENT_DIRECTION: &str = "ftd#linear-gradient.direction";
pub const FTD_LINEAR_GRADIENT_COLORS: &str = "ftd#linear-gradient.colors";

pub const FTD_LINEAR_GRADIENT_COLOR: &str = "ftd#linear-gradient-color";
pub const FTD_LINEAR_GRADIENT_COLOR_NAME: &str = "ftd#linear-gradient-color.color";
pub const FTD_LINEAR_GRADIENT_COLOR_START: &str = "ftd#linear-gradient-color.start";
pub const FTD_LINEAR_GRADIENT_COLOR_END: &str = "ftd#linear-gradient-color.end";
pub const FTD_LINEAR_GRADIENT_COLOR_STOP_POSITION: &str = "ftd#linear-gradient-color.stop-position";

pub const FTD_LINEAR_GRADIENT_DIRECTIONS: &str = "ftd#linear-gradient-directions";
pub const FTD_LINEAR_GRADIENT_DIRECTIONS_ANGLE: &str = "ftd#linear-gradient-directions.angle";
pub const FTD_LINEAR_GRADIENT_DIRECTIONS_TURN: &str = "ftd#linear-gradient-directions.turn";
pub const FTD_LINEAR_GRADIENT_DIRECTIONS_LEFT: &str = "ftd#linear-gradient-directions.left";
pub const FTD_LINEAR_GRADIENT_DIRECTIONS_RIGHT: &str = "ftd#linear-gradient-directions.right";
pub const FTD_LINEAR_GRADIENT_DIRECTIONS_TOP: &str = "ftd#linear-gradient-directions.top";
pub const FTD_LINEAR_GRADIENT_DIRECTIONS_BOTTOM: &str = "ftd#linear-gradient-directions.bottom";
pub const FTD_LINEAR_GRADIENT_DIRECTIONS_TOP_LEFT: &str = "ftd#linear-gradient-directions.top-left";
pub const FTD_LINEAR_GRADIENT_DIRECTIONS_TOP_RIGHT: &str =
    "ftd#linear-gradient-directions.top-right";
pub const FTD_LINEAR_GRADIENT_DIRECTIONS_BOTTOM_LEFT: &str =
    "ftd#linear-gradient-directions.bottom-left";
pub const FTD_LINEAR_GRADIENT_DIRECTIONS_BOTTOM_RIGHT: &str =
    "ftd#linear-gradient-directions.bottom-right";

pub const FTD_BACKGROUND_REPEAT: &str = "ftd#background-repeat";
pub const FTD_BACKGROUND_REPEAT_BOTH_REPEAT: &str = "ftd#background-repeat.repeat";
pub const FTD_BACKGROUND_REPEAT_X_REPEAT: &str = "ftd#background-repeat.repeat-x";
pub const FTD_BACKGROUND_REPEAT_Y_REPEAT: &str = "ftd#background-repeat.repeat-y";
pub const FTD_BACKGROUND_REPEAT_NO_REPEAT: &str = "ftd#background-repeat.no-repeat";
pub const FTD_BACKGROUND_REPEAT_SPACE: &str = "ftd#background-repeat.space";
pub const FTD_BACKGROUND_REPEAT_ROUND: &str = "ftd#background-repeat.round";

pub const FTD_BACKGROUND_SIZE: &str = "ftd#background-size";
pub const FTD_BACKGROUND_SIZE_AUTO: &str = "ftd#background-size.auto";
pub const FTD_BACKGROUND_SIZE_COVER: &str = "ftd#background-size.cover";
pub const FTD_BACKGROUND_SIZE_CONTAIN: &str = "ftd#background-size.contain";
pub const FTD_BACKGROUND_SIZE_LENGTH: &str = "ftd#background-size.length";

pub const FTD_BACKGROUND_POSITION: &str = "ftd#background-position";
pub const FTD_BACKGROUND_POSITION_LEFT: &str = "ftd#background-position.left";
pub const FTD_BACKGROUND_POSITION_CENTER: &str = "ftd#background-position.center";
pub const FTD_BACKGROUND_POSITION_RIGHT: &str = "ftd#background-position.right";
pub const FTD_BACKGROUND_POSITION_LEFT_TOP: &str = "ftd#background-position.left-top";
pub const FTD_BACKGROUND_POSITION_LEFT_CENTER: &str = "ftd#background-position.left-center";
pub const FTD_BACKGROUND_POSITION_LEFT_BOTTOM: &str = "ftd#background-position.left-bottom";
pub const FTD_BACKGROUND_POSITION_CENTER_TOP: &str = "ftd#background-position.center-top";
pub const FTD_BACKGROUND_POSITION_CENTER_CENTER: &str = "ftd#background-position.center-center";
pub const FTD_BACKGROUND_POSITION_CENTER_BOTTOM: &str = "ftd#background-position.center-bottom";
pub const FTD_BACKGROUND_POSITION_RIGHT_TOP: &str = "ftd#background-position.right-top";
pub const FTD_BACKGROUND_POSITION_RIGHT_CENTER: &str = "ftd#background-position.right-center";
pub const FTD_BACKGROUND_POSITION_RIGHT_BOTTOM: &str = "ftd#background-position.right-bottom";
pub const FTD_BACKGROUND_POSITION_LENGTH: &str = "ftd#background-position.length";

pub const FTD_RAW_IMAGE_SRC: &str = "ftd#raw-image-src";

pub const FTD_IMAGE_SRC: &str = "ftd#image-src";
pub const FTD_IMAGE_SRC_LIGHT: &str = "ftd#image-src.light";
pub const FTD_IMAGE_SRC_DARK: &str = "ftd#image-src.dark";

pub const FTD_IMAGE_FIT: &str = "ftd#image-fit";
pub const FTD_IMAGE_FIT_NONE: &str = "ftd#image-fit.none";
pub const FTD_IMAGE_FIT_COVER: &str = "ftd#image-fit.cover";
pub const FTD_IMAGE_FIT_CONTAIN: &str = "ftd#image-fit.contain";
pub const FTD_IMAGE_FIT_FILL: &str = "ftd#image-fit.fill";
pub const FTD_IMAGE_FIT_SCALE_DOWN: &str = "ftd#image-fit.scale-down";

pub const FTD_VIDEO_SRC: &str = "ftd#video-src";
pub const FTD_VIDEO_SRC_LIGHT: &str = "ftd#video-src.light";
pub const FTD_VIDEO_SRC_DARK: &str = "ftd#video-src.dark";

pub const FTD_VIDEO_POSTER: &str = "ftd#video-poster";
pub const FTD_VIDEO_POSTER_LIGHT: &str = "ftd#video-poster.light";
pub const FTD_VIDEO_POSTER_DARK: &str = "ftd#video-poster.dark";

pub const FTD_VIDEO_AUTOPLAY: &str = "ftd#video-autoplay";
pub const FTD_VIDEO_MUTED: &str = "ftd#muted";
pub const FTD_VIDEO_CONTROLS: &str = "ftd#video-controls";
pub const FTD_VIDEO_LOOP: &str = "ftd#video-loop";

pub const FTD_SPACING: &str = "ftd#spacing";
pub const FTD_SPACING_FIXED: &str = "ftd#spacing.fixed";
pub const FTD_SPACING_SPACE_BETWEEN: &str = "ftd#spacing.space-between";
pub const FTD_SPACING_SPACE_AROUND: &str = "ftd#spacing.space-around";
pub const FTD_SPACING_SPACE_EVENLY: &str = "ftd#spacing.space-evenly";

pub const FTD_ALIGN_SELF: &str = "ftd#align-self";
pub const FTD_ALIGN_SELF_START: &str = "ftd#align-self.start";
pub const FTD_ALIGN_SELF_CENTER: &str = "ftd#align-self.center";
pub const FTD_ALIGN_SELF_END: &str = "ftd#align-self.end";

pub const FTD_TEXT_ALIGN: &str = "ftd#text-align";
pub const FTD_TEXT_ALIGN_START: &str = "ftd#text-align.start";
pub const FTD_TEXT_ALIGN_CENTER: &str = "ftd#text-align.center";
pub const FTD_TEXT_ALIGN_END: &str = "ftd#text-align.end";
pub const FTD_TEXT_ALIGN_JUSTIFY: &str = "ftd#text-align.justify";

pub const FTD_SHADOW: &str = "ftd#shadow";
pub const FTD_SHADOW_COLOR: &str = "ftd#shadow.color";

// FTD overflow(todo docs link)
pub const FTD_OVERFLOW: &str = "ftd#overflow";
pub const FTD_OVERFLOW_SCROLL: &str = "ftd#overflow.scroll";
pub const FTD_OVERFLOW_VISIBLE: &str = "ftd#overflow.visible";
pub const FTD_OVERFLOW_HIDDEN: &str = "ftd#overflow.hidden";
pub const FTD_OVERFLOW_AUTO: &str = "ftd#overflow.auto";

pub const FTD_RESIZE: &str = "ftd#resize";
pub const FTD_RESIZE_HORIZONTAL: &str = "ftd#resize.horizontal";
pub const FTD_RESIZE_VERTICAL: &str = "ftd#resize.vertical";
pub const FTD_RESIZE_BOTH: &str = "ftd#resize.both";

// FTD cursor(todo docs link)
pub const FTD_CURSOR: &str = "ftd#cursor";
pub const FTD_CURSOR_DEFAULT: &str = "ftd#cursor.default";
pub const FTD_CURSOR_NONE: &str = "ftd#cursor.none";
pub const FTD_CURSOR_CONTEXT_MENU: &str = "ftd#cursor.context-menu";
pub const FTD_CURSOR_HELP: &str = "ftd#cursor.help";
pub const FTD_CURSOR_POINTER: &str = "ftd#cursor.pointer";
pub const FTD_CURSOR_PROGRESS: &str = "ftd#cursor.progress";
pub const FTD_CURSOR_WAIT: &str = "ftd#cursor.wait";
pub const FTD_CURSOR_CELL: &str = "ftd#cursor.cell";
pub const FTD_CURSOR_CROSSHAIR: &str = "ftd#cursor.crosshair";
pub const FTD_CURSOR_TEXT: &str = "ftd#cursor.text";
pub const FTD_CURSOR_VERTICAL_TEXT: &str = "ftd#cursor.vertical-text";
pub const FTD_CURSOR_ALIAS: &str = "ftd#cursor.alias";
pub const FTD_CURSOR_COPY: &str = "ftd#cursor.copy";
pub const FTD_CURSOR_MOVE: &str = "ftd#cursor.move";
pub const FTD_CURSOR_NO_DROP: &str = "ftd#cursor.no-drop";
pub const FTD_CURSOR_NOT_ALLOWED: &str = "ftd#cursor.not-allowed";
pub const FTD_CURSOR_GRAB: &str = "ftd#cursor.grab";
pub const FTD_CURSOR_GRABBING: &str = "ftd#cursor.grabbing";
pub const FTD_CURSOR_E_RESIZE: &str = "ftd#cursor.e-resize";
pub const FTD_CURSOR_N_RESIZE: &str = "ftd#cursor.n-resize";
pub const FTD_CURSOR_NE_RESIZE: &str = "ftd#cursor.ne-resize";
pub const FTD_CURSOR_NW_RESIZE: &str = "ftd#cursor.nw-resize";
pub const FTD_CURSOR_S_RESIZE: &str = "ftd#cursor.s-resize";
pub const FTD_CURSOR_SE_RESIZE: &str = "ftd#cursor.se-resize";
pub const FTD_CURSOR_SW_RESIZE: &str = "ftd#cursor.sw-resize";
pub const FTD_CURSOR_W_RESIZE: &str = "ftd#cursor.w-resize";
pub const FTD_CURSOR_EW_RESIZE: &str = "ftd#cursor.ew-resize";
pub const FTD_CURSOR_NS_RESIZE: &str = "ftd#cursor.ns-resize";
pub const FTD_CURSOR_NESW_RESIZE: &str = "ftd#cursor.nesw-resize";
pub const FTD_CURSOR_NWSE_RESIZE: &str = "ftd#cursor.nwse-resize";
pub const FTD_CURSOR_COL_RESIZE: &str = "ftd#cursor.col-resize";
pub const FTD_CURSOR_ROW_RESIZE: &str = "ftd#cursor.row-resize";
pub const FTD_CURSOR_ALL_SCROLL: &str = "ftd#cursor.all-scroll";
pub const FTD_CURSOR_ZOOM_IN: &str = "ftd#cursor.zoom-in";
pub const FTD_CURSOR_ZOOM_OUT: &str = "ftd#cursor.zoom-out";

pub const FTD_FONT_SIZE: &str = "ftd#font-size";
pub const FTD_FONT_SIZE_PX: &str = "ftd#font-size.px";
pub const FTD_FONT_SIZE_EM: &str = "ftd#font-size.em";
pub const FTD_FONT_SIZE_REM: &str = "ftd#font-size.rem";

pub const FTD_TYPE: &str = "ftd#type";

pub const FTD_RESPONSIVE_TYPE: &str = "ftd#responsive-type";
pub const FTD_RESPONSIVE_TYPE_DESKTOP: &str = "ftd#responsive-type.desktop";

pub const FTD_ANCHOR: &str = "ftd#anchor";
pub const FTD_ANCHOR_WINDOW: &str = "ftd#anchor.window";
pub const FTD_ANCHOR_PARENT: &str = "ftd#anchor.parent";
pub const FTD_ANCHOR_ID: &str = "ftd#anchor.id";

pub const FTD_COLOR_SCHEME: &str = "ftd#color-scheme";
pub const FTD_BACKGROUND_COLOR: &str = "ftd#background-colors";
pub const FTD_CTA_COLOR: &str = "ftd#cta-colors";
pub const FTD_PST: &str = "ftd#pst";
pub const FTD_BTB: &str = "ftd#btb";
pub const FTD_CUSTOM_COLORS: &str = "ftd#custom-colors";

pub const FTD_TYPE_DATA: &str = "ftd#type-data";

pub const FTD_TEXT_INPUT_TYPE: &str = "ftd#text-input-type";
pub const FTD_TEXT_INPUT_TYPE_TEXT: &str = "ftd#text-input-type.text";
pub const FTD_TEXT_INPUT_TYPE_EMAIL: &str = "ftd#text-input-type.email";
pub const FTD_TEXT_INPUT_TYPE_PASSWORD: &str = "ftd#text-input-type.password";
pub const FTD_TEXT_INPUT_TYPE_URL: &str = "ftd#text-input-type.url";
pub const FTD_TEXT_INPUT_TYPE_DATETIME: &str = "ftd#text-input-type.datetime";
pub const FTD_TEXT_INPUT_TYPE_DATE: &str = "ftd#text-input-type.date";
pub const FTD_TEXT_INPUT_TYPE_TIME: &str = "ftd#text-input-type.time";
pub const FTD_TEXT_INPUT_TYPE_MONTH: &str = "ftd#text-input-type.month";
pub const FTD_TEXT_INPUT_TYPE_WEEK: &str = "ftd#text-input-type.week";
pub const FTD_TEXT_INPUT_TYPE_COLOR: &str = "ftd#text-input-type.color";
pub const FTD_TEXT_INPUT_TYPE_FILE: &str = "ftd#text-input-type.file";

pub const FTD_REGION: &str = "ftd#region";
pub const FTD_REGION_H1: &str = "ftd#region.h1";
pub const FTD_REGION_H2: &str = "ftd#region.h2";
pub const FTD_REGION_H3: &str = "ftd#region.h3";
pub const FTD_REGION_H4: &str = "ftd#region.h4";
pub const FTD_REGION_H5: &str = "ftd#region.h5";
pub const FTD_REGION_H6: &str = "ftd#region.h6";

pub const FTD_DISPLAY: &str = "ftd#display";
pub const FTD_DISPLAY_BLOCK: &str = "ftd#display.block";
pub const FTD_DISPLAY_INLINE: &str = "ftd#display.inline";
pub const FTD_DISPLAY_INLINE_BLOCK: &str = "ftd#display.inline-block";

pub const FTD_WHITESPACE: &str = "ftd#white-space";
pub const FTD_WHITESPACE_NORMAL: &str = "ftd#white-space.normal";
pub const FTD_WHITESPACE_NOWRAP: &str = "ftd#white-space.nowrap";
pub const FTD_WHITESPACE_PRE: &str = "ftd#white-space.pre";
pub const FTD_WHITESPACE_PREWRAP: &str = "ftd#white-space.pre-wrap";
pub const FTD_WHITESPACE_PRELINE: &str = "ftd#white-space.pre-line";
pub const FTD_WHITESPACE_BREAKSPACES: &str = "ftd#white-space.break-spaces";

pub const FTD_TEXT_TRANSFORM: &str = "ftd#text-transform";
pub const FTD_TEXT_TRANSFORM_NONE: &str = "ftd#text-transform.none";
pub const FTD_TEXT_TRANSFORM_CAPITALIZE: &str = "ftd#text-transform.capitalize";
pub const FTD_TEXT_TRANSFORM_UPPERCASE: &str = "ftd#text-transform.uppercase";
pub const FTD_TEXT_TRANSFORM_LOWERCASE: &str = "ftd#text-transform.lowercase";
pub const FTD_TEXT_TRANSFORM_INITIAL: &str = "ftd#text-transform.initial";
pub const FTD_TEXT_TRANSFORM_INHERIT: &str = "ftd#text-transform.inherit";

pub const FTD_LOADING: &str = "ftd#loading";
pub const FTD_LOADING_EAGER: &str = "ftd#loading.eager";
pub const FTD_LOADING_LAZY: &str = "ftd#loading.lazy";

pub const FTD_SPECIAL_VALUE: &str = "$VALUE";
pub const FTD_SPECIAL_CHECKED: &str = "$CHECKED";
pub const FTD_INHERITED: &str = "inherited";
pub const FTD_LOOP_COUNTER: &str = "LOOP.COUNTER";
pub const FTD_DEFAULT_TYPES: &str = "default-types";
pub const FTD_DEFAULT_COLORS: &str = "default-colors";
pub const FTD_NONE: &str = "none";
pub const FTD_NO_VALUE: &str = "NO-VALUE";
pub const FTD_IGNORE_KEY: &str = "IGNORE-KEY";
pub const FTD_REMOVE_KEY: &str = "REMOVE-KEY";

pub const FTD_BORDER_STYLE: &str = "ftd#border-style";
pub const FTD_BORDER_STYLE_DOTTED: &str = "ftd#border-style.dotted";
pub const FTD_BORDER_STYLE_DASHED: &str = "ftd#border-style.dashed";
pub const FTD_BORDER_STYLE_SOLID: &str = "ftd#border-style.solid";
pub const FTD_BORDER_STYLE_DOUBLE: &str = "ftd#border-style.double";
pub const FTD_BORDER_STYLE_GROOVE: &str = "ftd#border-style.groove";
pub const FTD_BORDER_STYLE_RIDGE: &str = "ftd#border-style.ridge";
pub const FTD_BORDER_STYLE_INSET: &str = "ftd#border-style.inset";
pub const FTD_BORDER_STYLE_OUTSET: &str = "ftd#border-style.outset";

pub const FTD_EMPTY_STR: &str = "";
pub const FTD_VALUE_UNCHANGED: &str = "unchanged";
pub const FTD_TEXT_STYLE: &str = "ftd#text-style";
pub const FTD_TEXT_STYLE_ITALIC: &str = "ftd#text-style.italic";
pub const FTD_TEXT_STYLE_UNDERLINE: &str = "ftd#text-style.underline";
pub const FTD_TEXT_STYLE_STRIKE: &str = "ftd#text-style.strike";
pub const FTD_TEXT_STYLE_WEIGHT_HEAVY: &str = "ftd#text-style.heavy";
pub const FTD_TEXT_STYLE_WEIGHT_EXTRA_BOLD: &str = "ftd#text-style.extra-bold";
pub const FTD_TEXT_STYLE_WEIGHT_BOLD: &str = "ftd#text-style.bold";
pub const FTD_TEXT_STYLE_WEIGHT_SEMI_BOLD: &str = "ftd#text-style.semi-bold";
pub const FTD_TEXT_STYLE_WEIGHT_MEDIUM: &str = "ftd#text-style.medium";
pub const FTD_TEXT_STYLE_WEIGHT_REGULAR: &str = "ftd#text-style.regular";
pub const FTD_TEXT_STYLE_WEIGHT_LIGHT: &str = "ftd#text-style.light";
pub const FTD_TEXT_STYLE_WEIGHT_EXTRA_LIGHT: &str = "ftd#text-style.extra-light";
pub const FTD_TEXT_STYLE_WEIGHT_HAIRLINE: &str = "ftd#text-style.hairline";

pub const FTD_LINK_REL: &str = "ftd#link-rel";
pub const FTD_LINK_REL_NO_FOLLOW: &str = "ftd#link-rel.no-follow";
pub const FTD_LINK_REL_SPONSORED: &str = "ftd#link-rel.sponsored";
pub const FTD_LINK_REL_UGC: &str = "ftd#link-rel.ugc";

pub const FTD_BACKDROP_MULTI: &str = "ftd#backdrop-multi";
pub const FTD_BACKDROP_FILTER: &str = "ftd#backdrop-filter";
pub const FTD_BACKDROP_FILTER_BLUR: &str = "ftd#backdrop-filter.blur";
pub const FTD_BACKDROP_FILTER_BRIGHTNESS: &str = "ftd#backdrop-filter.brightness";
pub const FTD_BACKDROP_FILTER_CONTRAST: &str = "ftd#backdrop-filter.contrast";
pub const FTD_BACKDROP_FILTER_GRAYSCALE: &str = "ftd#backdrop-filter.grayscale";
pub const FTD_BACKDROP_FILTER_INVERT: &str = "ftd#backdrop-filter.invert";
pub const FTD_BACKDROP_FILTER_OPACITY: &str = "ftd#backdrop-filter.opacity";
pub const FTD_BACKDROP_FILTER_SEPIA: &str = "ftd#backdrop-filter.sepia";
pub const FTD_BACKDROP_FILTER_SATURATE: &str = "ftd#backdrop-filter.saturate";
pub const FTD_BACKDROP_FILTER_MULTI: &str = "ftd#backdrop-filter.multi";

pub const FTD_MASK: &str = "ftd#mask";
pub const FTD_MASK_IMAGE: &str = "ftd#mask-image";
pub const FTD_MASK_IMAGE_SRC: &str = "ftd#mask-image.src";
pub const FTD_MASK_IMAGE_LINEAR_GRADIENT: &str = "ftd#mask-image.linear-gradient";

pub const FTD_MASK_CLIP: &str = "ftd#mask-clip";
pub const FTD_MASK_CLIP_CONTENT_BOX: &str = "ftd#mask-clip.content-box";
pub const FTD_MASK_CLIP_PADDING_BOX: &str = "ftd#mask-clip.padding-box";
pub const FTD_MASK_CLIP_BORDER_BOX: &str = "ftd#mask-clip.border-box";
pub const FTD_MASK_CLIP_FILL_BOX: &str = "ftd#mask-clip.fill-box";
pub const FTD_MASK_CLIP_STROKE_BOX: &str = "ftd#mask-clip.stroke-box";
pub const FTD_MASK_CLIP_VIEW_BOX: &str = "ftd#mask-clip.view-box";
pub const FTD_MASK_CLIP_NO_CLIP: &str = "ftd#mask-clip.no-clip";
pub const FTD_MASK_CLIP_MULTI_VALUES: &str = "ftd#mask-clip.multi";

pub const FTD_MASK_CLIP_MULTI: &str = "ftd#mask-clip-multi";
