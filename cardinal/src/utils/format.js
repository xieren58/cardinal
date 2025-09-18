// Format bytes into KB with one decimal place (legacy function)
export function formatKB(bytes) {
  if (bytes == null || !isFinite(bytes)) return null;
  const kb = bytes / 1024;
  return `${kb.toFixed(kb < 10 ? 1 : 0)} KB`;
}
