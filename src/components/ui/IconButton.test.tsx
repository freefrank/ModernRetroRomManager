import { describe, it, expect, afterEach } from "vitest";
import { render, cleanup } from "@testing-library/react";
import { IconButton } from "./IconButton";

afterEach(cleanup);

describe("IconButton", () => {
  it("渲染 rr-icon-button 类", () => {
    const { container } = render(<IconButton aria-label="测试">x</IconButton>);
    const btn = container.querySelector("button")!;
    expect(btn.classList.contains("rr-icon-button")).toBe(true);
  });

  it("变体 class 有差异", () => {
    const { container: ghost } = render(<IconButton aria-label="a">x</IconButton>);
    const { container: primary } = render(
      <IconButton aria-label="b" variant="primary">
        x
      </IconButton>,
    );
    expect(ghost.querySelector("button")!.className).not.toContain("bg-accent-primary");
    expect(primary.querySelector("button")!.className).toContain("bg-accent-primary");
  });

  it("sm 尺寸与 className 透传", () => {
    const { container } = render(
      <IconButton aria-label="c" size="sm" className="extra-class">
        x
      </IconButton>,
    );
    const btn = container.querySelector("button")!;
    expect(btn.className).toContain("h-8 w-8");
    expect(btn.classList.contains("extra-class")).toBe(true);
  });
});
