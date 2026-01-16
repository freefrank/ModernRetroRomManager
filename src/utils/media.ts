import { convertFileSrc } from "@tauri-apps/api/core";

/**
 * 将本地文件路径转换为前端可访问的 URL
 * @param path 本地文件路径
 */
export function getMediaUrl(path: string | undefined): string | undefined {
  if (!path) return undefined;
  return convertFileSrc(path);
}
