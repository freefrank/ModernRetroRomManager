import { useTranslation } from "react-i18next";
import { Gamepad2, CheckCircle2 } from "lucide-react";
import type { Rom } from "@/types";
import { clsx } from "clsx";

interface RomListProps {
  roms: Rom[];
  selectedIds: Set<string>;
  onRomClick: (rom: Rom) => void;
  onToggleSelect: (id: string) => void;
}

export default function RomList({ roms, selectedIds, onRomClick, onToggleSelect }: RomListProps) {
  const { t } = useTranslation();

  return (
    <div className="bg-[#151621] rounded-xl border border-white/5 overflow-hidden">
      <table className="w-full text-left">
        <thead className="bg-white/5 text-text-secondary text-xs uppercase font-medium">
          <tr>
            <th className="px-6 py-4 w-12">#</th>
            <th className="px-6 py-4">{t("common.name", { defaultValue: "Name" })}</th>
            <th className="px-6 py-4">{t("common.system", { defaultValue: "System" })}</th>
            <th className="px-6 py-4">{t("common.size", { defaultValue: "Size" })}</th>
            <th className="px-6 py-4">{t("common.date", { defaultValue: "Date" })}</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-white/5">
          {roms.map((rom, index) => {
            const isSelected = selectedIds.has(rom.id);
            return (
              <tr
                key={rom.id}
                onClick={() => onRomClick(rom)}
                className={clsx(
                  "group transition-colors cursor-pointer",
                  isSelected ? "bg-accent-primary/10 hover:bg-accent-primary/20" : "hover:bg-white/5"
                )}
              >
                <td className="px-6 py-4 text-text-muted text-sm">
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      onToggleSelect(rom.id);
                    }}
                    className={clsx(
                      "w-5 h-5 rounded flex items-center justify-center border transition-colors",
                      isSelected
                        ? "bg-accent-primary border-accent-primary text-white"
                        : "bg-transparent border-white/20 text-transparent hover:border-white/50"
                    )}
                  >
                    <CheckCircle2 className="w-3 h-3" />
                  </button>
                </td>
                <td className="px-6 py-4">
                  <div className="flex items-center gap-3">
                    <div className="w-8 h-8 rounded bg-[#0B0C15] flex items-center justify-center text-text-muted">
                      <Gamepad2 className="w-4 h-4" />
                    </div>
                    <div>
                      <div className="font-medium text-white group-hover:text-accent-primary transition-colors">
                        {rom.metadata?.name || rom.filename}
                      </div>
                      <div className="text-xs text-text-muted truncate max-w-[200px]">
                        {rom.path}
                      </div>
                    </div>
                  </div>
                </td>
                <td className="px-6 py-4">
                  <span className="px-2 py-1 rounded bg-[#0B0C15] border border-white/10 text-xs font-medium text-text-secondary uppercase">
                    {rom.systemId}
                  </span>
                </td>
                <td className="px-6 py-4 text-sm text-text-secondary">
                  {Math.round((rom.size / 1024 / 1024) * 100) / 100} MB
                </td>
                <td className="px-6 py-4 text-sm text-text-muted">
                  {new Date(rom.createdAt).toLocaleDateString()}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
