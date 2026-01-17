import { motion, AnimatePresence } from "framer-motion";
import { X, FolderOpen, Check, Gamepad2 } from "lucide-react";
import { clsx } from "clsx";
import { useTranslation } from "react-i18next";
import { useState } from "react";

interface MetadataFileInfo {
  format: string;
  format_name: string;
  file_path: string;
  file_name: string;
}

interface SubDirectoryInfo {
  name: string;
  path: string;
  metadata_files: MetadataFileInfo[];
}

interface RootDirectoryDialogProps {
  isOpen: boolean;
  onClose: () => void;
  subDirectories: SubDirectoryInfo[];
  onImportAsRoot: () => void;
  onSelectSubDirectory: (subDir: SubDirectoryInfo, format: string) => void;
}

export default function RootDirectoryDialog({
  isOpen,
  onClose,
  subDirectories,
  onImportAsRoot,
  onSelectSubDirectory,
}: RootDirectoryDialogProps) {
  const { t } = useTranslation();
  const [expandedDir, setExpandedDir] = useState<string | null>(null);

  if (!isOpen) return null;

  const dirsWithMetadata = subDirectories.filter((d) => d.metadata_files.length > 0);
  const dirsWithoutMetadata = subDirectories.filter((d) => d.metadata_files.length === 0);

  return (
    <AnimatePresence>
      <div className="fixed inset-0 z-[60] flex items-center justify-center p-4">
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          onClick={onClose}
          className="absolute inset-0 bg-black/80 backdrop-blur-sm"
        />

        <motion.div
          initial={{ scale: 0.9, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
          exit={{ scale: 0.9, opacity: 0 }}
          onClick={(e) => e.stopPropagation()}
          className="relative w-full max-w-lg bg-bg-primary border border-border-default rounded-2xl shadow-2xl overflow-hidden max-h-[80vh] flex flex-col"
        >
          <div className="p-6 border-b border-border-default flex items-center justify-between flex-shrink-0">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 bg-accent-primary/20 rounded-lg flex items-center justify-center">
                <FolderOpen className="w-5 h-5 text-accent-primary" />
              </div>
              <div>
                <h2 className="text-lg font-bold text-text-primary">
                  {t("rootDirectoryDialog.title", { defaultValue: "检测到游戏平台目录" })}
                </h2>
                <p className="text-sm text-text-secondary">
                  {t("rootDirectoryDialog.subtitle", { defaultValue: "发现 {{count}} 个平台目录", count: subDirectories.length })}
                </p>
              </div>
            </div>
            <button
              onClick={onClose}
              className="p-2 rounded-lg hover:bg-bg-tertiary text-text-secondary"
            >
              <X className="w-5 h-5" />
            </button>
          </div>

          <div className="p-6 overflow-y-auto flex-1">
            <p className="text-sm text-text-secondary mb-4">
              {t("rootDirectoryDialog.description", { defaultValue: "选择导入方式：作为根目录自动扫描所有平台，或单独选择某个平台" })}
            </p>

            <button
              onClick={onImportAsRoot}
              className={clsx(
                "w-full p-4 rounded-xl border transition-all text-left mb-4",
                "bg-accent-primary/10 border-accent-primary hover:bg-accent-primary/20",
                "flex items-center gap-4"
              )}
            >
              <div className="w-12 h-12 bg-accent-primary/20 rounded-lg flex items-center justify-center">
                <Check className="w-6 h-6 text-accent-primary" />
              </div>
              <div className="flex-1">
                <div className="font-medium text-text-primary">
                  {t("rootDirectoryDialog.importAsRoot", { defaultValue: "导入为根目录" })}
                </div>
                <div className="text-sm text-text-secondary">
                  {t("rootDirectoryDialog.importAsRootDesc", { defaultValue: "自动扫描所有子目录的 metadata" })}
                </div>
              </div>
            </button>

            {dirsWithMetadata.length > 0 && (
              <div className="mb-4">
                <h3 className="text-sm font-medium text-text-secondary mb-2">
                  {t("rootDirectoryDialog.withMetadata", { defaultValue: "检测到 Metadata 的平台" })}
                </h3>
                <div className="space-y-2">
                  {dirsWithMetadata.map((dir) => (
                    <div key={dir.path} className="border border-border-default rounded-xl overflow-hidden">
                      <button
                        onClick={() => setExpandedDir(expandedDir === dir.path ? null : dir.path)}
                        className="w-full p-3 flex items-center justify-between bg-bg-secondary hover:bg-bg-tertiary transition-colors"
                      >
                        <div className="flex items-center gap-3">
                          <Gamepad2 className="w-5 h-5 text-text-muted" />
                          <span className="font-medium text-text-primary">{dir.name}</span>
                          <span className="text-xs text-text-muted">
                            ({dir.metadata_files.map((f) => f.format_name).join(", ")})
                          </span>
                        </div>
                        <span className="text-text-muted">{expandedDir === dir.path ? "−" : "+"}</span>
                      </button>
                      
                      {expandedDir === dir.path && (
                        <div className="p-3 bg-bg-primary border-t border-border-default space-y-2">
                          {dir.metadata_files.map((file) => (
                            <button
                              key={file.file_path}
                              onClick={() => onSelectSubDirectory(dir, file.format)}
                              className="w-full p-3 rounded-lg bg-bg-secondary hover:bg-accent-primary/10 hover:border-accent-primary border border-border-default transition-all flex items-center gap-3 text-left"
                            >
                              <Gamepad2 className="w-4 h-4 text-text-muted" />
                              <div className="flex-1">
                                <div className="text-sm font-medium text-text-primary">{file.format_name}</div>
                                <div className="text-xs text-text-muted">{file.file_name}</div>
                              </div>
                            </button>
                          ))}
                        </div>
                      )}
                    </div>
                  ))}
                </div>
              </div>
            )}

            {dirsWithoutMetadata.length > 0 && (
              <div>
                <h3 className="text-sm font-medium text-text-muted mb-2">
                  {t("rootDirectoryDialog.withoutMetadata", { defaultValue: "无 Metadata 的目录 ({{count}} 个)", count: dirsWithoutMetadata.length })}
                </h3>
                <div className="text-xs text-text-muted p-3 bg-bg-secondary rounded-lg">
                  {dirsWithoutMetadata.slice(0, 5).map((d) => d.name).join(", ")}
                  {dirsWithoutMetadata.length > 5 && ` ... 等 ${dirsWithoutMetadata.length - 5} 个`}
                </div>
              </div>
            )}
          </div>

          <div className="p-6 border-t border-border-default flex justify-end flex-shrink-0">
            <button
              onClick={onClose}
              className="px-6 py-2 rounded-xl text-text-secondary hover:text-text-primary hover:bg-bg-tertiary transition-colors text-sm font-medium"
            >
              {t("common.cancel", { defaultValue: "取消" })}
            </button>
          </div>
        </motion.div>
      </div>
    </AnimatePresence>
  );
}
