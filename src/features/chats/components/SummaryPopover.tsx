// ──────────────────────────────────────────────
// Summary Popover — structured rolling summary editor.
// ──────────────────────────────────────────────
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { useQueryClient } from "@tanstack/react-query";
import {
  Check,
  CopyPlus,
  Eye,
  EyeOff,
  Loader2,
  Pencil,
  Plus,
  Save,
  ScrollText,
  Sparkles,
  Trash2,
  X,
} from "lucide-react";
import { cn } from "../../../shared/lib/utils";
import { useUIStore } from "../../../shared/stores/ui.store";
import { storageApi } from "../../../shared/api/storage-api";
import {
  compileChatSummaryEntries,
  createChatSummaryEntry,
  ensureSummaryEntryMetadata,
  estimateChatSummaryTokens,
  normalizeChatSummaryEntries,
  normalizeChatSummaryPromptTemplates,
} from "../../../engine/shared/text/chat-summary-entries";
import type { ChatSummaryEntry, ChatSummaryPromptTemplate } from "../../../engine/contracts/types/chat";
import {
  useGenerateSummary,
  chatKeys,
  useUpdateRollingChatSummary,
} from "../../chats/hooks/use-chats";

const MAX_SUMMARY_MESSAGES = 200;
const DEFAULT_SUMMARY_PROMPT =
  "Summarize the provided chat transcript for future roleplay/conversation context. Preserve durable facts, relationships, goals, decisions, unresolved threads, and emotional state. Do not add new events.";

type SourceMode = "last" | "range";

interface SummaryPopoverProps {
  chatId: string;
  summary: string | null;
  summaryEntries?: ChatSummaryEntry[];
  promptTemplates?: ChatSummaryPromptTemplate[];
  activePromptTemplateId?: string | null;
  contextSize: number;
  totalMessageCount?: number;
  onContextSizeChange: (size: number) => void;
  onClose: () => void;
}

function clampMessageCount(value: number) {
  return Math.max(5, Math.min(MAX_SUMMARY_MESSAGES, Math.trunc(value)));
}

function newId(prefix: string) {
  return globalThis.crypto?.randomUUID ? `${prefix}-${globalThis.crypto.randomUUID()}` : `${prefix}-${Date.now()}`;
}

function formatTokens(tokens: number) {
  return tokens >= 1000 ? `${(tokens / 1000).toFixed(1)}k` : tokens.toLocaleString();
}

function templateLabel(template: ChatSummaryPromptTemplate | null) {
  return template ? template.name : "Default";
}

