import { Database, SkipForward, Gamepad2, Package, FileText } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button, Dialog } from "@/components/ui";

interface MetadataFileInfo {
    format: string;
    format_name: string;
    file_path: string;
    file_name: string;
}

interface MetadataImportDialogProps {
    isOpen: boolean;
    onClose: () => void;
    metadataFiles: MetadataFileInfo[];
    onImport: (file: MetadataFileInfo) => void;
    onSkip: () => void;
}

export default function MetadataImportDialog({
    isOpen,
    onClose,
    metadataFiles,
    onImport,
    onSkip,
}: MetadataImportDialogProps) {
    const { t } = useTranslation();

    const formatIcons: Record<string, React.ReactNode> = {
        emulationstation: <Gamepad2 className="w-6 h-6" />,
        pegasus: <Gamepad2 className="w-6 h-6" />,
        launchbox: <Package className="w-6 h-6" />,
    };

    return (
        <Dialog
            open={isOpen}
            onClose={onClose}
            title={
                <span className="flex items-center gap-3">
                    <span className="w-10 h-10 bg-accent-primary/20 rounded-[var(--radius-md)] flex items-center justify-center shrink-0">
                        <Database className="w-5 h-5 text-accent-primary" />
                    </span>
                    <span className="min-w-0">
                        <span className="block text-lg font-bold text-text-primary">{t("metadataDialog.title")}</span>
                        <span className="block text-sm font-normal text-text-secondary">{t("metadataDialog.subtitle")}</span>
                    </span>
                </span>
            }
            footer={
                <Button
                    variant="ghost"
                    onClick={(e) => {
                        e.preventDefault();
                        e.stopPropagation();
                        onSkip();
                    }}
                >
                    <SkipForward className="w-4 h-4" />
                    {t("metadataDialog.skip")}
                </Button>
            }
        >
            <div className="space-y-3">
                <p className="text-sm text-text-secondary mb-4">
                    {t("metadataDialog.description")}
                </p>
                {metadataFiles.map((file) => (
                    <button
                        key={file.file_path}
                        type="button"
                        onClick={(e) => {
                            e.preventDefault();
                            e.stopPropagation();
                            onImport(file);
                        }}
                        className={
                            "w-full p-4 rounded-[var(--radius-md)] border-[length:var(--border-width)] text-left " +
                            "bg-bg-secondary border-border-default hover:border-border-highlight hover:bg-accent-primary/10 " +
                            "transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)] " +
                            "focus-visible:outline-none flex items-center gap-4 group"
                        }
                    >
                        <div className="w-12 h-12 bg-bg-tertiary rounded-[var(--radius-md)] flex items-center justify-center text-text-secondary group-hover:bg-accent-primary/20 group-hover:text-accent-primary transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)] shrink-0">
                            {formatIcons[file.format] || <FileText className="w-6 h-6" />}
                        </div>
                        <div className="flex-1 min-w-0">
                            <div className="font-medium text-text-primary group-hover:text-accent-primary transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)]">
                                {file.format_name}
                            </div>
                            <div className="text-sm text-text-muted truncate">{file.file_name}</div>
                        </div>
                        <div className="text-text-muted group-hover:text-accent-primary transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)]">
                            →
                        </div>
                    </button>
                ))}
            </div>
        </Dialog>
    );
}
