import React, { createContext, useState, useContext, useEffect, ReactNode } from 'react';
import { message } from 'antd';

interface UserInfo {
  username: string;
  email: string;
  avatar?: string;
}

interface AuthContextType {
  isLoggedIn: boolean;
  userInfo: UserInfo | null;
  login: (username: string, email: string) => void;
  logout: () => void;
  updateUserInfo: (info: Partial<UserInfo>) => void;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export const useAuth = () => {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
};

interface AuthProviderProps {
  children: ReactNode;
}

export const AuthProvider: React.FC<AuthProviderProps> = ({ children }) => {
  const [isLoggedIn, setIsLoggedIn] = useState<boolean>(false);
  const [userInfo, setUserInfo] = useState<UserInfo | null>(null);

  // 从本地存储加载用户信息
  useEffect(() => {
    const storedAuth = localStorage.getItem('auth');
    if (storedAuth) {
      try {
        const authData = JSON.parse(storedAuth);
        setIsLoggedIn(true);
        setUserInfo(authData.userInfo);
      } catch (error) {
        console.error('Failed to parse auth data:', error);
        localStorage.removeItem('auth');
      }
    }
  }, []);

  // 登录
  const login = (username: string, email: string) => {
    const user: UserInfo = {
      username,
      email
    };
    // 生成一个模拟的token，实际应用中这应该从后端获取
    const token = `token_${Math.random().toString(36).substring(2, 15)}`;
    
    setIsLoggedIn(true);
    setUserInfo(user);
    
    // 保存到本地存储，包括token
    localStorage.setItem('auth', JSON.stringify({ userInfo: user, token }));
    message.success('登录成功');
  };

  // 登出
  const logout = () => {
    setIsLoggedIn(false);
    setUserInfo(null);
    localStorage.removeItem('auth');
    message.success('已退出登录');
  };

  // 更新用户信息
  const updateUserInfo = (info: Partial<UserInfo>) => {
    if (userInfo) {
      const updatedInfo = { ...userInfo, ...info };
      setUserInfo(updatedInfo);
      
      // 获取当前存储的auth数据，保留token
      const storedAuth = localStorage.getItem('auth');
      let token = '';
      if (storedAuth) {
        try {
          const authData = JSON.parse(storedAuth);
          token = authData.token || '';
        } catch (error) {
          console.error('Failed to parse auth data:', error);
        }
      }
      
      // 保存更新后的用户信息和token
      localStorage.setItem('auth', JSON.stringify({ userInfo: updatedInfo, token }));
    }
  };

  const value = {
    isLoggedIn,
    userInfo,
    login,
    logout,
    updateUserInfo
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
};