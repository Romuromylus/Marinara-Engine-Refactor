import { Modal } from "../Modal";
import { SpriteCleanupCanvasStage } from "./SpriteCleanupCanvasStage";
import { SpriteCleanupControls } from "./SpriteCleanupControls";
import { SpriteCleanupFooter } from "./SpriteCleanupFooter";
import { useSpriteCleanupEditor } from "./useSpriteCleanupEditor";

interface SpriteWandCleanupEditorProps {
  imageUrl: string;
  label: string;
  applying?: boolean;
  onApply: (cleanedDataUrl: string) => Promise<void> | void;
  onClose: () => void;
}

export function SpriteWandCleanupEditor({
  imageUrl,
  label,
  applying = false,
  onApply,
  onClose,
}: SpriteWandCleanupEditorProps) {
  const editor = useSpriteCleanupEditor({ imageUrl, applying, onApply });

  return (
    <Modal
      open
      onClose={onClose}
      title={`Clean ${label}`}
      width="!w-[calc(100vw_-_1.5rem)] min-w-0 max-w-[calc(100vw_-_1.5rem)] sm:!w-full sm:max-w-6xl"
    >
      <div className="flex h-[calc(100dvh-7rem)] w-full min-w-0 max-w-full flex-col gap-3 overflow-x-hidden overflow-y-auto sm:h-[min(44rem,calc(90dvh-6rem))]">
        <SpriteCleanupControls {...editor.controls} />

        <SpriteCleanupCanvasStage
          label={label}
          previewBackground={editor.controls.previewBackground}
          pickingBrushColor={editor.controls.pickingBrushColor}
          loading={editor.loading}
          cursorClass={editor.cursorClass}
          canvasDisplayStyle={editor.canvasDisplayStyle}
          reticleStyle={editor.reticleStyle}
          stageRef={editor.stageRef}
          canvasRef={editor.canvasRef}
          onStageWheel={editor.handleStageWheel}
          onCanvasPointerDown={editor.handleCanvasPointerDown}
          onCanvasPointerMove={editor.handleCanvasPointerMove}
          onCanvasPointerUp={editor.handleCanvasPointerUp}
          onCanvasPointerCancel={editor.handleCanvasPointerCancel}
          onCanvasPointerLeave={editor.handleCanvasPointerLeave}
        />

        <SpriteCleanupFooter
          applying={applying}
          loading={editor.loading}
          hasChanges={editor.hasChanges}
          canUndo={editor.canUndo}
          error={editor.error}
          status={editor.status}
          hoverReadout={editor.hoverReadout}
          onUndo={editor.handleUndo}
          onReset={editor.handleReset}
          onClose={onClose}
          onApply={() => void editor.handleApply()}
        />
      </div>
    </Modal>
  );
}
