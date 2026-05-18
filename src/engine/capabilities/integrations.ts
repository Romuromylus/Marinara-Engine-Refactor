export interface IntegrationGateway {
  call<T = unknown>(integration: string, operation: string, payload?: unknown): Promise<T>;
}
