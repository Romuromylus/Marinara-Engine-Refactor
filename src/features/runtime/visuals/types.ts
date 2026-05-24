import type { AvatarCrop } from "../../../shared/lib/utils";

export type CharacterMap = Map<
  string,
  {
    name: string;
    description?: string;
    personality?: string;
    backstory?: string;
    appearance?: string;
    scenario?: string;
    example?: string;
    avatarUrl: string | null;
    nameColor?: string;
    dialogueColor?: string;
    boxColor?: string;
    avatarCrop?: AvatarCrop | null;
    conversationStatus?: "online" | "idle" | "dnd" | "offline";
    conversationActivity?: string;
  }
>;

export type PersonaInfo = {
  name: string;
  description?: string;
  personality?: string;
  backstory?: string;
  appearance?: string;
  scenario?: string;
  avatarUrl?: string;
  avatarCrop?: AvatarCrop | null;
  nameColor?: string;
  dialogueColor?: string;
  boxColor?: string;
};
