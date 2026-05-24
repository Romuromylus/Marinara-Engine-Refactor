export const knowledgeSourceKeys = {
  all: ["knowledge-sources"] as const,
  list: () => [...knowledgeSourceKeys.all, "list"] as const,
};
