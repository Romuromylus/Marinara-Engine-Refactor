import { CheckCircle2, Plus, Target, X } from "lucide-react";
import type { QuestProgress } from "../../../../engine/contracts/types/game-state";
import { cn } from "../../../../shared/lib/utils";
import {
  addQuestObjective,
  removeQuestObjective,
  toggleQuestObjectiveCompletion,
  updateQuestObjectiveText,
} from "../../../world-state/lib/tracker-state-edits";
import { TRACKER_BAR } from "../tracker-data-sidebar.constants";
import { InlineEdit } from "../tracker-data-sidebar.controls";
import { visibleText } from "../tracker-display.helpers";
import { QuestObjectiveRow } from "./QuestObjectiveRow";

export function QuestRow({
  quest,
  onUpdate,
  onRemove,
  deleteMode = false,
  addMode = false,
}: {
  quest: QuestProgress;
  onUpdate?: (quest: QuestProgress) => void;
  onRemove?: () => void;
  deleteMode?: boolean;
  addMode?: boolean;
}) {
  const completed = quest.objectives.filter((objective) => objective.completed).length;
  const totalObjectives = quest.objectives.length;
  const completionPercent = quest.completed ? 100 : totalObjectives > 0 ? (completed / totalObjectives) * 100 : 0;
  const completionLabel = totalObjectives > 0 ? `${completed}/${totalObjectives}` : quest.completed ? "done" : "open";
  const objectiveGridColumns = deleteMode
    ? "grid-cols-[0.875rem_minmax(0,1fr)_1rem]"
    : "grid-cols-[0.875rem_minmax(0,1fr)]";
  const questTitle = visibleText(quest.name, "Quest");
  const updateObjective = (index: number, nextText: string) => {
    if (!onUpdate) return;
    onUpdate(updateQuestObjectiveText(quest, index, nextText));
  };
  const toggleObjective = (index: number) => {
    if (!onUpdate) return;
    onUpdate(toggleQuestObjectiveCompletion(quest, index));
  };
  const removeObjective = (index: number) => {
    if (!onUpdate) return;
    onUpdate(removeQuestObjective(quest, index));
  };
  const addObjective = () => {
    if (!onUpdate) return;
    onUpdate(addQuestObjective(quest));
  };
  return (
    <article
      className={cn(
        "group/quest relative mx-1 overflow-hidden rounded-sm border border-[var(--border)]/30 bg-[color-mix(in_srgb,var(--background)_22%,transparent)] shadow-[inset_0_1px_0_color-mix(in_srgb,var(--foreground)_5%,transparent)]",
        quest.completed && "opacity-75",
      )}
    >
      <div className="pointer-events-none absolute inset-x-0 top-0 h-px bg-[var(--primary)]/16" />
      <div
        className={cn(
          "relative grid min-h-5 grid-cols-[1rem_minmax(0,1fr)_auto] items-center gap-1 px-1 py-0.5",
          deleteMode && "grid-cols-[1rem_minmax(0,1fr)_auto_1rem]",
        )}
      >
        {onUpdate && (
          <button
            type="button"
            onClick={() => onUpdate({ ...quest, completed: !quest.completed })}
            className={cn(
              "flex h-4 w-4 shrink-0 items-center justify-center rounded-sm text-[var(--muted-foreground)] transition-colors hover:bg-[var(--primary)]/10 hover:text-emerald-300",
              quest.completed && "text-emerald-300",
            )}
            title={quest.completed ? "Mark incomplete" : "Mark complete"}
            aria-label={quest.completed ? "Mark quest incomplete" : "Mark quest complete"}
          >
            {quest.completed ? <CheckCircle2 size="0.75rem" /> : <Target size="0.75rem" />}
          </button>
        )}
        {!onUpdate && (
          <span className="flex h-4 w-4 shrink-0 items-center justify-center text-[var(--muted-foreground)]">
            {quest.completed ? <CheckCircle2 size="0.75rem" /> : <Target size="0.75rem" />}
          </span>
        )}
        {onUpdate ? (
          <InlineEdit
            value={quest.name}
            onSave={(name) => onUpdate({ ...quest, name: name || "Quest" })}
            placeholder="Quest"
            title={`Quest: ${questTitle}`}
            showEditHint={false}
            className={cn(
              "h-5 w-full min-w-0 overflow-hidden px-0.5 py-0 text-[0.75rem] font-semibold leading-5 text-[var(--foreground)]/92 hover:bg-[var(--accent)]/20",
              quest.completed && "line-through opacity-60",
            )}
          />
        ) : (
          <div
            className={cn(
              "min-w-0 truncate text-[0.75rem] font-semibold",
              quest.completed && "text-[var(--muted-foreground)] line-through",
            )}
          >
            {questTitle}
          </div>
        )}
        <span className="shrink-0 rounded-sm border border-[var(--border)]/32 bg-[var(--background)]/18 px-1 py-0.5 text-[0.5625rem] font-semibold uppercase leading-none tabular-nums text-[var(--foreground)]/68">
          {completionLabel}
        </span>
        {onRemove && deleteMode && (
          <button
            type="button"
            onClick={onRemove}
            className="flex h-4 w-4 items-center justify-center rounded-sm text-[var(--destructive)] transition-all hover:bg-[var(--accent)] focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-[var(--primary)] active:scale-90"
            title="Remove quest"
            aria-label="Remove quest"
          >
            <X size="0.625rem" />
          </button>
        )}
      </div>

      <div className={cn("relative mx-1 overflow-hidden bg-[var(--border)]/28", TRACKER_BAR)}>
        <div
          className={cn(
            "h-full rounded-[1px] transition-[width] duration-200",
            quest.completed ? "bg-emerald-300/85" : "bg-[var(--primary)]/85",
          )}
          style={{ width: `${completionPercent}%` }}
        />
      </div>

      {(quest.objectives.length > 0 || (onUpdate && addMode)) && (
        <div className="relative mx-1 mb-0.5 mt-0.5 grid gap-px pl-4">
          <span
            className={cn(
              "pointer-events-none absolute left-[0.4375rem] top-1 w-px bg-[var(--border)]/28",
              addMode ? "bottom-4" : "bottom-1",
            )}
          />
          {quest.objectives.map((objective, index) => (
            <QuestObjectiveRow
              key={`${objective.text}-${index}`}
              objective={objective}
              objectiveGridColumns={objectiveGridColumns}
              onToggle={onUpdate ? () => toggleObjective(index) : undefined}
              onUpdate={onUpdate ? (text) => updateObjective(index, text) : undefined}
              onRemove={onUpdate ? () => removeObjective(index) : undefined}
              deleteMode={deleteMode}
            />
          ))}
          {onUpdate && addMode && (
            <button
              type="button"
              onClick={addObjective}
              className="relative grid h-4 w-full grid-cols-[0.875rem_minmax(0,1fr)] items-center gap-1 rounded-[2px] px-0.5 text-left text-[0.6875rem] leading-4 text-[var(--foreground)]/35 transition-colors hover:bg-[var(--primary)]/10 hover:text-[var(--primary)] focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-[var(--primary)]"
              title="Add objective"
              aria-label="Add objective"
            >
              <Plus size="0.625rem" className="justify-self-center" />
              <span className="truncate font-medium">Objective</span>
            </button>
          )}
        </div>
      )}
    </article>
  );
}
