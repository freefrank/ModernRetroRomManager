import { motion, AnimatePresence } from "framer-motion";
import { X, Database, SkipForward, Gamepad2, Package, FileText } from "lucide-react";
import { clsx } from "clsx";
import { useTranslation } from "react-i18next";

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
    if (!isOpen) return null;

    const formatIcons: Record<string, React.ReactNode> = {
        emulationstation: <Gamepad2 className="w-6 h-6" />,
        pegasus: <Gamepad2 className="w-6 h-6" />,
        launchbox: <Package className="w-6 h-6" />,
    };

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
                    className="relative w-full max-w-md bg-bg-primary border border-border-default rounded-2xl shadow-2xl overflow-hidden"
                >
                    <div className="p-6 border-b border-border-default flex items-center justify-between">
                        <div className="flex items-center gap-3">
                            <div className="w-10 h-10 bg-accent-primary/20 rounded-lg flex items-center justify-center">
                                <Database className="w-5 h-5 text-accent-primary" />
                            </div>
                            <div>
                                <h2 className="text-lg font-bold text-text-primary">{t("metadataDialog.title")}</h2>
                                <p className="text-sm text-text-secondary">{t("metadataDialog.subtitle")}</p>
                            </div>
                        </div>
                        <button
                            onClick={onClose}
                            className="p-2 rounded-lg hover:bg-bg-tertiary text-text-secondary"
                        >
                            <X className="w-5 h-5" />
                        </button>
                    </div>

                    <div className="p-6 space-y-3">
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
                                className={clsx(
                                    "w-full p-4 rounded-xl border transition-all text-left",
                                    "bg-bg-secondary border-border-default hover:border-accent-primary hover:bg-accent-primary/10",
                                    "flex items-center gap-4 group"
                                )}
                            >
                                <div className="w-12 h-12 bg-bg-tertiary rounded-lg flex items-center justify-center text-text-secondary group-hover:bg-accent-primary/20 group-hover:text-accent-primary transition-colors">
                                    {formatIcons[file.format] || <FileText className="w-6 h-6" />}
                                </div>
                                <div className="flex-1 min-w-0">
                                    <div className="font-medium text-text-primary group-hover:text-accent-primary transition-colors">
                                        {file.format_name}
                                    </div>
                                    <div className="text-sm text-text-muted truncate">{file.file_name}</div>
                                </div>
                                <div className="text-text-muted group-hover:text-accent-primary transition-colors">
                                    →
                                </div>
                            </button>
                        ))}
                    </div>

                    <div className="p-6 border-t border-border-default flex justify-end">
                        <button
                            type="button"
                            onClick={(e) => {
                                e.preventDefault();
                                e.stopPropagation();
                                onSkip();
                            }}
                            className="flex items-center gap-2 px-6 py-2 rounded-xl text-text-secondary hover:text-text-primary hover:bg-bg-tertiary transition-colors text-sm font-medium"
                        >
                            <SkipForward className="w-4 h-4" />
                            {t("metadataDialog.skip")}
                        </button>
                    </div>
                </motion.div>
            </div>
        </AnimatePresence>
    );
}

