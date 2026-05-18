export interface SpotifyGateway {
  player<T = unknown>(input: { agentId?: string | null }): Promise<T>;
  playlists<T = unknown>(input: { agentId?: string | null; limit?: number | null }): Promise<T>;
  playlistTracks<T = unknown>(input: Record<string, unknown>): Promise<T>;
  searchTracks<T = unknown>(input: Record<string, unknown>): Promise<T>;
  playTrack<T = unknown>(input: Record<string, unknown>): Promise<T>;
  play<T = unknown>(input: Record<string, unknown>): Promise<T>;
  volume<T = unknown>(input: Record<string, unknown>): Promise<T>;
}

export interface HapticGateway {
  command<T = unknown>(input: Record<string, unknown>): Promise<T>;
  stopAll<T = unknown>(): Promise<T>;
}

export interface CustomToolsGateway {
  execute<T = unknown>(input: { toolName: string; arguments: unknown }): Promise<T>;
}

export interface IntegrationGateway {
  spotify: SpotifyGateway;
  haptic: HapticGateway;
  customTools: CustomToolsGateway;
}
