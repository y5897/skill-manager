import { Routes, Route, NavLink } from "react-router-dom";
import { useTheme } from "./context/ThemeContext";
import ErrorBoundary from "./components/ui/ErrorBoundary";
import Dashboard from "./pages/Dashboard";
import AgentsPage from "./pages/AgentsPage";
import SkillsPage from "./pages/SkillsPage";
import SkillDetail from "./pages/SkillDetail";

const navItems = [
  { to: "/", label: "仪表盘", icon: "📊" },
  { to: "/agents", label: "Agent 管理", icon: "🤖" },
  { to: "/skills", label: "Skill 资源库", icon: "🧩" },
];

function App() {
  const { theme, toggle } = useTheme();

  return (
    <div className="min-h-screen flex">
      <nav className="w-56 bg-white dark:bg-gray-900 border-r border-gray-200 dark:border-gray-800 p-4 flex flex-col gap-1">
        <div className="flex items-center justify-between mb-4 px-2">
          <h1 className="text-lg font-bold text-gray-800 dark:text-gray-100">
            Skill Manager
          </h1>
          <button
            onClick={toggle}
            className="text-lg cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-800 rounded p-1 transition-colors"
            title={theme === "dark" ? "切换到亮色模式" : "切换到暗色模式"}
          >
            {theme === "dark" ? "☀️" : "🌙"}
          </button>
        </div>

        {navItems.map(({ to, label, icon }) => (
          <NavLink
            key={to}
            to={to}
            end={to === "/"}
            className={({ isActive }) =>
              `flex items-center gap-2 px-3 py-2 rounded-md text-sm transition-colors ${
                isActive
                  ? "bg-blue-50 dark:bg-blue-900/40 text-blue-700 dark:text-blue-300 font-medium"
                  : "text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800"
              }`
            }
          >
            <span>{icon}</span>
            {label}
          </NavLink>
        ))}

        <div className="mt-auto text-xs text-gray-400 dark:text-gray-600 px-2 pt-4 border-t border-gray-100 dark:border-gray-800">
          v0.1.0
        </div>
      </nav>
      <main className="flex-1 p-6 overflow-auto scrollbar-thin">
        <ErrorBoundary>
          <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/agents" element={<AgentsPage />} />
            <Route path="/skills" element={<SkillsPage />} />
            <Route path="/skills/:id" element={<SkillDetail />} />
          </Routes>
        </ErrorBoundary>
      </main>
    </div>
  );
}

export default App;