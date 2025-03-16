import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import { message } from "antd";
import AppLayout from "./components/Layout/AppLayout";
import HomePage from "./components/Home/HomePage";
import SearchPage from "./components/Search/SearchPage";
import NotesPage from "./components/Notes/NotesPage";
import SettingsPage from "./components/Settings/SettingsPage";
import AccountPage from "./components/Account/AccountPage";
import ProtectedRoute from "./components/ProtectedRoute";
import { AuthProvider } from "./contexts/AuthContext";
import "./App.css";




function App() {
  const [monitorActive, setMonitorActive] = useState(false);
  
  // 启动剪贴板监听
  const startMonitor = async () => {
    try {
      setMonitorActive(true);
      message.success("剪贴板监听已启动");
      await invoke("start_clipboard_monitor");
    } catch (error) {
      console.error("启动剪贴板监听失败:", error);
      message.error("启动剪贴板监听失败");
      setMonitorActive(false);
    }
  };

  // 监听剪贴板更新事件
  useEffect(() => {
    const unlisten = listen("clipboard_update", (event) => {
      const content = event.payload;
      // 这里将来会实现与后端的数据处理逻辑
      message.info("新内容已添加到剪贴板列表");
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, []);



  return (
    <AuthProvider>
      <Router>
        <AppLayout monitorActive={monitorActive} onStartMonitor={startMonitor}>
          <Routes>
            <Route path="/" element={
              <ProtectedRoute>
                <HomePage />
              </ProtectedRoute>
            } />
            <Route path="/search" element={
              <ProtectedRoute>
                <SearchPage />
              </ProtectedRoute>
            } />
            <Route path="/notes" element={
              <ProtectedRoute>
                <NotesPage />
              </ProtectedRoute>
            } />
            <Route path="/settings" element={
              <ProtectedRoute>
                <SettingsPage />
              </ProtectedRoute>
            } />
            <Route path="/account" element={<AccountPage />} />
          </Routes>
        </AppLayout>
      </Router>
    </AuthProvider>
  );

}

export default App;
