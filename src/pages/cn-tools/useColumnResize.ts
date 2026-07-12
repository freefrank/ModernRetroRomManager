import { useEffect, useState } from "react";

/** 表格列宽拖拽调整(百分比宽度,拖动时联动下一列保持总宽 100%) */
export function useColumnResize(initialWidths: number[]) {
  const [columnWidths, setColumnWidths] = useState(initialWidths);
  const [resizingColumn, setResizingColumn] = useState<number | null>(null);
  const [startX, setStartX] = useState(0);
  const [startWidth, setStartWidth] = useState(0);

  // 开始调整列宽
  const handleResizeStart = (columnIndex: number, e: React.MouseEvent) => {
    e.preventDefault();
    setResizingColumn(columnIndex);
    setStartX(e.clientX);
    setStartWidth(columnWidths[columnIndex]);
  };

  // 调整列宽中
  useEffect(() => {
    if (resizingColumn === null) return;

    const handleMouseMove = (e: MouseEvent) => {
      const diff = e.clientX - startX;
      const tableWidth = document.querySelector("table")?.offsetWidth || 1000;
      const diffPercent = (diff / tableWidth) * 100;

      const newWidths = [...columnWidths];
      const newWidth = Math.max(10, Math.min(50, startWidth + diffPercent));
      const oldWidth = columnWidths[resizingColumn];
      const delta = newWidth - oldWidth;

      newWidths[resizingColumn] = newWidth;

      // 调整下一列的宽度以保持总宽度为100%
      if (resizingColumn < columnWidths.length - 1) {
        newWidths[resizingColumn + 1] = Math.max(10, columnWidths[resizingColumn + 1] - delta);
      }

      setColumnWidths(newWidths);
    };

    const handleMouseUp = () => {
      setResizingColumn(null);
    };

    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);

    return () => {
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
    };
  }, [resizingColumn, startX, startWidth, columnWidths]);

  return { columnWidths, handleResizeStart };
}
