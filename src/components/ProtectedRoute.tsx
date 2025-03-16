import React from 'react';
import { Navigate } from 'react-router-dom';
import { useAuth } from '../contexts/AuthContext';

interface ProtectedRouteProps {
  children: React.ReactNode;
}

const ProtectedRoute: React.FC<ProtectedRouteProps> = ({ children }) => {
  const { isLoggedIn } = useAuth();

  if (!isLoggedIn) {
    // 如果用户未登录，重定向到登录页面
    return <Navigate to="/account" replace />;
  }

  // 如果用户已登录，渲染子组件
  return <>{children}</>;
};

export default ProtectedRoute;