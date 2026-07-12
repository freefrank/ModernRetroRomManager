import { describe, it, expect, afterEach } from "vitest";
import { render, cleanup, fireEvent } from "@testing-library/react";
import { useState } from "react";
import { Input } from "./Input";

afterEach(cleanup);

function Controlled() {
  const [v, setV] = useState("");
  return <Input value={v} onChange={(e) => setV(e.target.value)} />;
}

describe("Input", () => {
  it("渲染 rr-input 类", () => {
    const { container } = render(<Input placeholder="搜索" />);
    const el = container.querySelector("input")!;
    expect(el.classList.contains("rr-input")).toBe(true);
    expect(el.className).toContain("rounded-[var(--radius-md)]");
  });

  it("受控输入与 className 透传", () => {
    const { container } = render(<Controlled />);
    const el = container.querySelector("input")!;
    fireEvent.change(el, { target: { value: "mario" } });
    expect(el.value).toBe("mario");

    const { container: c2 } = render(<Input className="extra-class" />);
    expect(c2.querySelector("input")!.classList.contains("extra-class")).toBe(true);
  });
});
