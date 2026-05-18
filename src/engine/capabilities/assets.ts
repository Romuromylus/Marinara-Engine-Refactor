export interface AssetGateway {
  list(path?: string): Promise<unknown[]>;
  readText(path: string): Promise<string>;
  writeText(path: string, content: string): Promise<void>;
  remove(path: string): Promise<void>;
  copy(path: string, targetFolder: string): Promise<unknown>;
  move(path: string, targetFolder: string): Promise<unknown>;
  openFolder(path?: string): Promise<void>;
}
