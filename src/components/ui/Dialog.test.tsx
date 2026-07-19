import { describe, it, expect, afterEach, vi } from "vitest";
import { render, cleanup, fireEvent, screen } from "@testing-library/react";
import "@/i18n";
import { Dialog } from "./Dialog";

afterEach(cleanup);

describe("Dialog", () => {
  it("open=false 不渲染", () => {
    render(
      <Dialog open={false} onClose={() => {}} title="标题">
        内容
      </Dialog>,
    );
    expect(document.querySelector(".rr-dialog")).toBeNull();
  });

  it("open=true 渲染 rr-dialog、标题与内容", () => {
    render(
      <Dialog open onClose={() => {}} title="设置" className="extra-class">
        对话内容
      </Dialog>,
    );
    const panel = document.querySelector(".rr-dialog") as HTMLElement;
    expect(panel).not.toBeNull();
    expect(panel.classList.contains("extra-class")).toBe(true);
    expect(panel.className).toContain("max-h-[80vh]");
    expect(panel.className).toContain("overflow-y-auto");
    expect(panel.className).toContain("[box-shadow:var(--shadow-dialog)]");
    expect(screen.getByText("设置")).not.toBeNull();
    expect(screen.getByText("对话内容")).not.toBeNull();
  });

  it("ESC 触发 onClose", () => {
    const onClose = vi.fn();
    render(
      <Dialog open onClose={onClose} title="t">
        x
      </Dialog>,
    );
    fireEvent.keyDown(window, { key: "Escape" });
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("遮罩点击关闭,面板点击不关闭", () => {
    const onClose = vi.fn();
    render(
      <Dialog open onClose={onClose} title="t">
        面板内容
      </Dialog>,
    );
    const dialog = document.querySelector(".rr-dialog")!;
    const backdrop = dialog.parentElement!;
    // 面板内点击不关闭
    fireEvent.mouseDown(dialog);
    fireEvent.click(dialog);
    expect(onClose).not.toHaveBeenCalled();
    // 遮罩按下并松开才关闭
    fireEvent.mouseDown(backdrop);
    fireEvent.click(backdrop);
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("拖拽选中输入框文字后在遮罩松开不关闭", () => {
    const onClose = vi.fn();
    render(
      <Dialog open onClose={onClose} title="t">
        <input aria-label="search" />
      </Dialog>,
    );
    const backdrop = document.querySelector(".rr-dialog")!.parentElement!;
    const input = screen.getByLabelText("search");
    // 在输入框内按下,拖拽到遮罩松开:click 落在遮罩但按下不在遮罩,不应关闭
    fireEvent.mouseDown(input);
    fireEvent.click(backdrop);
    expect(onClose).not.toHaveBeenCalled();
  });
});
