export type EngineEventHandler<T = unknown> = (payload: T) => void;

export interface EventGateway {
  emit<T = unknown>(event: string, payload: T): Promise<void>;
  listen<T = unknown>(event: string, handler: EngineEventHandler<T>): Promise<() => void>;
}
