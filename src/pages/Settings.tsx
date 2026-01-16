export default function Settings() {
  return (
    <div className="flex flex-col h-full">
      {/* 工具栏 */}
      <div className="flex items-center justify-between px-6 py-4 border-b border-border-default">
        <h1 className="text-xl font-semibold">设置</h1>
      </div>

      {/* 内容区 */}
      <div className="flex-1 p-6 overflow-auto">
        <div className="max-w-3xl space-y-8">
          {/* 扫描目录 */}
          <section>
            <h2 className="text-lg font-medium mb-4">扫描目录</h2>
            <p className="text-text-secondary mb-4">
              配置 ROM 扫描目录，软件会自动检测其中的 ROM 文件
            </p>

            <div className="space-y-3">
              {/* 空状态 */}
              <div className="p-4 bg-bg-secondary border border-dashed border-border-default rounded-lg text-center">
                <p className="text-text-secondary mb-3">暂未添加扫描目录</p>
                <button className="px-4 py-2 bg-accent-primary hover:bg-accent-primary/90 text-white rounded-lg transition-colors text-sm">
                  添加目录
                </button>
              </div>
            </div>
          </section>

          {/* 存储设置 */}
          <section>
            <h2 className="text-lg font-medium mb-4">存储设置</h2>

            <div className="space-y-4">
              <div>
                <label className="block text-sm text-text-secondary mb-1">数据库位置</label>
                <div className="flex gap-2">
                  <input
                    type="text"
                    value="默认位置"
                    readOnly
                    className="flex-1 px-3 py-2 bg-bg-secondary border border-border-default rounded-lg text-sm text-text-secondary"
                  />
                  <button className="px-4 py-2 bg-bg-tertiary hover:bg-border-hover text-text-primary rounded-lg transition-colors text-sm">
                    浏览
                  </button>
                </div>
              </div>

              <div>
                <label className="block text-sm text-text-secondary mb-1">媒体资源目录</label>
                <div className="flex gap-2">
                  <input
                    type="text"
                    value="默认位置"
                    readOnly
                    className="flex-1 px-3 py-2 bg-bg-secondary border border-border-default rounded-lg text-sm text-text-secondary"
                  />
                  <button className="px-4 py-2 bg-bg-tertiary hover:bg-border-hover text-text-primary rounded-lg transition-colors text-sm">
                    浏览
                  </button>
                </div>
              </div>
            </div>
          </section>

          {/* 关于 */}
          <section>
            <h2 className="text-lg font-medium mb-4">关于</h2>

            <div className="p-4 bg-bg-secondary border border-border-default rounded-lg">
              <div className="flex items-center gap-4">
                <div className="w-16 h-16 bg-accent-primary/20 rounded-xl flex items-center justify-center">
                  <span className="text-2xl font-bold text-accent-primary">MR</span>
                </div>
                <div>
                  <h3 className="text-lg font-semibold">ModernRetroRomManager</h3>
                  <p className="text-text-secondary">版本 0.1.0</p>
                  <a
                    href="https://github.com/dotslash/modern-retro-rom-manager"
                    className="text-accent-primary hover:underline text-sm"
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    GitHub
                  </a>
                </div>
              </div>
            </div>
          </section>
        </div>
      </div>
    </div>
  );
}
