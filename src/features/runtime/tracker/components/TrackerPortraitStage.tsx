import { ImagePlus, RotateCcw, ZoomIn, ZoomOut } from "lucide-react";
import { useRef, type CSSProperties, type PointerEvent as ReactPointerEvent, type ReactNode } from "react";
import { cn } from "../../../../shared/lib/utils";
import {
  TRACKER_PORTRAIT_DEFAULT_FOCUS_X,
  TRACKER_PORTRAIT_DEFAULT_FOCUS_Y,
  TRACKER_PORTRAIT_DEFAULT_ZOOM,
  TRACKER_PORTRAIT_EXPRESSION_FOCUS_Y_MAX,
  TRACKER_PORTRAIT_MAX_ZOOM,
  TRACKER_PORTRAIT_MIN_ZOOM,
  TRACKER_PORTRAIT_ZOOM_STEP,
  TRACKER_PROFILE_PORTRAIT_FRAME_STAGE_CLASS,
} from "./tracker-data-sidebar.constants";
import { clampNumber } from "./tracker-display.helpers";
import {
  TRACKER_PROFILE_PORTRAIT_FADE_CLASS_BY_OUTSIDE_SIDE,
  TRACKER_PROFILE_PORTRAIT_LOWER_OUTSIDE_FRAME_CLASS_BY_SIDE,
  TRACKER_PROFILE_PORTRAIT_LOWER_OUTSIDE_RADIUS_CLASS_BY_SIDE,
  type TrackerProfileSide,
} from "../lib/tracker-profile-layout";
import { TrackerPortraitStageBackdrop } from "./tracker-data-sidebar.controls";

export type TrackerPortraitStageMediaKind = "expression" | "art";
type TrackerPortraitStageFrameTone = "featured" | "persona";
type TrackerPortraitStageStyle = CSSProperties & Record<`--${string}`, string | number>;

interface TrackerPortraitStageView {
  defaultX?: number;
  defaultY?: number;
  defaultZoom?: number;
  x?: number;
  y?: number;
  zoom?: number;
  onChange?: (focusX: number, focusY: number, zoom: number) => void;
}

interface TrackerPortraitStageUploadAction {
  ariaLabel: string;
  onClick: () => void;
  title: string;
}

interface PortraitDragState {
  pointerId: number;
  startClientX: number;
  startClientY: number;
  startFocusX: number;
  startFocusY: number;
  stageHeight: number;
  stageWidth: number;
  zoom: number;
}