export function SummaryPopover({
  chatId,
  summary,
  summaryEntries,
  promptTemplates,
  activePromptTemplateId,
  contextSize,
  totalMessageCount = 0,
  onContextSizeChange,
  onClose,
}: SummaryPopoverProps) {
  const normalizedTemplates = useMemo(() => normalizeChatSummaryPromptTemplates(promptTemplates), [promptTemplates]);
  const normalizedEntries = useMemo(() => {
    const ensured = ensureSummaryEntryMetadata({ summary, summaryEntries });
    return normalizeChatSummaryEntries(ensured.entries);
  }, [summary, summaryEntries]);

  const [entries, setEntries] = useState<ChatSummaryEntry[]>(normalizedEntries);
  const [templates, setTemplates] = useState<ChatSummaryPromptTemplate[]>(normalizedTemplates);
  const [activeTemplateId, setActiveTemplateId] = useState(activePromptTemplateId ?? "");
  const [sourceMode, setSourceMode] = useState<SourceMode>("last");
  const [localSize, setLocalSize] = useState(String(contextSize || 50));
  const [rangeStart, setRangeStart] = useState("1");
  const [rangeEnd, setRangeEnd] = useState(String(Math.max(1, totalMessageCount)));
  const [draftContent, setDraftContent] = useState("");
  const [draftTitle, setDraftTitle] = useState("Manual summary");
  const [editingEntryId, setEditingEntryId] = useState<string | null>(null);
  const [editingTemplateId, setEditingTemplateId] = useState<string | null>(null);
  const [templateName, setTemplateName] = useState("");
  const [templatePrompt, setTemplatePrompt] = useState(DEFAULT_SUMMARY_PROMPT);
  const [showInactive, setShowInactive] = useState(true);
  const [hideSummarizedMessages, setHideSummarizedMessages] = useState(false);
  const sizeInputFocused = useRef(false);
  const panelRef = useRef<HTMLDivElement>(null);
  const updateSummary = useUpdateRollingChatSummary();
  const generateSummary = useGenerateSummary();
  const queryClient = useQueryClient();
  const collapseHiddenAiMessages = useUIStore((state) => state.collapseHiddenAiMessages);
  const setCollapseHiddenAiMessages = useUIStore((state) => state.setCollapseHiddenAiMessages);

  useEffect(() => setEntries(normalizedEntries), [normalizedEntries]);
  useEffect(() => setTemplates(normalizedTemplates), [normalizedTemplates]);
  useEffect(() => setActiveTemplateId(activePromptTemplateId ?? ""), [activePromptTemplateId]);
  useEffect(() => {
    if (!sizeInputFocused.current) setLocalSize(String(contextSize || 50));
  }, [contextSize]);
  useEffect(() => {
    setRangeEnd(String(Math.max(1, totalMessageCount)));
  }, [totalMessageCount]);

  useEffect(() => {
    const handler = (event: MouseEvent) => {
      if (panelRef.current && !panelRef.current.contains(event.target as Node)) onClose();
    };
    const raf = requestAnimationFrame(() => document.addEventListener("mousedown", handler));
    return () => {
      cancelAnimationFrame(raf);
      document.removeEventListener("mousedown", handler);
    };
  }, [onClose]);

  useEffect(() => {
    const handler = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    document.addEventListener("keydown", handler);
    return () => document.removeEventListener("keydown", handler);
  }, [onClose]);

  const enabledTokens = entries.reduce((total, entry) => (entry.enabled ? total + entry.tokenEstimate : total), 0);
  const visibleEntries = showInactive ? entries : entries.filter((entry) => entry.enabled);
  const selectedTemplate = templates.find((template) => template.id === activeTemplateId) ?? null;
  const rangeLow = Math.max(1, Math.trunc(Number(rangeStart) || 1));
  const rangeHigh = Math.max(rangeLow, Math.trunc(Number(rangeEnd) || rangeLow));
  const selectedRangeCount = rangeHigh - rangeLow + 1;
  const rangeTooLarge = sourceMode === "range" && selectedRangeCount > MAX_SUMMARY_MESSAGES;

  const persistEntries = useCallback(
    async (nextEntries: ChatSummaryEntry[]) => {
      const normalized = normalizeChatSummaryEntries(nextEntries);
      setEntries(normalized);
      await updateSummary.mutateAsync({
        id: chatId,
        summaryEntries: normalized,
        summary: compileChatSummaryEntries(normalized),
      });
    },
    [chatId, updateSummary],
  );

  const persistTemplates = useCallback(
    async (nextTemplates: ChatSummaryPromptTemplate[], nextActiveId = activeTemplateId) => {
      const normalized = normalizeChatSummaryPromptTemplates(nextTemplates);
      const activeId = normalized.some((template) => template.id === nextActiveId) ? nextActiveId : "";
      setTemplates(normalized);
      setActiveTemplateId(activeId);
      await updateSummary.mutateAsync({
        id: chatId,
        summaryPromptTemplates: normalized,
        activeSummaryPromptTemplateId: activeId || null,
      });
    },
    [activeTemplateId, chatId, updateSummary],
  );

  const handleGenerate = useCallback(() => {
    const size = clampMessageCount(Number(localSize) || contextSize || 50);
    setLocalSize(String(size));
    onContextSizeChange(size);
    generateSummary.mutate(
      sourceMode === "range"
        ? {
            chatId,
            sourceMode,
            rangeStartIndex: rangeLow,
            rangeEndIndex: rangeHigh,
            promptTemplateId: activeTemplateId || null,
          }
        : { chatId, contextSize: size, sourceMode, promptTemplateId: activeTemplateId || null },
      {
        onSuccess: async (result) => {
          if (!hideSummarizedMessages) return;
          await Promise.all(result.messageIds.map((messageId) => storageApi.patchChatMessageExtra(messageId, { hiddenFromAI: true })));
          await queryClient.invalidateQueries({ queryKey: chatKeys.messages(chatId) });
        },
      },
    );
  }, [
    activeTemplateId,
    chatId,
    contextSize,
    generateSummary,
    hideSummarizedMessages,
    localSize,
    onContextSizeChange,
    queryClient,
    rangeHigh,
    rangeLow,
    sourceMode,
  ]);

  const handleSaveDraft = useCallback(async () => {
    const content = draftContent.trim();
    if (!content) return;
    const now = new Date().toISOString();
    const next =
      editingEntryId === null
        ? [
            ...entries,
            createChatSummaryEntry(
              {
                id: newId("summary"),
                content,
                title: draftTitle.trim() || "Manual summary",
                origin: "manual",
                sourceMode: "last",
                tokenEstimate: estimateChatSummaryTokens(content),
              },
              { now },
            ),
          ]
        : entries.map((entry) =>
            entry.id === editingEntryId
              ? {
                  ...entry,
                  title: draftTitle.trim() || entry.title,
                  content,
                  tokenEstimate: estimateChatSummaryTokens(content),
                  updatedAt: now,
                }
              : entry,
          );
    await persistEntries(next);
    setEditingEntryId(null);
    setDraftTitle("Manual summary");
    setDraftContent("");
  }, [draftContent, draftTitle, editingEntryId, entries, persistEntries]);

  const isMobile = typeof window !== "undefined" && window.innerWidth < 768;
  const isSaving = updateSummary.isPending;
  const isGenerating = generateSummary.isPending;

  const content = (
    <div
      ref={panelRef}
      onMouseDown={(event) => event.stopPropagation()}
      className={cn(
        isMobile
          ? "fixed inset-0 z-[9999] flex items-center justify-center p-3 max-md:pt-[max(0.75rem,env(safe-area-inset-top))]"
          : "absolute right-0 top-full z-[100] mt-1",
      )}
    >
      {isMobile && <div className="absolute inset-0 bg-black/30" onClick={onClose} />}
      <div
        className={cn(
          "relative overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--card)] shadow-2xl shadow-black/40",
          isMobile ? "h-[calc(100dvh-1.5rem)] w-full max-w-md" : "w-[28rem] max-w-[calc(100vw-2rem)]",
        )}
      >
        <div className="flex items-center justify-between border-b border-[var(--border)] px-3 py-2">
          <div className="flex items-center gap-2 text-xs font-semibold">
            <ScrollText size="0.875rem" className="text-amber-400" />
            <span>Chat Summary</span>
            <span className="rounded bg-[var(--secondary)] px-1.5 py-0.5 text-[0.625rem] font-normal text-[var(--muted-foreground)]">
              {entries.filter((entry) => entry.enabled).length} active · ~{formatTokens(enabledTokens)} tokens
            </span>
          </div>
          <button
            onClick={onClose}
            className="rounded-md p-1 text-[var(--muted-foreground)] hover:bg-[var(--accent)] hover:text-[var(--foreground)]"
            title="Close"
          >
            <X size="0.8125rem" />
          </button>
        </div>

        <div className={cn("overflow-y-auto p-3", isMobile ? "h-[calc(100%-2.5rem)]" : "max-h-[36rem]")}>
          <div className="space-y-3">
            <div className="rounded-lg bg-[var(--secondary)]/70 p-2.5 ring-1 ring-[var(--border)]">
              <div className="flex flex-wrap items-center gap-2">
                <div className="flex rounded-lg bg-[var(--card)] p-0.5 ring-1 ring-[var(--border)]">
                  {(["last", "range"] as const).map((mode) => (
                    <button
                      key={mode}
                      onClick={() => setSourceMode(mode)}
                      className={cn(
                        "rounded-md px-2 py-1 text-[0.625rem] font-medium transition-colors",
                        sourceMode === mode
                          ? "bg-amber-400 text-black"
                          : "text-[var(--muted-foreground)] hover:text-[var(--foreground)]",
                      )}
                    >
                      {mode === "last" ? "Last" : "Range"}
                    </button>
                  ))}
                </div>
                {sourceMode === "last" ? (
                  <input
                    type="number"
                    min={5}
                    max={MAX_SUMMARY_MESSAGES}
                    value={localSize}
                    onFocus={() => {
                      sizeInputFocused.current = true;
                    }}
                    onChange={(event) => setLocalSize(event.target.value)}
                    onBlur={() => {
                      sizeInputFocused.current = false;
                      const next = clampMessageCount(Number(localSize) || 50);
                      setLocalSize(String(next));
                      onContextSizeChange(next);
                    }}
                    className="w-16 rounded-md bg-[var(--card)] px-2 py-1 text-center text-[0.6875rem] tabular-nums ring-1 ring-[var(--border)] focus:outline-none focus:ring-2 focus:ring-[var(--ring)]"
                    title="Recent message count"
                  />
                ) : (
                  <div className="flex items-center gap-1">
                    <input
                      type="number"
                      min={1}
                      value={rangeStart}
                      onChange={(event) => setRangeStart(event.target.value)}
                      className="w-16 rounded-md bg-[var(--card)] px-2 py-1 text-center text-[0.6875rem] tabular-nums ring-1 ring-[var(--border)] focus:outline-none focus:ring-2 focus:ring-[var(--ring)]"
                      title="Start message index"
                    />
                    <span className="text-[0.625rem] text-[var(--muted-foreground)]">to</span>
                    <input
                      type="number"
                      min={1}
                      value={rangeEnd}
                      onChange={(event) => setRangeEnd(event.target.value)}
                      className="w-16 rounded-md bg-[var(--card)] px-2 py-1 text-center text-[0.6875rem] tabular-nums ring-1 ring-[var(--border)] focus:outline-none focus:ring-2 focus:ring-[var(--ring)]"
                      title="End message index"
                    />
                  </div>
                )}
                <select
                  value={activeTemplateId}
                  onChange={(event) => {
                    const next = event.target.value;
                    setActiveTemplateId(next);
                    void updateSummary.mutateAsync({
                      id: chatId,
                      activeSummaryPromptTemplateId: next || null,
                    });
                  }}
                  className="min-w-0 flex-1 rounded-md bg-[var(--card)] px-2 py-1 text-[0.6875rem] ring-1 ring-[var(--border)] focus:outline-none focus:ring-2 focus:ring-[var(--ring)]"
                  title="Prompt template"
                >
                  <option value="">Default</option>
                  {templates.map((template) => (
                    <option key={template.id} value={template.id}>
                      {template.name}
                    </option>
                  ))}
                </select>
                <button
                  onClick={handleGenerate}
                  disabled={isGenerating || rangeTooLarge}
                  className={cn(
                    "flex items-center gap-1 rounded-lg px-2 py-1 text-[0.6875rem] font-medium transition-all",
                    isGenerating || rangeTooLarge
                      ? "cursor-not-allowed text-amber-300/50"
                      : "text-amber-300 hover:bg-amber-400/15 hover:text-amber-200",
                  )}
                  title="Generate summary with AI"
                >
                  {isGenerating ? <Loader2 size="0.75rem" className="animate-spin" /> : <Sparkles size="0.75rem" />}
                  Generate
                </button>
              </div>
              {rangeTooLarge && (
                <p className="mt-1.5 text-[0.625rem] text-amber-300">
                  Selected range has {selectedRangeCount} messages. Summary ranges are limited to {MAX_SUMMARY_MESSAGES}.
                </p>
              )}
              <label className="mt-2 flex items-center gap-2 text-[0.6875rem] text-[var(--muted-foreground)]">
                <input
                  type="checkbox"
                  checked={hideSummarizedMessages}
                  onChange={(event) => setHideSummarizedMessages(event.target.checked)}
                  className="h-3.5 w-3.5 accent-amber-400"
                />
                Hide summarized messages from AI
              </label>
              <label className="mt-1.5 flex items-center gap-2 text-[0.6875rem] text-[var(--muted-foreground)]">
                <input
                  type="checkbox"
                  checked={collapseHiddenAiMessages}
                  onChange={(event) => setCollapseHiddenAiMessages(event.target.checked)}
                  className="h-3.5 w-3.5 accent-amber-400"
                />
                Collapse messages hidden from AI
              </label>
            </div>

            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-[0.6875rem] font-semibold text-[var(--muted-foreground)]">
                  Rolling summary entries
                </span>
                <div className="flex items-center gap-1">
                  <button
                    onClick={() => setShowInactive((value) => !value)}
                    className="rounded-md p-1 text-[var(--muted-foreground)] hover:bg-[var(--accent)] hover:text-[var(--foreground)]"
                    title={showInactive ? "Hide inactive entries" : "Show inactive entries"}
                  >
                    {showInactive ? <Eye size="0.75rem" /> : <EyeOff size="0.75rem" />}
                  </button>
                  <button
                    onClick={() => {
                      setEditingEntryId(null);
                      setDraftTitle("Manual summary");
                      setDraftContent("");
                    }}
                    className="rounded-md p-1 text-[var(--muted-foreground)] hover:bg-[var(--accent)] hover:text-[var(--foreground)]"
                    title="Write summary entry"
                  >
                    <Plus size="0.75rem" />
                  </button>
                </div>
              </div>

              {(editingEntryId !== null || draftContent || entries.length === 0) && (
                <div className="rounded-lg bg-[var(--secondary)]/70 p-2 ring-1 ring-[var(--border)]">
                  <input
                    value={draftTitle}
                    onChange={(event) => setDraftTitle(event.target.value)}
                    className="mb-2 w-full rounded-md bg-[var(--card)] px-2 py-1 text-xs font-medium ring-1 ring-[var(--border)] focus:outline-none focus:ring-2 focus:ring-[var(--ring)]"
                    placeholder="Summary title"
                  />
                  <textarea
                    value={draftContent}
                    onChange={(event) => setDraftContent(event.target.value)}
                    rows={5}
                    className="w-full resize-y rounded-md bg-[var(--card)] px-2 py-1.5 text-xs leading-relaxed ring-1 ring-[var(--border)] focus:outline-none focus:ring-2 focus:ring-[var(--ring)]"
                    placeholder="Write durable facts, relationship changes, plans, and unresolved threads."
                  />
                  <div className="mt-2 flex justify-end gap-1.5">
                    <button
                      onClick={() => {
                        setEditingEntryId(null);
                        setDraftTitle("Manual summary");
                        setDraftContent("");
                      }}
                      className="rounded-lg px-2 py-1 text-[0.625rem] font-medium text-[var(--muted-foreground)] hover:bg-[var(--accent)]"
                    >
                      Cancel
                    </button>
                    <button
                      onClick={() => void handleSaveDraft()}
                      disabled={!draftContent.trim() || isSaving}
                      className="flex items-center gap-1 rounded-lg bg-amber-400 px-2 py-1 text-[0.625rem] font-semibold text-black disabled:opacity-50"
                    >
                      <Save size="0.6875rem" />
                      Save
                    </button>
                  </div>
                </div>
              )}

              <div className="space-y-1.5">
                {visibleEntries.length === 0 ? (
                  <div className="rounded-lg border border-dashed border-[var(--border)] px-3 py-5 text-center text-xs text-[var(--muted-foreground)]">
                    No summary entries yet.
                  </div>
                ) : (
                  visibleEntries.map((entry) => (
                    <div
                      key={entry.id}
                      className={cn(
                        "rounded-lg bg-[var(--secondary)]/50 p-2 ring-1 ring-[var(--border)]",
                        !entry.enabled && "opacity-60",
                      )}
                    >
                      <div className="flex items-start gap-2">
                        <button
                          onClick={() =>
                            void persistEntries(entries.map((item) => (item.id === entry.id ? { ...item, enabled: !item.enabled } : item)))
                          }
                          className={cn(
                            "mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center rounded-md ring-1 ring-[var(--border)]",
                            entry.enabled ? "bg-amber-400 text-black" : "text-transparent",
                          )}
                          title={entry.enabled ? "Disable summary" : "Enable summary"}
                        >
                          <Check size="0.75rem" />
                        </button>
                        <div className="min-w-0 flex-1">
                          <div className="flex items-center gap-2">
                            <span className="min-w-0 truncate text-xs font-semibold">{entry.title}</span>
                            <span className="shrink-0 text-[0.625rem] text-[var(--muted-foreground)]">
                              ~{formatTokens(entry.tokenEstimate)}
                            </span>
                          </div>
                          <p className="mt-1 line-clamp-3 whitespace-pre-wrap text-[0.6875rem] leading-relaxed text-[var(--foreground)]/80">
                            {entry.content}
                          </p>
                        </div>
                        <button
                          onClick={() => {
                            setEditingEntryId(entry.id);
                            setDraftTitle(entry.title);
                            setDraftContent(entry.content);
                          }}
                          className="rounded-md p-1 text-[var(--muted-foreground)] hover:bg-[var(--accent)] hover:text-[var(--foreground)]"
                          title="Edit"
                        >
                          <Pencil size="0.75rem" />
                        </button>
                        <button
                          onClick={() => void persistEntries(entries.filter((item) => item.id !== entry.id))}
                          className="rounded-md p-1 text-[var(--muted-foreground)] hover:bg-[var(--destructive)]/15 hover:text-[var(--destructive)]"
                          title="Delete"
                        >
                          <Trash2 size="0.75rem" />
                        </button>
                      </div>
                    </div>
                  ))
                )}
              </div>
            </div>

            <div className="space-y-2 border-t border-[var(--border)] pt-3">
              <div className="flex items-center justify-between">
                <span className="text-[0.6875rem] font-semibold text-[var(--muted-foreground)]">
                  Prompt templates
                </span>
                <button
                  onClick={() => {
                    setEditingTemplateId(null);
                    setTemplateName("Custom summary");
                    setTemplatePrompt(DEFAULT_SUMMARY_PROMPT);
                  }}
                  className="rounded-md p-1 text-[var(--muted-foreground)] hover:bg-[var(--accent)] hover:text-[var(--foreground)]"
                  title="New template"
                >
                  <Plus size="0.75rem" />
                </button>
              </div>
              {(editingTemplateId !== null || templateName !== "" || templates.length === 0) && (
                <div className="rounded-lg bg-[var(--secondary)]/70 p-2 ring-1 ring-[var(--border)]">
                  <input
                    value={templateName}
                    onChange={(event) => setTemplateName(event.target.value)}
                    className="mb-2 w-full rounded-md bg-[var(--card)] px-2 py-1 text-xs font-medium ring-1 ring-[var(--border)] focus:outline-none focus:ring-2 focus:ring-[var(--ring)]"
                    placeholder="Template name"
                  />
                  <textarea
                    value={templatePrompt}
                    onChange={(event) => setTemplatePrompt(event.target.value)}
                    rows={4}
                    className="w-full resize-y rounded-md bg-[var(--card)] px-2 py-1.5 text-xs leading-relaxed ring-1 ring-[var(--border)] focus:outline-none focus:ring-2 focus:ring-[var(--ring)]"
                    placeholder={DEFAULT_SUMMARY_PROMPT}
                  />
                  <div className="mt-2 flex justify-end gap-1.5">
                    <button
                      onClick={() => {
                        setEditingTemplateId(null);
                        setTemplateName("");
                        setTemplatePrompt(DEFAULT_SUMMARY_PROMPT);
                      }}
                      className="rounded-lg px-2 py-1 text-[0.625rem] font-medium text-[var(--muted-foreground)] hover:bg-[var(--accent)]"
                    >
                      Cancel
                    </button>
                    <button
                      onClick={() => {
                        const name = templateName.trim();
                        const prompt = templatePrompt.trim();
                        if (!name || !prompt) return;
                        const id = editingTemplateId ?? newId("summary-template");
                        const next = editingTemplateId
                          ? templates.map((template) => (template.id === editingTemplateId ? { id, name, prompt } : template))
                          : [...templates, { id, name, prompt }];
                        void persistTemplates(next, activeTemplateId || id);
                        setEditingTemplateId(null);
                        setTemplateName("");
                        setTemplatePrompt(DEFAULT_SUMMARY_PROMPT);
                      }}
                      disabled={!templateName.trim() || !templatePrompt.trim() || isSaving}
                      className="flex items-center gap-1 rounded-lg bg-amber-400 px-2 py-1 text-[0.625rem] font-semibold text-black disabled:opacity-50"
                    >
                      <Save size="0.6875rem" />
                      Save
                    </button>
                  </div>
                </div>
              )}
              <div className="space-y-1">
                <div className="rounded-md bg-[var(--secondary)]/40 px-2 py-1.5 text-[0.6875rem] text-[var(--muted-foreground)]">
                  Active: {templateLabel(selectedTemplate)}
                </div>
                {templates.map((template) => (
                  <div key={template.id} className="flex items-center gap-2 rounded-md bg-[var(--secondary)]/50 px-2 py-1">
                    <span className="min-w-0 flex-1 truncate text-xs">{template.name}</span>
                    <button
                      onClick={() => {
                        setEditingTemplateId(null);
                        setTemplateName(`${template.name} copy`);
                        setTemplatePrompt(template.prompt);
                      }}
                      className="rounded p-1 text-[var(--muted-foreground)] hover:bg-[var(--accent)] hover:text-[var(--foreground)]"
                      title="Duplicate"
                    >
                      <CopyPlus size="0.6875rem" />
                    </button>
                    <button
                      onClick={() => {
                        setEditingTemplateId(template.id);
                        setTemplateName(template.name);
                        setTemplatePrompt(template.prompt);
                      }}
                      className="rounded p-1 text-[var(--muted-foreground)] hover:bg-[var(--accent)] hover:text-[var(--foreground)]"
                      title="Edit"
                    >
                      <Pencil size="0.6875rem" />
                    </button>
                    <button
                      onClick={() => void persistTemplates(templates.filter((item) => item.id !== template.id))}
                      className="rounded p-1 text-[var(--muted-foreground)] hover:bg-[var(--destructive)]/15 hover:text-[var(--destructive)]"
                      title="Delete"
                    >
                      <Trash2 size="0.6875rem" />
                    </button>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );

  return isMobile ? createPortal(content, document.body) : content;
}
