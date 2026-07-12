import { describe, it, expect, afterEach } from "vitest";
import { render, cleanup } from "@testing-library/react";
import "@/i18n";
import { Spinner } from "./Spinner";

afterEach(cleanup);

describe("Spinner", () => {
  it("渲染 rr-spinner 类与默认尺寸", () => {
    const { container } = render(<Spinner />);
    const el = container.querySelector("svg")!;
    expect(el.classList.contains("rr-spinner")).toBe(true);
    expect(el.getAttribute("width")).toBe("20");
    expect(el.getAttribute("role")).toBe("status");
  });

  it("size 与 className 透传", () => {
    const { container } = render(<Spinner size={16} className="extra-class" />);
    const el = container.querySelector("svg")!;
    expect(el.getAttribute("width")).toBe("16");
    expect(el.classList.contains("extra-class")).toBe(true);
  });
});
