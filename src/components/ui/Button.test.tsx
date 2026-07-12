import { describe, it, expect, afterEach } from "vitest";
import { render, cleanup } from "@testing-library/react";
import "@/i18n";
import { Button } from "./Button";

afterEach(cleanup);

describe("Button", () => {
  it("渲染 rr-button 类与主变体样式", () => {
    const { container } = render(<Button>确定</Button>);
    const btn = container.querySelector("button")!;
    expect(btn.classList.contains("rr-button")).toBe(true);
    expect(btn.className).toContain("bg-accent-primary");
    expect(btn.textContent).toBe("确定");
  });

  it("变体 class 有差异并透传 className", () => {
    const { container } = render(
      <Button variant="ghost" className="extra-class">
        取消
      </Button>,
    );
    const btn = container.querySelector("button")!;
    expect(btn.className).toContain("border-border-default");
    expect(btn.className).not.toContain("bg-accent-primary");
    expect(btn.classList.contains("extra-class")).toBe(true);
  });

  it("danger 变体与 sm 尺寸", () => {
    const { container } = render(
      <Button variant="danger" size="sm">
        删除
      </Button>,
    );
    const btn = container.querySelector("button")!;
    expect(btn.className).toContain("text-accent-error");
    expect(btn.className).toContain("h-8");
  });

  it("loading 时禁用并渲染 Spinner", () => {
    const { container } = render(<Button loading>保存</Button>);
    const btn = container.querySelector("button")!;
    expect(btn.disabled).toBe(true);
    expect(container.querySelector(".rr-spinner")).not.toBeNull();
  });
});
