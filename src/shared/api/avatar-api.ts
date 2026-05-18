import { invokeTauri } from "./tauri-client";

export const npcAvatarApi = {
  upload: (chatId: string, name: string, avatar: string) =>
    invokeTauri<{ avatarPath: string; avatarFilePath?: string; avatarFilename?: string }>("npc_avatar_upload", {
      chatId,
      body: { name, avatar },
    }),
};
