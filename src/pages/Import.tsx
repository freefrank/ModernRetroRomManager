export default function Import() {
  return (
    <div className="flex flex-col h-full">
      {/* 工具栏 */}
      <div className="flex items-center justify-between px-6 py-4 border-b border-border-default">
        <h1 className="text-xl font-semibold">导入/导出</h1>
      </div>

      {/* 内容区 */}
      <div className="flex-1 p-6 overflow-auto">
        <div className="max-w-3xl">
          {/* 导入 */}
          <section className="mb-8">
            <h2 className="text-lg font-medium mb-4">导入</h2>
            <p className="text-text-secondary mb-6">
              从现有的 ROM 管理软件导入元数据和媒体资源
            </p>

            <div className="grid grid-cols-2 gap-4">
              {/* EmulationStation */}
              <button className="p-4 bg-bg-secondary border border-border-default rounded-lg hover:border-accent-primary transition-colors text-left">
                <div className="flex items-center gap-3 mb-2">
                  <div className="w-10 h-10 bg-orange-500/20 rounded-lg flex items-center justify-center">
                    <span className="text-orange-400 font-bold">ES</span>
                  </div>
                  <div>
                    <h3 className="font-medium">EmulationStation</h3>
                    <p className="text-sm text-text-secondary">gamelist.xml</p>
                  </div>
                </div>
              </button>

              {/* metadata.txt */}
              <button className="p-4 bg-bg-secondary border border-border-default rounded-lg hover:border-accent-primary transition-colors text-left">
                <div className="flex items-center gap-3 mb-2">
                  <div className="w-10 h-10 bg-cyan-500/20 rounded-lg flex items-center justify-center">
                    <span className="text-cyan-400 font-bold">MT</span>
                  </div>
                  <div>
                    <h3 className="font-medium">Pegasus/Recalbox</h3>
                    <p className="text-sm text-text-secondary">metadata.txt</p>
                  </div>
                </div>
              </button>

              {/* LaunchBox */}
              <button className="p-4 bg-bg-secondary border border-border-default rounded-lg hover:border-accent-primary transition-colors text-left">
                <div className="flex items-center gap-3 mb-2">
                  <div className="w-10 h-10 bg-blue-500/20 rounded-lg flex items-center justify-center">
                    <span className="text-blue-400 font-bold">LB</span>
                  </div>
                  <div>
                    <h3 className="font-medium">LaunchBox</h3>
                    <p className="text-sm text-text-secondary">XML</p>
                  </div>
                </div>
              </button>

              {/* RetroArch */}
              <button className="p-4 bg-bg-secondary border border-border-default rounded-lg hover:border-accent-primary transition-colors text-left">
                <div className="flex items-center gap-3 mb-2">
                  <div className="w-10 h-10 bg-red-500/20 rounded-lg flex items-center justify-center">
                    <span className="text-red-400 font-bold">RA</span>
                  </div>
                  <div>
                    <h3 className="font-medium">RetroArch</h3>
                    <p className="text-sm text-text-secondary">.lpl</p>
                  </div>
                </div>
              </button>
            </div>
          </section>

          {/* 导出 */}
          <section>
            <h2 className="text-lg font-medium mb-4">导出</h2>
            <p className="text-text-secondary mb-6">
              将 ROM 库导出为其他格式以便在其他软件中使用
            </p>

            <div className="grid grid-cols-2 gap-4">
              {/* EmulationStation */}
              <button className="p-4 bg-bg-secondary border border-border-default rounded-lg hover:border-accent-primary transition-colors text-left">
                <div className="flex items-center gap-3 mb-2">
                  <div className="w-10 h-10 bg-orange-500/20 rounded-lg flex items-center justify-center">
                    <span className="text-orange-400 font-bold">ES</span>
                  </div>
                  <div>
                    <h3 className="font-medium">EmulationStation</h3>
                    <p className="text-sm text-text-secondary">gamelist.xml</p>
                  </div>
                </div>
              </button>

              {/* metadata.txt */}
              <button className="p-4 bg-bg-secondary border border-border-default rounded-lg hover:border-accent-primary transition-colors text-left">
                <div className="flex items-center gap-3 mb-2">
                  <div className="w-10 h-10 bg-cyan-500/20 rounded-lg flex items-center justify-center">
                    <span className="text-cyan-400 font-bold">MT</span>
                  </div>
                  <div>
                    <h3 className="font-medium">Pegasus/Recalbox</h3>
                    <p className="text-sm text-text-secondary">metadata.txt</p>
                  </div>
                </div>
              </button>
            </div>
          </section>
        </div>
      </div>
    </div>
  );
}
