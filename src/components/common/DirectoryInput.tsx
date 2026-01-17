import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { clsx } from "clsx";
import { Folder, Check, X, Loader2 } from "lucide-react";

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
    placeholder = "输入目录路径...",
    className,
}: DirectoryInputProps) {
    const [validation, setValidation] = useState<PathValidation | null>(null);
    const [isValidating, setIsValidating] = useState(false);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        if (!value.trim()) {
            setValidation(null);
            setError(null);
            return;
        }

        const timer = setTimeout(async () => {
            setIsValidating(true);
            setError(null);
            try {
                const result = await invoke<PathValidation>("validate_path", { path: value });
                setValidation(result);
                if (result.exists && result.is_directory && result.readable) {
                    onValidPath?.(result);
                }
            } catch (err) {
                setError(String(err));
                setValidation(null);
            } finally {
                setIsValidating(false);
            }
        }, 500);

        return () => clearTimeout(timer);
    }, [value, onValidPath]);

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
        if (isValidating) return "验证中...";
        if (error) return `错误: ${error}`;
        if (validation) {
            if (!validation.exists) return "路径不存在";
            if (!validation.is_directory) return "不是目录";
            if (!validation.readable) return "无读取权限";
            if (!validation.writable) return "警告: 无写入权限";
            return "有效路径 ✓";
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
                    placeholder={placeholder}
                    className="flex-1 bg-transparent border-none focus:ring-0 text-sm px-3 py-3 text-text-primary placeholder:text-text-muted focus:outline-none"
                />
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
