import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { Loader2, FileText, Tag, ArrowUp, ArrowDown } from "lucide-react";
import { clsx } from "clsx";
import type { CheckResult } from "@/stores/cnRomToolsStore";
import { Badge } from "@/components/ui";
import { useColumnResize } from "./useColumnResize";

export type SortOrder = "asc" | "desc" | null;

interface Props {
  results: CheckResult[];
  sortOrder: SortOrder;
  onToggleSort: () => void;
  isSettingName: boolean;
  isAddingTag: boolean;
  onSetAsRomName: () => void;
  onAddAsTag: () => void;
  onSaveEnglishName: (file: string, value: string) => void;
  onSaveExtractedCnName: (file: string, value: string) => void;
}

/** 根据置信度返回 Badge 变体:≥90 success / ≥60 warning / 其余 error */
const confidenceVariant = (confidence: number): "success" | "warning" | "error" =>
  confidence >= 90 ? "success" : confidence >= 60 ? "warning" : "error";

/** 扫描结果表格:列宽拖拽、置信度排序、行内编辑 */
export default function ScanResultTable({
  results,
  sortOrder,
  onToggleSort,
  isSettingName,
  isAddingTag,
  onSetAsRomName,
  onAddAsTag,
  onSaveEnglishName,
  onSaveExtractedCnName,
}: Props) {
  const { t } = useTranslation();

  // 编辑状态管理
  const [editingFile, setEditingFile] = useState<string | null>(null);
  const [editingValue, setEditingValue] = useState("");
  const [editingExtractedFile, setEditingExtractedFile] = useState<string | null>(null);
  const [editingExtractedValue, setEditingExtractedValue] = useState("");

  // 列宽状态(百分比)
  const { columnWidths, handleResizeStart } = useColumnResize([25, 25, 25, 25]);

  // 排序后的结果
  const sortedResults = useMemo(() => {
    if (!sortOrder) return results;

    return [...results].sort((a, b) => {
      const confA = a.confidence ?? -1; // 没有置信度的排在最后
      const confB = b.confidence ?? -1;
      return sortOrder === "asc" ? confA - confB : confB - confA;
    });
  }, [results, sortOrder]);

  // 开始编辑英文名
  const handleStartEdit = (file: string, currentValue: string) => {
    setEditingExtractedFile(null);
    setEditingExtractedValue("");
    setEditingFile(file);
    setEditingValue(currentValue || "");
  };

  const handleStartEditExtracted = (file: string, currentValue: string) => {
    setEditingFile(null);
    setEditingValue("");
    setEditingExtractedFile(file);
    setEditingExtractedValue(currentValue || "");
  };

  // 确认编辑
  const handleConfirmEdit = (file: string) => {
    if (editingFile !== file) return;
    onSaveEnglishName(file, editingValue.trim());
    setEditingFile(null);
    setEditingValue("");
  };

  const handleConfirmEditExtracted = (file: string) => {
    if (editingExtractedFile !== file) return;
    onSaveExtractedCnName(file, editingExtractedValue.trim());
    setEditingExtractedFile(null);
    setEditingExtractedValue("");
  };

  // 取消编辑
  const handleCancelEdit = () => {
    setEditingFile(null);
    setEditingValue("");
  };

  const handleCancelEditExtracted = () => {
    setEditingExtractedFile(null);
    setEditingExtractedValue("");
  };

  const headerActionClass =
    "px-2 py-1 text-[10px] rounded-[var(--radius-sm)] transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)] disabled:opacity-50 normal-case font-medium";

  const editInputClass =
    "w-full bg-bg-tertiary border-[length:var(--border-width)] border-accent-primary rounded-[var(--radius-sm)] px-2 py-1 text-sm text-text-primary focus:outline-none";

  return (
    <div className="flex-1 min-h-0 overflow-auto custom-scrollbar">
      <table className="w-full text-left text-sm table-fixed">
        <thead className="sticky top-0 z-10">
          <tr className="bg-bg-tertiary border-b border-border-default text-xs uppercase tracking-wider text-text-muted">
            <th className="px-6 py-3 font-bold relative" style={{ width: `${columnWidths[0]}%` }}>
              {t("cnRomTools.table.fileName")}
              <div
                className="absolute top-0 right-0 w-1 h-full cursor-col-resize hover:bg-accent-primary/50 transition-colors duration-[var(--motion-fast)]"
                onMouseDown={(e) => handleResizeStart(0, e)}
              />
            </th>
            <th className="px-6 py-3 font-bold relative" style={{ width: `${columnWidths[1]}%` }}>
              {t("cnRomTools.table.romName")}
              <div
                className="absolute top-0 right-0 w-1 h-full cursor-col-resize hover:bg-accent-primary/50 transition-colors duration-[var(--motion-fast)]"
                onMouseDown={(e) => handleResizeStart(1, e)}
              />
            </th>
            <th className="px-6 py-3 font-bold relative" style={{ width: `${columnWidths[2]}%` }}>
              <div className="flex items-center justify-between">
                <span>{t("cnRomTools.table.extractedCnName")}</span>
                <button
                  onClick={onSetAsRomName}
                  disabled={isSettingName}
                  className={clsx(
                    headerActionClass,
                    "bg-accent-success/20 text-accent-success hover:bg-accent-success/30",
                  )}
                  title={t("cnRomTools.table.setAsRomNameTitle")}
                >
                  {isSettingName ? (
                    <Loader2 className="w-3 h-3 animate-spin" />
                  ) : (
                    <FileText className="w-3 h-3 inline mr-1" />
                  )}
                  {t("cnRomTools.table.setAsRomName")}
                </button>
              </div>
              <div
                className="absolute top-0 right-0 w-1 h-full cursor-col-resize hover:bg-accent-primary/50 transition-colors duration-[var(--motion-fast)]"
                onMouseDown={(e) => handleResizeStart(2, e)}
              />
            </th>
            <th className="px-6 py-3 font-bold relative" style={{ width: `${columnWidths[3]}%` }}>
              <div className="flex items-center justify-between">
                <button
                  onClick={onToggleSort}
                  className="flex items-center gap-1 hover:text-accent-primary transition-colors duration-[var(--motion-fast)] cursor-pointer"
                  title={t("cnRomTools.table.clickToSort")}
                >
                  <span>{t("cnRomTools.table.matchedEnglishName")}</span>
                  {sortOrder === "desc" && <ArrowDown className="w-3 h-3" />}
                  {sortOrder === "asc" && <ArrowUp className="w-3 h-3" />}
                </button>
                <button
                  onClick={onAddAsTag}
                  disabled={isAddingTag}
                  className={clsx(
                    headerActionClass,
                    "bg-accent-primary/20 text-accent-primary hover:bg-accent-primary/30",
                  )}
                  title={t("cnRomTools.table.addAsTagTitle")}
                >
                  {isAddingTag ? (
                    <Loader2 className="w-3 h-3 animate-spin" />
                  ) : (
                    <Tag className="w-3 h-3 inline mr-1" />
                  )}
                  {t("cnRomTools.table.addAsTag")}
                </button>
              </div>
            </th>
          </tr>
        </thead>
        <tbody className="divide-y divide-border-default">
          {sortedResults.map((res, idx) => {
            const isMissing = !res.english_name;
            const isEditing = editingFile === res.file;
            const isEditingExtracted = editingExtractedFile === res.file;

            return (
              <tr
                key={idx}
                className={clsx(
                  "hover:bg-bg-tertiary/30 transition-colors duration-[var(--motion-fast)]",
                  isMissing && "bg-accent-error/5",
                )}
              >
                <td
                  className="px-6 py-3 text-text-primary font-mono text-xs truncate"
                  title={res.file}
                >
                  {res.file}
                </td>

                <td
                  className={clsx(
                    "px-6 py-3 font-medium truncate",
                    res.name && res.name !== res.file
                      ? "text-text-primary"
                      : "text-text-muted italic",
                  )}
                >
                  {res.name === res.file ? t("cnRomTools.table.notSet") : res.name}
                </td>

                <td className="px-6 py-3 font-medium truncate">
                  {isEditingExtracted ? (
                    <input
                      type="text"
                      value={editingExtractedValue}
                      onChange={(e) => setEditingExtractedValue(e.target.value)}
                      onKeyDown={(e) => {
                        if (e.key === "Enter") {
                          handleConfirmEditExtracted(res.file);
                        } else if (e.key === "Escape") {
                          handleCancelEditExtracted();
                        }
                      }}
                      onBlur={() => handleConfirmEditExtracted(res.file)}
                      onFocus={(e) => e.target.select()}
                      autoFocus
                      className={editInputClass}
                    />
                  ) : (
                    <button
                      onClick={() => handleStartEditExtracted(res.file, res.extracted_cn_name || "")}
                      className={clsx(
                        "w-full text-left truncate hover:underline cursor-pointer",
                        res.extracted_cn_name ? "text-accent-success" : "text-text-muted italic",
                      )}
                      title={t("cnRomTools.table.clickToEdit")}
                    >
                      {res.extracted_cn_name || "-"}
                    </button>
                  )}
                </td>

                <td className="px-6 py-3 font-medium truncate">
                  {isEditing ? (
                    <input
                      type="text"
                      value={editingValue}
                      onChange={(e) => setEditingValue(e.target.value)}
                      onKeyDown={(e) => {
                        if (e.key === "Enter") {
                          handleConfirmEdit(res.file);
                        } else if (e.key === "Escape") {
                          handleCancelEdit();
                        }
                      }}
                      onBlur={() => handleConfirmEdit(res.file)}
                      onFocus={(e) => e.target.select()}
                      autoFocus
                      className={editInputClass}
                    />
                  ) : (
                    <span className="flex items-center gap-2">
                      <button
                        onClick={() => handleStartEdit(res.file, res.english_name || "")}
                        className={clsx(
                          "min-w-0 flex-1 text-left truncate hover:underline cursor-pointer",
                          res.english_name ? "text-accent-primary" : "text-text-muted italic",
                        )}
                        title={
                          res.confidence
                            ? t("cnRomTools.table.confidenceWithEdit", {
                                confidence: res.confidence.toFixed(1),
                              })
                            : t("cnRomTools.table.clickToEdit")
                        }
                      >
                        {res.english_name || t("cnRomTools.table.notMatched")}
                      </button>
                      {res.confidence != null && (
                        <Badge variant={confidenceVariant(res.confidence)} className="shrink-0">
                          {Math.round(res.confidence)}%
                        </Badge>
                      )}
                    </span>
                  )}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