const PORTRAIT_STAGE_BASE_CLASS = "tracker-portrait-stage";
const PORTRAIT_FRAME_TONE_CLASS = {
  featured: "tracker-portrait-stage--featured",
  persona: "tracker-portrait-stage--persona",
} satisfies Record<TrackerPortraitStageFrameTone, string>;
const PORTRAIT_STAGE_INNER_GLOW_CLASS = "tracker-portrait-stage-inner-glow";
const PORTRAIT_TONE_OVERLAY_CLASS = "tracker-portrait-tone-overlay";
const PORTRAIT_MEDIA_DRAG_SURFACE_CLASS = "tracker-portrait-media-drag-surface";
const PORTRAIT_MEDIA_OFFSET_CLASS = "tracker-portrait-media-offset";
const SPRITE_IMAGE_CLASS = "tracker-portrait-sprite-image";
const ART_IMAGE_CLASS = "tracker-portrait-art-image";
const EMPTY_PORTRAIT_CLASS = "tracker-portrait-empty";
const EMPTY_PORTRAIT_FLOOR_CLASS = "tracker-portrait-empty-floor";
const EMPTY_PORTRAIT_AVATAR_CLASS = "tracker-portrait-empty-avatar";
const EMPTY_PORTRAIT_INITIAL_CLASS = "tracker-portrait-empty-initial";
const EMPTY_PORTRAIT_UPLOAD_BADGE_CLASS = "tracker-portrait-upload-badge";
const PORTRAIT_VIEW_CONTROLS_CLASS = "tracker-portrait-view-controls";
const PORTRAIT_VIEW_BUTTON_CLASS = "tracker-portrait-view-button";
const PORTRAIT_EDGE_OVERLAY_CLASS = "tracker-portrait-edge-overlay";
const PORTRAIT_TOP_GLEAM_BASE_CLASS = "tracker-portrait-top-gleam";
const PORTRAIT_TOP_GLEAM_CLASS_BY_SIDE = {
  left: "tracker-portrait-top-gleam--left",
  right: "tracker-portrait-top-gleam--right",
} satisfies Record<TrackerProfileSide, string>;
const PORTRAIT_BOTTOM_HIGHLIGHT_BASE_CLASS = "tracker-portrait-bottom-highlight";
const PORTRAIT_BOTTOM_HIGHLIGHT_CLASS_BY_SIDE = {
  left: "tracker-portrait-bottom-highlight--left",
  right: "tracker-portrait-bottom-highlight--right",
} satisfies Record<TrackerProfileSide, string>;
const PORTRAIT_SIDE_FADE_BASE_CLASS = "tracker-portrait-side-fade";
const PORTRAIT_UPLOAD_HIT_TARGET_CLASS = "tracker-portrait-upload-hit-target";
const PORTRAIT_UPLOAD_BUTTON_CLASS = "tracker-portrait-upload-button";
const EMPTY_AVATAR_STAGE_STYLE: TrackerPortraitStageStyle = {
  "--tracker-profile-surface":
    "radial-gradient(ellipse at 50% 42%, color-mix(in srgb, var(--muted-foreground) 10%, transparent) 0%, transparent 42%), linear-gradient(180deg, color-mix(in srgb, var(--card) 82%, var(--background) 18%) 0%, color-mix(in srgb, var(--background) 92%, var(--card) 8%) 100%)",
  "--tracker-profile-surface-blend": "normal",
  "--tracker-profile-surface-layer":
    "radial-gradient(ellipse at 50% 42%, color-mix(in srgb, var(--foreground) 7%, transparent) 0%, transparent 46%)",
  "--tracker-profile-tint-opacity": "0.16",
  "--tracker-profile-display-solid": "color-mix(in srgb, var(--muted-foreground) 48%, var(--background) 52%)",
  "--tracker-profile-accent-solid": "color-mix(in srgb, var(--muted-foreground) 38%, var(--background) 62%)",
  "--tracker-profile-portrait-base":
    "radial-gradient(ellipse at 50% 42%, color-mix(in srgb, var(--muted-foreground) 14%, transparent) 0%, transparent 34%), linear-gradient(180deg, color-mix(in srgb, var(--card) 88%, var(--background) 12%) 0%, color-mix(in srgb, var(--background) 94%, var(--card) 6%) 100%)",
  "--tracker-profile-portrait-veil":
    "radial-gradient(ellipse at 50% 43%, transparent 0%, transparent 30%, color-mix(in srgb, var(--background) 48%, transparent) 72%, color-mix(in srgb, var(--background) 82%, transparent) 100%), linear-gradient(90deg, color-mix(in srgb, var(--background) 58%, transparent) 0%, transparent 25%, transparent 75%, color-mix(in srgb, var(--background) 58%, transparent) 100%)",
  "--tracker-profile-portrait-light":
    "radial-gradient(ellipse at 50% 42%, color-mix(in srgb, var(--muted-foreground) 12%, transparent) 0%, transparent 36%)",
  "--tracker-profile-portrait-light-opacity": "0.36",
  "--tracker-profile-portrait-rim":
    "linear-gradient(180deg, color-mix(in srgb, var(--foreground) 6%, transparent) 0%, transparent 22%), linear-gradient(90deg, transparent 0%, color-mix(in srgb, var(--muted-foreground) 18%, transparent) 50%, transparent 100%)",
  "--tracker-profile-portrait-rim-opacity": "0.32",
  "--tracker-profile-portrait-bottom-glow-opacity": "0.18",
  "--tracker-profile-portrait-bottom-rule-opacity": "0.28",
  "--tracker-profile-portrait-side-mask-opacity": "0.22",
  "--tracker-portrait-frame-accent": "color-mix(in srgb, var(--tracker-profile-rule) 72%, var(--muted-foreground) 28%)",
};

function normalizeZoom(value: number) {
  return Math.round(clampNumber(value, TRACKER_PORTRAIT_MIN_ZOOM, TRACKER_PORTRAIT_MAX_ZOOM) * 100) / 100;
}

