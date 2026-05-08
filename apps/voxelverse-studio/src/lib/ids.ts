export function toContentId(displayName: string) {
  return displayName
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "_")
    .replace(/^_+|_+$/g, "");
}

export function namespaced(namespace: string, id: string) {
  return `${namespace}:${id}`;
}
