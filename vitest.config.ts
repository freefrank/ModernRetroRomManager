import { defineConfig } from "vitest/config";
import path from "node:path";

export default defineConfig({
  resolve: { alias: { "@": path.resolve(__dirname, "src") } },
  test: {
    environment: "jsdom",
    include: ["src/**/*.test.{ts,tsx}"],
    // WSL /mnt 路径上 forks 池的 worker 进程孵化极慢,常出现
    // "Timeout waiting for worker to respond" 的假错;threads 池
    // 线程孵化开销小一个量级,限并发以降低 I/O 争抢
    pool: "threads",
    maxWorkers: 4,
  },
});
