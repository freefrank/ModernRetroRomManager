import { FolderOpen, Check, Gamepad2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useState } from "react";
import { Button, Dialog } from "@/components/ui";

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

  const dirsWithMetadata = subDirectories.filter((d) => d.metadata_files.length > 0);
  const dirsWithoutMetadata = subDirectories.filter((d) => d.metadata_files.length === 0);

  return (
    <Dialog
      open={isOpen}
      onClose={onClose}
      title={
        <span className="flex items-center gap-3">
          <span className="w-10 h-10 bg-accent-primary/20 rounded-[var(--radius-md)] flex items-center justify-center shrink-0">
            <FolderOpen className="w-5 h-5 text-accent-primary" />
          </span>
          <span className="min-w-0">
            <span className="block text-lg font-bold text-text-primary">
              {t("rootDirectoryDialog.title")}
            </span>
            <span className="block text-sm font-normal text-text-secondary">
              {t("rootDirectoryDialog.subtitle", { count: subDirectories.length })}
            </span>
          </span>
        </span>
      }
      footer={
        <Button variant="ghost" onClick={onClose}>
          {t("common.cancel")}
        </Button>
      }
    >
      <p className="text-sm text-text-secondary mb-4">
        {t("rootDirectoryDialog.description")}
      </p>

      <button
        type="button"
        onClick={onImportAsRoot}
        className={
          "w-full p-4 rounded-[var(--radius-md)] border-[length:var(--border-width)] text-left mb-4 " +
          "transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)] " +
          "bg-accent-primary/10 border-border-highlight hover:bg-accent-primary/20 " +
          "focus-visible:outline-none flex items-center gap-4"
        }
      >
        <div className="w-12 h-12 bg-accent-primary/20 rounded-[var(--radius-md)] flex items-center justify-center shrink-0">
          <Check className="w-6 h-6 text-accent-primary" />
        </div>
        <div className="flex-1">
          <div className="font-medium text-text-primary">
            {t("rootDirectoryDialog.importAsRoot")}
          </div>
          <div className="text-sm text-text-secondary">
            {t("rootDirectoryDialog.importAsRootDesc")}
          </div>
        </div>
      </button>

      {dirsWithMetadata.length > 0 && (
        <div className="mb-4">
          <h3 className="text-sm font-medium text-text-secondary mb-2">
            {t("rootDirectoryDialog.withMetadata")}
          </h3>
          <div className="space-y-2">
            {dirsWithMetadata.map((dir) => (
              <div key={dir.path} className="border-[length:var(--border-width)] border-border-default rounded-[var(--radius-md)] overflow-hidden">
                <button
                  type="button"
                  onClick={() => setExpandedDir(expandedDir === dir.path ? null : dir.path)}
                  aria-expanded={expandedDir === dir.path}
                  className="w-full p-3 flex items-center justify-between bg-bg-secondary hover:bg-bg-tertiary transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)] focus-visible:outline-none"
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
                  <div className="p-3 bg-bg-primary border-t-[length:var(--border-width)] border-border-default space-y-2">
                    {dir.metadata_files.map((file) => (
                      <button
                        key={file.file_path}
                        type="button"
                        onClick={() => onSelectSubDirectory(dir, file.format)}
                        className={
                          "w-full p-3 rounded-[var(--radius-sm)] bg-bg-secondary border-[length:var(--border-width)] border-border-default " +
                          "hover:bg-accent-primary/10 hover:border-border-highlight " +
                          "transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)] " +
                          "focus-visible:outline-none flex items-center gap-3 text-left"
                        }
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
            {t("rootDirectoryDialog.withoutMetadata", { count: dirsWithoutMetadata.length })}
          </h3>
          <div className="text-xs text-text-muted p-3 bg-bg-secondary rounded-[var(--radius-sm)]">
            {dirsWithoutMetadata.slice(0, 5).map((d) => d.name).join(", ")}
            {dirsWithoutMetadata.length > 5 &&
              t("rootDirectoryDialog.andMore", { count: dirsWithoutMetadata.length - 5 })}
          </div>
        </div>
      )}
    </Dialog>
  );
}
