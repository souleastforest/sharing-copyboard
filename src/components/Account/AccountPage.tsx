import React, { useState } from 'react';
import { Card, Tabs, Form, Input, Button, Typography, Space, message, Divider, Avatar } from 'antd';
import { UserOutlined, LockOutlined, MailOutlined, SaveOutlined, EditOutlined } from '@ant-design/icons';
import { useAuth } from '../../contexts/AuthContext';
import { invoke } from '@tauri-apps/api/core';

const { Title, Text } = Typography;
const { TabPane } = Tabs;
const { Password } = Input;

interface UserInfo {
  username: string;
  email: string;
  avatar?: string;
}

const AccountPage: React.FC = () => {
  const [activeTab, setActiveTab] = useState('login');
  const [loginForm] = Form.useForm();
  const [registerForm] = Form.useForm();
  const [profileForm] = Form.useForm();
  const [loading, setLoading] = useState(false);
  const { isLoggedIn, userInfo, login, logout, updateUserInfo } = useAuth();
  
  // 处理登录
  const handleLogin = async (values: any) => {
    setLoading(true);
    try {
      // 这里将来会实现与后端的登录逻辑
      console.log('登录信息:', values);
      
      // 模拟登录成功
      setTimeout(() => {
        login(values.username, 'user@example.com');
        setLoading(false);
      }, 1000);
    } catch (error) {
      console.error('登录失败:', error);
      message.error('登录失败');
      setLoading(false);
    }
  };

  // 处理注册
  const handleRegister = async (values: any) => {
    setLoading(true);
    try {
      // 这里将来会实现与后端的注册逻辑
      console.log('注册信息:', values);
      
      // 模拟注册成功
      setTimeout(() => {
        message.success('注册成功，请登录');
        setActiveTab('login');
        setLoading(false);
      }, 1000);
    } catch (error) {
      console.error('注册失败:', error);
      message.error('注册失败');
      setLoading(false);
    }
  };

  // 处理个人信息更新
  const handleProfileUpdate = async (values: any) => {
    setLoading(true);
    try {
      // 调用后端API更新个人信息
      console.log('个人信息更新:', values);
      
      // 从AuthContext获取token
      const token = localStorage.getItem('auth') ? 
        JSON.parse(localStorage.getItem('auth') || '{}').token : '';
      
      if (!token) {
        throw new Error('未找到登录凭证');
      }
      
      // 添加调试信息
      console.log('准备调用invoke函数', { token, request: { username: values.username, email: values.email } });
      
      try {
        // 调用后端API
        const result = await invoke<{ success: boolean }>('update_user_profile', {
          token,
          request: {
            username: values.username,
            email: values.email
          }
        });
        
        console.log('invoke调用成功，结果:', result);
        
        if (result.success) {
          // 更新前端状态
          updateUserInfo({
            username: values.username,
            email: values.email
          });
          
          message.success('个人信息已更新');
        } else {
          throw new Error('更新失败');
        }
      } catch (invokeError) {
        console.error('invoke调用失败:', invokeError);
        throw new Error(`调用后端API失败: ${invokeError instanceof Error ? invokeError.message : String(invokeError)}`);
      }
    } catch (error) {
      console.error('更新失败:', error);
      message.error('更新失败: ' + (error instanceof Error ? error.message : String(error)));
    } finally {
      setLoading(false);
    }
  };

  // 处理密码修改
  const handlePasswordChange = async (values: any) => {
    setLoading(true);
    try {
      // 这里将来会实现与后端的密码修改逻辑
      console.log('密码修改:', values);
      
      // 模拟修改成功
      setTimeout(() => {
        message.success('密码已修改');
        setLoading(false);
      }, 1000);
    } catch (error) {
      console.error('密码修改失败:', error);
      message.error('密码修改失败');
      setLoading(false);
    }
  };

  // 处理登出
  const handleLogout = () => {
    logout();
  };

  // 渲染登录表单
  const renderLoginForm = () => (
    <Form
      form={loginForm}
      layout="vertical"
      onFinish={handleLogin}
    >
      <Form.Item
        name="username"
        label="用户名"
        rules={[{ required: true, message: '请输入用户名' }]}
      >
        <Input prefix={<UserOutlined />} placeholder="请输入用户名" />
      </Form.Item>
      
      <Form.Item
        name="password"
        label="密码"
        rules={[{ required: true, message: '请输入密码' }]}
      >
        <Password prefix={<LockOutlined />} placeholder="请输入密码" />
      </Form.Item>
      
      <Form.Item>
        <Button type="primary" htmlType="submit" loading={loading} block>
          登录
        </Button>
      </Form.Item>
      
      <div style={{ textAlign: 'center' }}>
        <Button type="link" onClick={() => setActiveTab('register')}>
          没有账号？立即注册
        </Button>
      </div>
    </Form>
  );

  // 渲染注册表单
  const renderRegisterForm = () => (
    <Form
      form={registerForm}
      layout="vertical"
      onFinish={handleRegister}
    >
      <Form.Item
        name="username"
        label="用户名"
        rules={[{ required: true, message: '请输入用户名' }]}
      >
        <Input prefix={<UserOutlined />} placeholder="请输入用户名" />
      </Form.Item>
      
      <Form.Item
        name="email"
        label="邮箱"
        rules={[
          { required: true, message: '请输入邮箱' },
          { type: 'email', message: '请输入有效的邮箱地址' }
        ]}
      >
        <Input prefix={<MailOutlined />} placeholder="请输入邮箱" />
      </Form.Item>
      
      <Form.Item
        name="password"
        label="密码"
        rules={[
          { required: true, message: '请输入密码' },
          { min: 8, message: '密码长度不能小于8位' }
        ]}
      >
        <Password prefix={<LockOutlined />} placeholder="请输入密码" />
      </Form.Item>
      
      <Form.Item
        name="confirmPassword"
        label="确认密码"
        dependencies={['password']}
        rules={[
          { required: true, message: '请确认密码' },
          ({ getFieldValue }) => ({
            validator(_, value) {
              if (!value || getFieldValue('password') === value) {
                return Promise.resolve();
              }
              return Promise.reject(new Error('两次输入的密码不一致'));
            },
          }),
        ]}
      >
        <Password prefix={<LockOutlined />} placeholder="请确认密码" />
      </Form.Item>
      
      <Form.Item>
        <Button type="primary" htmlType="submit" loading={loading} block>
          注册
        </Button>
      </Form.Item>
      
      <div style={{ textAlign: 'center' }}>
        <Button type="link" onClick={() => setActiveTab('login')}>
          已有账号？立即登录
        </Button>
      </div>
    </Form>
  );

  // 渲染个人信息表单
  const renderProfileForm = () => (
    <div>
      <div style={{ textAlign: 'center', marginBottom: 24 }}>
        <Avatar size={80} icon={<UserOutlined />} src={userInfo?.avatar} />
        <Title level={4} style={{ marginTop: 16 }}>{userInfo?.username}</Title>
        <Text type="secondary">{userInfo?.email}</Text>
      </div>
      
      <Tabs defaultActiveKey="profile">
        <TabPane tab="个人信息" key="profile">
          <Form
            form={profileForm}
            layout="vertical"
            onFinish={handleProfileUpdate}
            initialValues={{
              username: userInfo?.username,
              email: userInfo?.email
            }}
          >
            <Form.Item
              name="username"
              label="用户名"
              rules={[{ required: true, message: '请输入用户名' }]}
            >
              <Input prefix={<UserOutlined />} />
            </Form.Item>
            
            <Form.Item
              name="email"
              label="邮箱"
              rules={[
                { required: true, message: '请输入邮箱' },
                { type: 'email', message: '请输入有效的邮箱地址' }
              ]}
            >
              <Input prefix={<MailOutlined />} />
            </Form.Item>
            
            <Form.Item>
              <Button 
                type="primary" 
                htmlType="submit" 
                icon={<SaveOutlined />}
                loading={loading}
              >
                保存修改
              </Button>
            </Form.Item>
          </Form>
        </TabPane>
        
        <TabPane tab="修改密码" key="password">
          <Form
            layout="vertical"
            onFinish={handlePasswordChange}
          >
            <Form.Item
              name="oldPassword"
              label="当前密码"
              rules={[{ required: true, message: '请输入当前密码' }]}
            >
              <Password prefix={<LockOutlined />} placeholder="请输入当前密码" />
            </Form.Item>
            
            <Form.Item
              name="newPassword"
              label="新密码"
              rules={[
                { required: true, message: '请输入新密码' },
                { min: 8, message: '密码长度不能小于8位' }
              ]}
            >
              <Password prefix={<LockOutlined />} placeholder="请输入新密码" />
            </Form.Item>
            
            <Form.Item
              name="confirmPassword"
              label="确认新密码"
              dependencies={['newPassword']}
              rules={[
                { required: true, message: '请确认新密码' },
                ({ getFieldValue }) => ({
                  validator(_, value) {
                    if (!value || getFieldValue('newPassword') === value) {
                      return Promise.resolve();
                    }
                    return Promise.reject(new Error('两次输入的密码不一致'));
                  },
                }),
              ]}
            >
              <Password prefix={<LockOutlined />} placeholder="请确认新密码" />
            </Form.Item>
            
            <Form.Item>
              <Button 
                type="primary" 
                htmlType="submit" 
                loading={loading}
              >
                修改密码
              </Button>
            </Form.Item>
          </Form>
        </TabPane>
      </Tabs>
      
      <Divider />
      
      <div style={{ textAlign: 'center' }}>
        <Button danger onClick={handleLogout}>
          退出登录
        </Button>
      </div>
    </div>
  );

  return (
    <Space direction="vertical" style={{ width: '100%' }} size="large">
      <Card 
        title={isLoggedIn ? "账户管理" : "用户登录/注册"}
        style={{ maxWidth: 600, margin: '0 auto' }}
      >
        {isLoggedIn ? (
          renderProfileForm()
        ) : (
          <Tabs activeKey={activeTab} onChange={setActiveTab}>
            <TabPane tab="登录" key="login">
              {renderLoginForm()}
            </TabPane>
            <TabPane tab="注册" key="register">
              {renderRegisterForm()}
            </TabPane>
          </Tabs>
        )}
      </Card>
    </Space>
  );
};

export default AccountPage;