export function TrackerPortraitStage({
  accessibleLabel,
  className,
  media,
  mediaKind,
  outsideSide = "right",
  frameTone = "featured",
  placeholder,
  placeholderVariant = "initial",
  stageSizeClassName = TRACKER_PROFILE_PORTRAIT_FRAME_STAGE_CLASS,
  uploadAction,
  view,
}: {
  accessibleLabel: string;
  className?: string;
  media: string | null;
  mediaKind: TrackerPortraitStageMediaKind | null;
  outsideSide?: TrackerProfileSide;
  frameTone?: TrackerPortraitStageFrameTone;
  placeholder: ReactNode;
  placeholderVariant?: "initial" | "avatar";
  stageSizeClassName?: string;
  uploadAction?: TrackerPortraitStageUploadAction;
  view?: TrackerPortraitStageView;
}) {
  const dragStateRef = useRef<PortraitDragState | null>(null);
  const isArt = mediaKind === "art";
  const isExpression = mediaKind === "expression";
  const isEmptyAvatarPlaceholder = !media && placeholderVariant === "avatar";
  const canAdjustView = !!media && !!view?.onChange;
  const defaultX = view?.defaultX ?? TRACKER_PORTRAIT_DEFAULT_FOCUS_X;
  const defaultY = view?.defaultY ?? TRACKER_PORTRAIT_DEFAULT_FOCUS_Y;
  const defaultZoom = normalizeZoom(view?.defaultZoom ?? TRACKER_PORTRAIT_DEFAULT_ZOOM);
  const focusYMax = isExpression ? TRACKER_PORTRAIT_EXPRESSION_FOCUS_Y_MAX : 100;
  const focusX = clampNumber(typeof view?.x === "number" ? view.x : defaultX, 0, 100);
  const focusY = clampNumber(typeof view?.y === "number" ? view.y : defaultY, 0, focusYMax);
  const visualFocusY = clampNumber(focusY, 0, 100);
  const zoom = normalizeZoom(typeof view?.zoom === "number" ? view.zoom : defaultZoom);
  const zoomPercent = Math.round(zoom * 100);
  const expressionOverdragY = isExpression ? Math.max(0, focusY - 100) : 0;
  const mediaOffsetStyle =
    expressionOverdragY > 0 ? { transform: `translate3d(0, ${expressionOverdragY}%, 0)` } : undefined;
  const imageStyle = {
    objectPosition: `${focusX}% ${visualFocusY}%`,
    transform: `scale(${zoom})`,
    transformOrigin: `${focusX}% ${visualFocusY}%`,
  };
  const setPortraitView = (nextFocusX: number, nextFocusY: number, nextZoom = zoom) => {
    if (!view?.onChange) return;
    const normalizedX = Math.round(clampNumber(nextFocusX, 0, 100));
    const normalizedY = Math.round(clampNumber(nextFocusY, 0, focusYMax));
    const normalizedZoom = normalizeZoom(nextZoom);
    if (normalizedX === Math.round(focusX) && normalizedY === Math.round(focusY) && normalizedZoom === zoom) return;
    view.onChange(normalizedX, normalizedY, normalizedZoom);
  };
  const resetPortraitView = () => setPortraitView(defaultX, defaultY, defaultZoom);
  const updateZoom = (amount: number) => setPortraitView(focusX, focusY, zoom + amount);
  const handlePointerDown = (event: ReactPointerEvent<HTMLDivElement>) => {
    if (!canAdjustView || event.button !== 0) return;
    const rect = event.currentTarget.getBoundingClientRect();
    dragStateRef.current = {
      pointerId: event.pointerId,
      startClientX: event.clientX,
      startClientY: event.clientY,
      startFocusX: focusX,
      startFocusY: focusY,
      stageHeight: Math.max(1, rect.height),
      stageWidth: Math.max(1, rect.width),
      zoom,
    };
    event.currentTarget.setPointerCapture(event.pointerId);
    event.preventDefault();
  };
  const handlePointerMove = (event: ReactPointerEvent<HTMLDivElement>) => {
    const dragState = dragStateRef.current;
    if (!dragState || dragState.pointerId !== event.pointerId) return;
    const zoomWeight = Math.max(0.75, dragState.zoom);
    const nextFocusX =
      dragState.startFocusX + ((event.clientX - dragState.startClientX) / dragState.stageWidth) * (100 / zoomWeight);
    const nextFocusY =
      dragState.startFocusY + ((event.clientY - dragState.startClientY) / dragState.stageHeight) * (100 / zoomWeight);
    setPortraitView(nextFocusX, nextFocusY, dragState.zoom);
  };
  const endDrag = (event: ReactPointerEvent<HTMLDivElement>) => {
    if (dragStateRef.current?.pointerId !== event.pointerId) return;
    dragStateRef.current = null;
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
  };

  return (
    <div
      className={cn(
        PORTRAIT_STAGE_BASE_CLASS,
        PORTRAIT_FRAME_TONE_CLASS[frameTone],
        stageSizeClassName,
        TRACKER_PROFILE_PORTRAIT_LOWER_OUTSIDE_FRAME_CLASS_BY_SIDE[outsideSide],
        uploadAction && "hover:border-[var(--primary)]/45",
        canAdjustView ? "cursor-grab active:cursor-grabbing" : "cursor-default",
        className,
      )}
      style={isEmptyAvatarPlaceholder ? EMPTY_AVATAR_STAGE_STYLE : undefined}
    >
      <div className={PORTRAIT_STAGE_INNER_GLOW_CLASS} />
      <TrackerPortraitStageBackdrop media={media} />
      <div className={cn(PORTRAIT_TONE_OVERLAY_CLASS, isExpression ? "opacity-95" : "opacity-80")} />
      {media ? (
        <div
          className={PORTRAIT_MEDIA_DRAG_SURFACE_CLASS}
          onPointerDown={handlePointerDown}
          onPointerMove={handlePointerMove}
          onPointerCancel={endDrag}
          onPointerUp={endDrag}
          title={canAdjustView ? "Drag to reposition portrait" : undefined}
        >
          <div className={PORTRAIT_MEDIA_OFFSET_CLASS} style={mediaOffsetStyle}>
            <img
              src={media}
              alt=""
              className={cn("z-[1]", isExpression && SPRITE_IMAGE_CLASS, isArt && ART_IMAGE_CLASS)}
              style={imageStyle}
              draggable={false}
            />
          </div>
        </div>
      ) : (
        <div className={EMPTY_PORTRAIT_CLASS}>
          {placeholderVariant === "avatar" && <div className={EMPTY_PORTRAIT_FLOOR_CLASS} />}
          <div
            className={cn(placeholderVariant === "avatar" ? EMPTY_PORTRAIT_AVATAR_CLASS : EMPTY_PORTRAIT_INITIAL_CLASS)}
          >
            <span className="translate-y-px">{placeholder}</span>
            {placeholderVariant === "avatar" && uploadAction && (
              <span className={EMPTY_PORTRAIT_UPLOAD_BADGE_CLASS}>
                <ImagePlus size="0.6875rem" />
              </span>
            )}
          </div>
        </div>
      )}
      {canAdjustView && (
        <div className={PORTRAIT_VIEW_CONTROLS_CLASS}>
          <button
            type="button"
            onClick={(event) => {
              event.stopPropagation();
              updateZoom(-TRACKER_PORTRAIT_ZOOM_STEP);
            }}
            title="Zoom portrait out"
            aria-label="Zoom portrait out"
            className={PORTRAIT_VIEW_BUTTON_CLASS}
          >
            <ZoomOut size="0.75rem" />
          </button>
          <button
            type="button"
            onClick={(event) => {
              event.stopPropagation();
              updateZoom(TRACKER_PORTRAIT_ZOOM_STEP);
            }}
            title="Zoom portrait in"
            aria-label="Zoom portrait in"
            className={PORTRAIT_VIEW_BUTTON_CLASS}
          >
            <ZoomIn size="0.75rem" />
          </button>
          <button
            type="button"
            onClick={(event) => {
              event.stopPropagation();
              resetPortraitView();
            }}
            title="Reset portrait view"
            aria-label="Reset portrait view"
            className={PORTRAIT_VIEW_BUTTON_CLASS}
          >
            <RotateCcw size="0.6875rem" />
          </button>
          <span className="sr-only">Portrait zoom {zoomPercent}%</span>
        </div>
      )}
      <div className={cn(PORTRAIT_TOP_GLEAM_BASE_CLASS, PORTRAIT_TOP_GLEAM_CLASS_BY_SIDE[outsideSide])} />
      <div className={cn(PORTRAIT_BOTTOM_HIGHLIGHT_BASE_CLASS, PORTRAIT_BOTTOM_HIGHLIGHT_CLASS_BY_SIDE[outsideSide])} />
      <div
        className={cn(
          PORTRAIT_EDGE_OVERLAY_CLASS,
          TRACKER_PROFILE_PORTRAIT_LOWER_OUTSIDE_RADIUS_CLASS_BY_SIDE[outsideSide],
        )}
      />
      <div
        className={cn(PORTRAIT_SIDE_FADE_BASE_CLASS, TRACKER_PROFILE_PORTRAIT_FADE_CLASS_BY_OUTSIDE_SIDE[outsideSide])}
      />
      {uploadAction && !media && (
        <button
          type="button"
          onClick={uploadAction.onClick}
          title={uploadAction.title}
          aria-label={uploadAction.ariaLabel}
          className={cn(
            PORTRAIT_UPLOAD_HIT_TARGET_CLASS,
            TRACKER_PROFILE_PORTRAIT_LOWER_OUTSIDE_RADIUS_CLASS_BY_SIDE[outsideSide],
          )}
        />
      )}
      {uploadAction && media && (
        <button
          type="button"
          onClick={(event) => {
            event.stopPropagation();
            uploadAction.onClick();
          }}
          title={uploadAction.title}
          aria-label={uploadAction.ariaLabel}
          className={PORTRAIT_UPLOAD_BUTTON_CLASS}
        >
          <ImagePlus size="0.6875rem" />
        </button>
      )}
      <span className="sr-only">{accessibleLabel}</span>
    </div>
  );
}
