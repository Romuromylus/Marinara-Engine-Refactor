import { useEffect, useState } from "react";
import { Loader2, Palette, RotateCcw, Save } from "lucide-react";
import { toast } from "sonner";
import { useTrackerCardColorSettingsTargets } from "../../../tracker/hooks/use-tracker-card-color-settings";
import { TrackerCardColorControls } from "./TrackerCardColorControls";
import {
  cleanTrackerCardColorConfig,
  serializeTrackerCardColorConfig,
} from "../../../../shared/lib/tracker-card-colors";
import { cn } from "../../../../shared/lib/utils";
import type { TrackerCardColorConfig } from "../../../../engine/contracts/types/persona";

function colorConfigKey(value: TrackerCardColorConfig) {
  return serializeTrackerCardColorConfig(cleanTrackerCardColorConfig(value));
}

export function TrackerCardColorSettings() {
  const { activeChatId, targets, loading, saving, saveTrackerCardColorTarget } =
    useTrackerCardColorSettingsTargets();
  const [selectedKey, setSelectedKey] = useState("");
  const [draftConfig, setDraftConfig] = useState<TrackerCardColorConfig | null>(null);

  const selectedTarget = targets.find((target) => target.key === selectedKey) ?? targets[0] ?? null;
  const savedConfigKey = selectedTarget ? colorConfigKey(selectedTarget.trackerCardColors) : "";
  const draftConfigKey = draftConfig ? colorConfigKey(draftConfig) : "";
  const dirty = !!selectedTarget && !!draftConfig && draftConfigKey !== savedConfigKey;

  useEffect(() => {
    if (!targets.length) {
      setSelectedKey("");
      setDraftConfig(null);
      return;
    }
    if (!targets.some((target) => target.key === selectedKey)) {
      setSelectedKey(targets[0]!.key);
    }
  }, [selectedKey, targets]);

  useEffect(() => {
    if (!selectedTarget) {
      setDraftConfig(null);
      return;
    }
    setDraftConfig(cleanTrackerCardColorConfig(selectedTarget.trackerCardColors));
  }, [selectedTarget?.key, savedConfigKey]);

  const save = () => {
    if (!selectedTarget || !draftConfig || !dirty || saving) return;
    const cleaned = cleanTrackerCardColorConfig(draftConfig);

    void saveTrackerCardColorTarget(selectedTarget.key, cleaned)
      .then(() => {
        toast.success(
          selectedTarget.kind === "persona"
            ? "Persona tracker card colors saved."
            : "Character tracker card colors saved.",
        );
      })
      .catch((error) => {
        if (
          selectedTarget.kind === "character" &&
          error instanceof Error &&
          error.message.includes("could not be parsed")
        ) {
          toast.error("Could not save tracker card colors because the character data is invalid.");
          return;
        }
        toast.error(
          selectedTarget.kind === "persona"
            ? "Failed to save persona tracker card colors."
            : "Failed to save character tracker card colors.",
        );
      });
  };

  return (
    <div className="mt-1.5 rounded-lg bg-[var(--background)]/36 p-1.5 ring-1 ring-[var(--border)]">
      <div className="flex min-h-7 items-center justify-between gap-2 px-0.5">
        <span className="inline-flex min-w-0 items-center gap-1 text-[0.625rem] font-medium text-[var(--foreground)]">
          <Palette size="0.6875rem" />
          Tracker card colors
        </span>
        {dirty && <span className="text-[0.5625rem] font-medium text-[var(--primary)]">Unsaved</span>}
      </div>

      {!activeChatId ? (
        <p className="rounded-md bg-[var(--secondary)]/35 px-2 py-2 text-[0.625rem] text-[var(--muted-foreground)]">
          Open a chat to edit tracker cards for its active persona and characters.
        </p>
      ) : loading ? (
        <div className="flex items-center gap-2 rounded-md bg-[var(--secondary)]/35 px-2 py-2 text-[0.625rem] text-[var(--muted-foreground)]">
          <Loader2 size="0.6875rem" className="animate-spin" />
          Loading tracker color targets...
        </div>
      ) : !selectedTarget || !draftConfig ? (
        <p className="rounded-md bg-[var(--secondary)]/35 px-2 py-2 text-[0.625rem] text-[var(--muted-foreground)]">
          This chat has no active tracker card color targets yet.
        </p>
      ) : (
        <div className="grid gap-2">
          <div className="grid gap-1.5 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-center">
            <select
              value={selectedTarget.key}
              onChange={(event) => setSelectedKey(event.target.value)}
              disabled={saving || dirty}
              className="min-w-0 rounded-md bg-[var(--secondary)] px-2 py-1.5 text-[0.6875rem] outline-none ring-1 ring-[var(--border)] focus:ring-[var(--primary)] disabled:opacity-60"
            >
              {targets.map((target) => (
                <option key={target.key} value={target.key}>
                  {target.kind === "persona" ? "Persona" : "Character"}: {target.name}
                </option>
              ))}
            </select>
            <div className="flex gap-1">
              <button
                type="button"
                onClick={() => setDraftConfig(cleanTrackerCardColorConfig(selectedTarget.trackerCardColors))}
                disabled={!dirty || saving}
                className="flex h-8 min-w-8 items-center justify-center rounded-md text-[var(--muted-foreground)] ring-1 ring-[var(--border)] transition-colors hover:bg-[var(--secondary)] hover:text-[var(--foreground)] disabled:cursor-default disabled:opacity-35"
                title="Revert tracker card color changes"
                aria-label="Revert tracker card color changes"
              >
                <RotateCcw size="0.75rem" />
              </button>
              <button
                type="button"
                onClick={save}
                disabled={!dirty || saving}
                className={cn(
                  "flex h-8 items-center justify-center gap-1.5 rounded-md px-2 text-[0.6875rem] font-semibold transition-colors disabled:cursor-default disabled:opacity-45",
                  dirty
                    ? "bg-[var(--primary)] text-[var(--primary-foreground)] hover:opacity-90"
                    : "bg-[var(--secondary)] text-[var(--muted-foreground)] ring-1 ring-[var(--border)]",
                )}
              >
                {saving ? <Loader2 size="0.75rem" className="animate-spin" /> : <Save size="0.75rem" />}
                Save
              </button>
            </div>
          </div>
          <TrackerCardColorControls
            value={draftConfig}
            onChange={setDraftConfig}
            chatColors={selectedTarget.chatColors}
            entityLabel={selectedTarget.kind === "persona" ? "Persona" : "Character"}
            previewName={selectedTarget.name}
          />
        </div>
      )}
    </div>
  );
}
