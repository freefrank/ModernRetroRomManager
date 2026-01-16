import { Outlet } from "react-router-dom";
import Sidebar from "./Sidebar";
import StatusBar from "./StatusBar";

export default function Layout() {
  return (
    <div className="h-screen flex flex-col bg-bg-primary text-text-primary relative overflow-hidden">
      {/* Background Ambience */}
      <div className="absolute top-0 left-0 w-full h-full overflow-hidden pointer-events-none z-0">
        <div className="absolute top-[-10%] left-[-10%] w-[40%] h-[40%] bg-accent-primary/20 rounded-full blur-[120px] mix-blend-screen animate-pulse"></div>
        <div className="absolute bottom-[-10%] right-[-10%] w-[40%] h-[40%] bg-accent-secondary/10 rounded-full blur-[120px] mix-blend-screen"></div>
      </div>

      <div className="flex-1 flex overflow-hidden relative z-10">
        <Sidebar />
        <main className="flex-1 overflow-hidden relative flex flex-col">
          {/* Main Content Area */}
          <div className="flex-1 overflow-y-auto custom-scrollbar p-6">
            <Outlet />
          </div>
        </main>
      </div>
      
      <div className="relative z-20">
        <StatusBar />
      </div>
    </div>
  );
}
