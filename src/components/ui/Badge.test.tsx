import { describe, it, expect, afterEach } from "vitest";
import { render, cleanup } from "@testing-library/react";
import { Badge } from "./Badge";

afterEach(cleanup);

describe("Badge", () => {
  it("渲染 rr-badge 类与 sm 圆角令牌", () => {
    const { container } = render(<Badge>默认</Badge>);
    const el = container.querySelector(".rr-badge") as HTMLElement;
    expect(el).not.toBeNull();
    expect(el.className).toContain("rounded-[var(--radius-sm)]");
    expect(el.className).toContain("bg-bg-tertiary");
  });

  it("变体 class 有差异", () => {
    const { container: ok } = render(<Badge variant="success">高</Badge>);
    const { container: warn } = render(<Badge variant="warning">中</Badge>);
    const { container: err } = render(<Badge variant="error">低</Badge>);
    expect(ok.querySelector(".rr-badge")!.className).toContain("text-accent-success");
    expect(warn.querySelector(".rr-badge")!.className).toContain("text-accent-warning");
    expect(err.querySelector(".rr-badge")!.className).toContain("text-accent-error");
  });

  it("透传 className", () => {
    const { container } = render(<Badge className="extra-class">x</Badge>);
    expect(container.querySelector(".rr-badge")!.classList.contains("extra-class")).toBe(true);
  });
});
