export default function Scraper() {
  return (
    <div className="flex flex-col h-full">
      {/* 工具栏 */}
      <div className="flex items-center justify-between px-6 py-4 border-b border-border-default">
        <h1 className="text-xl font-semibold">Scraper</h1>
        <button className="px-4 py-2 bg-accent-primary hover:bg-accent-primary/90 text-white rounded-lg transition-colors">
          开始 Scrape
        </button>
      </div>

      {/* API 配置 */}
      <div className="flex-1 p-6 overflow-auto">
        <div className="max-w-3xl">
          <h2 className="text-lg font-medium mb-4">API 配置</h2>
          <p className="text-text-secondary mb-6">
            配置 Scraper API 密钥以获取游戏元数据和媒体资源
          </p>

          <div className="space-y-4">
            {/* IGDB */}
            <div className="p-4 bg-bg-secondary border border-border-default rounded-lg">
              <div className="flex items-center justify-between mb-4">
                <div className="flex items-center gap-3">
                  <div className="w-10 h-10 bg-purple-500/20 rounded-lg flex items-center justify-center">
                    <span className="text-purple-400 font-bold">IG</span>
                  </div>
                  <div>
                    <h3 className="font-medium">IGDB</h3>
                    <p className="text-sm text-text-secondary">Twitch 旗下，数据全面</p>
                  </div>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" className="sr-only peer" />
                  <div className="w-11 h-6 bg-bg-tertiary rounded-full peer peer-checked:bg-accent-primary peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all"></div>
                </label>
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm text-text-secondary mb-1">Client ID</label>
                  <input
                    type="text"
                    placeholder="Twitch Client ID"
                    className="w-full px-3 py-2 bg-bg-primary border border-border-default rounded-lg focus:outline-none focus:border-accent-primary text-sm"
                  />
                </div>
                <div>
                  <label className="block text-sm text-text-secondary mb-1">Client Secret</label>
                  <input
                    type="password"
                    placeholder="Twitch Client Secret"
                    className="w-full px-3 py-2 bg-bg-primary border border-border-default rounded-lg focus:outline-none focus:border-accent-primary text-sm"
                  />
                </div>
              </div>
            </div>

            {/* SteamGridDB */}
            <div className="p-4 bg-bg-secondary border border-border-default rounded-lg">
              <div className="flex items-center justify-between mb-4">
                <div className="flex items-center gap-3">
                  <div className="w-10 h-10 bg-blue-500/20 rounded-lg flex items-center justify-center">
                    <span className="text-blue-400 font-bold">SG</span>
                  </div>
                  <div>
                    <h3 className="font-medium">SteamGridDB</h3>
                    <p className="text-sm text-text-secondary">高质量封面/Logo/图标</p>
                  </div>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" className="sr-only peer" />
                  <div className="w-11 h-6 bg-bg-tertiary rounded-full peer peer-checked:bg-accent-primary peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all"></div>
                </label>
              </div>
              <div>
                <label className="block text-sm text-text-secondary mb-1">API Key</label>
                <input
                  type="password"
                  placeholder="SteamGridDB API Key"
                  className="w-full px-3 py-2 bg-bg-primary border border-border-default rounded-lg focus:outline-none focus:border-accent-primary text-sm"
                />
              </div>
            </div>

            {/* TheGamesDB */}
            <div className="p-4 bg-bg-secondary border border-border-default rounded-lg">
              <div className="flex items-center justify-between mb-4">
                <div className="flex items-center gap-3">
                  <div className="w-10 h-10 bg-green-500/20 rounded-lg flex items-center justify-center">
                    <span className="text-green-400 font-bold">TG</span>
                  </div>
                  <div>
                    <h3 className="font-medium">TheGamesDB</h3>
                    <p className="text-sm text-text-secondary">社区驱动，免费</p>
                  </div>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" className="sr-only peer" />
                  <div className="w-11 h-6 bg-bg-tertiary rounded-full peer peer-checked:bg-accent-primary peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all"></div>
                </label>
              </div>
              <div>
                <label className="block text-sm text-text-secondary mb-1">API Key</label>
                <input
                  type="password"
                  placeholder="TheGamesDB API Key"
                  className="w-full px-3 py-2 bg-bg-primary border border-border-default rounded-lg focus:outline-none focus:border-accent-primary text-sm"
                />
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
