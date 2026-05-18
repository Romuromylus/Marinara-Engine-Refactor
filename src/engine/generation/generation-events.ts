export type GenerationEvent =
  | { type: "phase"; data: string }
  | { type: "thinking"; data: string }
  | { type: "token"; data: string }
  | { type: "assistant_message"; data: unknown }
  | { type: "agent_result"; data: unknown }
  | { type: "done"; data?: unknown };
