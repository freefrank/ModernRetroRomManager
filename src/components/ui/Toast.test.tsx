import { describe, it, expect, afterEach, beforeEach, vi } from "vitest";
import { render, cleanup, act, screen } from "@testing-library/react";
import "@/i18n";
import { Toast, Toaster, toast, useToastStore } from "./Toast";

beforeEach(() => {
  useToastStore.setState({ toasts: [] });
});
afterEach(() => {
  cleanup();
  vi.useRealTimers();
});

describe("Toast", () => {
  it("渲染 rr-toast 类与变体差异", () => {
    const { container: ok } = render(<Toast type="success">已保存</Toast>);
    const { container: err } = render(<Toast type="error">失败</Toast>);
    const okEl = ok.querySelector(".rr-toast") as HTMLElement;
    const errEl = err.querySelector(".rr-toast") as HTMLElement;
    expect(okEl).not.toBeNull();
    expect(okEl.className).toContain("border-accent-success/50");
    expect(errEl.className).toContain("border-accent-error/50");
  });

  it("透传 className", () => {
    const { container } = render(
      <Toast type="info" className="extra-class">
        提示
      </Toast>,
    );
    expect(container.querySelector(".rr-toast")!.classList.contains("extra-class")).toBe(true);
  });
});

describe("Toaster + toast API", () => {
  it("push 后显示,4 秒后自动消失", () => {
    vi.useFakeTimers();
    render(<Toaster />);
    act(() => {
      toast.success("操作成功");
    });
    expect(screen.getByText("操作成功")).not.toBeNull();
    act(() => {
      vi.advanceTimersByTime(4000);
    });
    expect(screen.queryByText("操作成功")).toBeNull();
  });

  it("多条 toast 堆叠显示", () => {
    render(<Toaster />);
    act(() => {
      toast.info("第一条");
      toast.error("第二条");
    });
    expect(document.querySelectorAll(".rr-toast").length).toBe(2);
  });
});
