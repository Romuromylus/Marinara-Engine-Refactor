export interface ImportGateway {
  review(pathOrToken: string, options?: Record<string, unknown>): Promise<unknown>;
  import(pathOrToken: string, options?: Record<string, unknown>): Promise<unknown>;
  export(entity: string, id: string, options?: Record<string, unknown>): Promise<unknown>;
}
