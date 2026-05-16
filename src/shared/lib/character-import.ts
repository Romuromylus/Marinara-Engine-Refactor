export interface EmbeddedLorebookPreview {
  name: string;
  embeddedLorebookEntries: number;
}

export async function inspectCharacterFilesForEmbeddedLorebooks(_files: File[]): Promise<EmbeddedLorebookPreview[]> {
  return [];
}
