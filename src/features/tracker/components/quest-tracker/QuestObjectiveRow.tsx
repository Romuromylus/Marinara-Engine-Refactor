import { CheckCircle2, Circle, X } from "lucide-react";
import type { QuestProgress } from "../../../../engine/contracts/types/game-state";
import { cn } from "../../../../shared/lib/utils";
import { InlineEdit } from "../tracker-data-sidebar.controls";
import { visibleText } from "../tracker-display.helpers";

type QuestObjective = QuestProgress["objectives"][number];

export function QuestObjectiveRow({
  objective,
  objectiveGridColumns,
  onRemove,
  onToggle,
  onUpdate,
  deleteMode,
}: {
  objective: QuestObjective;
  objectiveGridColumns: string;
  onRemove?: () => void;
  onToggle?: () => void;
  onUpdate?: (text: string) => void;
  deleteMode: boolean;
}) {
  return (
    <div
      className={cn(
        "relative grid min-h-4 items-center gap-1 rounded-[2px] px-0.5 text-[0.6875rem] leading-4 transition-colors hover:bg-[var(--accent)]/14",
        objectiveGridColumns,
      )}
    >
      {onToggle ? (
        <button
          type="button"
          onClick={onToggle}
          className={cn(
            "flex h-4 w-4 shrink-0 items-center justify-center rounded-sm text-[var(--muted-foreground)] transition-colors hover:bg-[var(--primary)]/10 hover:text-emerald-300",
            objective.completed && "text-emerald-300",
          )}
          title={objective.completed ? "Mark incomplete" : "Mark complete"}
          aria-label={objective.completed ? "Mark objective incomplete" : "Mark objective complete"}
        >
          {objective.completed ? <CheckCircle2 size="0.6875rem" /> : <Circle size="0.6875rem" />}
        </button>
      ) : objective.completed ? (
        <CheckCircle2 size="0.6875rem" className="shrink-0 text-emerald-300" />
      ) : (
        <Circle size="0.6875rem" className="shrink-0 text-[var(--muted-foreground)]" />
      )}
      {onUpdate ? (
        <InlineEdit
          value={objective.text}
          onSave={(text) => onUpdate(text || "Objective")}
          placeholder="Objective"
          title={`Objective: ${visibleText(objective.text, "Objective")}`}
          showEditHint={false}
          className={cn(
            "h-4 w-full min-w-0 overflow-hidden px-0.5 py-0 text-[0.6875rem] leading-4 hover:bg-[var(--accent)]/20",
            objective.completed && "line-through opacity-60",
          )}
        />
      ) : (
        <span
          className={cn(
            "min-w-0 truncate",
            objective.completed ? "text-[var(--muted-foreground)] line-through" : "text-[var(--foreground)]",
          )}
        >
          {visibleText(objective.text, "Objective")}
        </span>
      )}
      {onRemove && deleteMode && (
        <button
          type="button"
          onClick={onRemove}
          className="flex h-4 w-4 items-center justify-center rounded-sm text-[var(--destructive)] transition-all hover:bg-[var(--accent)] focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-[var(--primary)] active:scale-90"
          title="Remove objective"
          aria-label="Remove objective"
        >
          <X size="0.5rem" />
        </button>
      )}
    </div>
  );
}
