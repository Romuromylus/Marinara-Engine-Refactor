import type { IntegrationGateway } from "../../engine/capabilities/integrations";
import { spotifyApi } from "./integration-utility-api";
import { invokeTauri } from "./tauri-client";

export const integrationGateway: IntegrationGateway = {
  spotify: {
    player: (input) => spotifyApi.player(input),
    playlists: (input) =>
      spotifyApi.playlists({
        agentId: input.agentId,
        limit: input.limit ?? undefined,
    }),
    playlistTracks: (input) => spotifyApi.playlistTracks(input),
    searchTracks: (input) => spotifyApi.searchTracks(input),
    playTrack: <T = unknown>(input: Record<string, unknown>) => spotifyApi.playTrack(input) as Promise<T>,
    play: <T = unknown>(input: Record<string, unknown>) => spotifyApi.play(input) as Promise<T>,
    volume: <T = unknown>(input: Record<string, unknown>) => spotifyApi.volume(input) as Promise<T>,
  },
  haptic: {
    command: <T = unknown>(input: Record<string, unknown>) => invokeTauri<T>("haptic_command", { command: input }),
    stopAll: <T = unknown>() => invokeTauri<T>("haptic_stop_all"),
  },
  customTools: {
    execute: <T = unknown>(input: { toolName: string; arguments: unknown }) =>
      invokeTauri<T>("custom_tool_execute", { body: input }),
  },
};
