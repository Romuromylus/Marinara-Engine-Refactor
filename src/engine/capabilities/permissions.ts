export interface PermissionGateway {
  request(permission: string, context?: Record<string, unknown>): Promise<boolean>;
  check(permission: string, context?: Record<string, unknown>): Promise<boolean>;
}
