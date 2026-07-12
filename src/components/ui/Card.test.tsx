import { describe, it, expect, afterEach } from "vitest";
import { render, cleanup } from "@testing-library/react";
import { Card } from "./Card";

afterEach(cleanup);

describe("Card", () => {
  it("渲染 rr-card 类与令牌样式", () => {
    const { container } = render(<Card>内容</Card>);
    const el = container.firstElementChild as HTMLElement;
    expect(el.classList.contains("rr-card")).toBe(true);
    expect(el.className).toContain("rounded-[var(--radius-lg)]");
    expect(el.className).toContain("[box-shadow:var(--shadow-card)]");
    expect(el.textContent).toBe("内容");
  });

  it("透传 className", () => {
    const { container } = render(<Card className="extra-class">x</Card>);
    const el = container.firstElementChild as HTMLElement;
    expect(el.classList.contains("extra-class")).toBe(true);
  });
});
