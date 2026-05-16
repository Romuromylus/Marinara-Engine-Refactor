export const APP_VERSION = "0.1.0";

export interface Theme {
  id: string;
  name: string;
  css: string;
  isActive?: boolean;
  installedAt?: string;
  createdAt?: string;
  updatedAt?: string;
}
