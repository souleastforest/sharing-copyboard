/** @jsx React.createElement */
import React from 'react';
import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import HomePage from '../HomePage';

// 模拟tauri API
jest.mock('@tauri-apps/api/core', () => ({
  invoke: jest.fn(),
}));

jest.mock('@tauri-apps/api/event', () => ({
  listen: jest.fn(() => Promise.resolve(() => {})),
}));

// 模拟clipboard-manager插件
jest.mock('@tauri-apps/plugin-clipboard-manager', () => ({
  writeText: jest.fn(() => Promise.resolve()),
}));

describe('HomePage Component', () => {
  beforeEach(() => {
    // 清除所有模拟的调用记录
    jest.clearAllMocks();
  });

  test('renders HomePage correctly', () => {
    render(
      <MemoryRouter initialEntries={["/"]}>
        <HomePage />
      </MemoryRouter>
    );
    
    // 验证搜索框是否正确渲染
    expect(screen.getByPlaceholderText(/搜索剪贴板内容或标题/i)).toBeInTheDocument();
    // 验证空状态提示是否正确渲染
    expect(screen.getByText(/暂无剪贴板内容/i)).toBeInTheDocument();
  });
});