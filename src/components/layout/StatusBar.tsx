export default function StatusBar() {
  return (
    <footer className="h-7 flex items-center justify-between px-4 bg-bg-secondary border-t border-border-default text-xs text-text-muted">
      <div className="flex items-center gap-4">
        <span>ROM: 0</span>
        <span>已 Scrape: 0</span>
      </div>
      <div className="flex items-center gap-4">
        <span>存储: 0 MB</span>
      </div>
    </footer>
  );
}
