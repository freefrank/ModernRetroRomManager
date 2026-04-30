import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { clsx } from "clsx";
import { Folder, Check, X, Loader2, FolderOpen } from "lucide-react";
import { useTranslation } from "react-i18next";

interface PathValidation {
    path: string;
    exists: boolean;
    is_directory: boolean;
    readable: boolean;
    writable: boolean;
}

interface DirectoryInputProps {
    value: string;
    onChange: (value: string) => void;
    onValidPath?: (validation: PathValidation) => void;
    placeholder?: string;
    className?: string;
}

export default function DirectoryInput({
    value,
    onChange,
    onValidPath,
    placeholder,
    className,
}: DirectoryInputProps) {
    const { t } = useTranslation();
    const [validation, setValidation] = useState<PathValidation | null>(null);
    const [isValidating, setIsValidating] = useState(false);
    const [error, setError] = useState<string | null>(null);

    // 使用 ref 存储 onValidPath，避免作为依赖导致重复验证
    const onValidPathRef = useRef(onValidPath);
    onValidPathRef.current = onValidPath;

    useEffect(() => {
        if (!value.trim()) {
            setValidation(null);
            setError(null);
            return;
        }

        console.log("[DEBUG] DirectoryInput: 开始验证, value:", value);
        const timer = setTimeout(async () => {
            console.log("[DEBUG] DirectoryInput: 执行验证...");
            setIsValidating(true);
            setError(null);
            try {
                const result = await invoke<PathValidation>("validate_path", { path: value });
                console.log("[DEBUG] DirectoryInput: 验证结果:", result);
                setValidation(result);
                if (result.exists && result.is_directory && result.readable) {
                    onValidPathRef.current?.(result);
                }
            } catch (err) {
                console.error("[DEBUG] DirectoryInput: 验证错误:", err);
                setError(String(err));
                setValidation(null);
            } finally {
                setIsValidating(false);
            }
        }, 500);

        return () => clearTimeout(timer);
    }, [value]);

    const handleBrowse = async () => {
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: t("directoryInput.browseTitle"),
            });
            if (selected && typeof selected === "string") {
                onChange(selected);
            }
        } catch (err) {
            console.error("Failed to open directory picker:", err);
        }
    };

    const getStatusIcon = () => {
        if (isValidating) {
            return <Loader2 className="w-5 h-5 text-text-muted animate-spin" />;
        }
        if (!value.trim()) {
            return <Folder className="w-5 h-5 text-text-muted" />;
        }
        if (error) {
            return <X className="w-5 h-5 text-accent-error" />;
        }
        if (validation) {
            if (validation.exists && validation.is_directory && validation.readable) {
                return <Check className="w-5 h-5 text-accent-success" />;
            }
            return <X className="w-5 h-5 text-accent-error" />;
        }
        return <Folder className="w-5 h-5 text-text-muted" />;
    };

    const getStatusMessage = () => {
        if (!value.trim()) return null;
        if (isValidating) return t("directoryInput.status.validating");
        if (error) return t("directoryInput.status.error", { error });
        if (validation) {
            if (!validation.exists) return t("directoryInput.status.missing");
            if (!validation.is_directory) return t("directoryInput.status.notDirectory");
            if (!validation.readable) return t("directoryInput.status.notReadable");
            if (!validation.writable) return t("directoryInput.status.notWritable");
            return t("directoryInput.status.valid");
        }
        return null;
    };

    const isValid = validation?.exists && validation?.is_directory && validation?.readable;

    return (
        <div className={clsx("space-y-2", className)}>
            <div
                className={clsx(
                    "flex items-center bg-bg-secondary border rounded-xl transition-colors",
                    isValid ? "border-accent-success/50" : validation && !isValid ? "border-accent-error/50" : "border-border-default",
                    "focus-within:border-accent-primary"
                )}
            >
                <div className="pl-4">
                    {getStatusIcon()}
                </div>
                <input
                    type="text"
                    value={value}
                    onChange={(e) => onChange(e.target.value)}
                    placeholder={placeholder || t("directoryInput.placeholder")}
                    className="flex-1 bg-transparent border-none focus:ring-0 text-sm px-3 py-3 text-text-primary placeholder:text-text-muted focus:outline-none"
                />
                <button
                    type="button"
                    onClick={handleBrowse}
                    className="px-3 py-2 mr-1 rounded-lg hover:bg-bg-tertiary transition-colors text-text-secondary hover:text-text-primary"
                    title={t("directoryInput.browse")}
                >
                    <FolderOpen className="w-5 h-5" />
                </button>
            </div>
            {getStatusMessage() && (
                <p
                    className={clsx(
                        "text-xs px-1",
                        isValid ? "text-accent-success" : error || (validation && !isValid) ? "text-accent-error" : "text-text-muted"
                    )}
                >
                    {getStatusMessage()}
                </p>
            )}
        </div>
    );
}


