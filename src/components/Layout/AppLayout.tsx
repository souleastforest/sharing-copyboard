import React, { ReactNode } from 'react';
import { Layout, Typography, Space, Button, Tooltip, Menu } from 'antd';
import { 
  CopyOutlined, 
  SettingOutlined, 
  SyncOutlined, 
  UserOutlined,
  HomeOutlined,
  SearchOutlined,
  BarsOutlined
} from '@ant-design/icons';
import { Link, useLocation } from 'react-router-dom';

const { Header, Content, Footer, Sider } = Layout;
const { Title } = Typography;

interface AppLayoutProps {
  children: ReactNode;
  monitorActive?: boolean;
  onStartMonitor?: () => void;
}

const AppLayout: React.FC<AppLayoutProps> = ({ 
  children, 
  monitorActive = false, 
  onStartMonitor = () => {}
}) => {
  const location = useLocation();
  const currentPath = location.pathname;

  const menuItems = [
    {
      key: '/',
      icon: <HomeOutlined />,
      label: <Link to="/">主页</Link>,
    },
    {
      key: '/search',
      icon: <SearchOutlined />,
      label: <Link to="/search">搜索</Link>,
    },
    {
      key: '/notes',
      icon: <BarsOutlined />,
      label: <Link to="/notes">便签管理</Link>,
    },
    {
      key: '/settings',
      icon: <SettingOutlined />,
      label: <Link to="/settings">设置</Link>,
    },
    {
      key: '/account',
      icon: <UserOutlined />,
      label: <Link to="/account">账户</Link>,
    },
  ];

  return (
    <Layout style={{ minHeight: '100vh' }}>
      <Sider 
        theme="light" 
        breakpoint="lg"
        collapsedWidth="0"
        width={200}
      >
        <div style={{ height: 32, margin: 16, textAlign: 'center', color: "black", fontSize: "16px"}}>
          <CopyOutlined /> Sharing-Copyboard
        </div>
        <Menu 
          mode="inline" 
          selectedKeys={[currentPath]}
          items={menuItems}
          style={{ borderRight: 0 }}
        />
      </Sider>
      <Layout>
        <Header style={{ background: '#fff', padding: '0 20px' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <Title level={4} style={{ margin: '16px 0' }}>
              {menuItems.find(item => item.key === currentPath)?.label || '主页'}
            </Title>
            <Space>
              <Tooltip title={monitorActive ? '监听中' : '启动监听'}>
                <Button 
                  type={monitorActive ? 'primary' : 'default'}
                  icon={<SyncOutlined spin={monitorActive} />}
                  onClick={onStartMonitor}
                  disabled={monitorActive}
                >
                  {monitorActive ? '监听中' : '启动监听'}
                </Button>
              </Tooltip>
            </Space>
          </div>
        </Header>
        
        <Content style={{ margin: '24px 16px 0' }}>
          <div style={{ padding: 24, minHeight: 360, background: '#fff' }}>
            {children}
          </div>
        </Content>
        
        <Footer style={{ textAlign: 'center' }}>
          Sharing-Copyboard ©{new Date().getFullYear()} Created with Tauri + React
        </Footer>
      </Layout>
    </Layout>
  );
};

export default AppLayout;