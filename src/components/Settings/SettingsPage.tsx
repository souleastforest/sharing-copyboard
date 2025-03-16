import React, { useState } from 'react';
import { Card, Form, Switch, InputNumber, Select, Button, Typography, Divider, message, Space, Radio } from 'antd';
import { SaveOutlined, SyncOutlined, LockOutlined, BgColorsOutlined } from '@ant-design/icons';

// const { Title, Text } = Typography;
const { Option } = Select;

interface SettingsFormValues {
  autoMonitor: boolean;
  monitorInterval: number;
  syncEnabled: boolean;
  syncInterval: number;
  maxStorageItems: number;
  encryptionEnabled: boolean;
  theme: string;
  language: string;
}

const SettingsPage: React.FC = () => {
  const [form] = Form.useForm();
  const [loading, setLoading] = useState(false);
  
  // 初始设置值
  const initialValues: SettingsFormValues = {
    autoMonitor: true,
    monitorInterval: 500,
    syncEnabled: true,
    syncInterval: 30,
    maxStorageItems: 100,
    encryptionEnabled: true,
    theme: 'light',
    language: 'zh_CN'
  };

  // 保存设置
  const handleSaveSettings = async (values: SettingsFormValues) => {
    setLoading(true);
    try {
      // 这里将来会实现与后端的保存逻辑
      console.log('保存设置:', values);
      message.success('设置已保存');
    } catch (error) {
      console.error('保存设置失败:', error);
      message.error('保存设置失败');
    } finally {
      setLoading(false);
    }
  };

  // 重置设置
  const handleResetSettings = () => {
    form.resetFields();
    message.info('设置已重置为默认值');
  };

  return (
    <Space direction="vertical" style={{ width: '100%' }} size="large">
      <Form
        form={form}
        layout="vertical"
        initialValues={initialValues}
        onFinish={handleSaveSettings}
      >
        <Card title={<><SyncOutlined /> 剪贴板监听设置</>} style={{ marginBottom: 16 }}>
          <Form.Item
            name="autoMonitor"
            label="自动启动监听"
            valuePropName="checked"
          >
            <Switch checkedChildren="开启" unCheckedChildren="关闭" />
          </Form.Item>
          
          <Form.Item
            name="monitorInterval"
            label="监听间隔 (毫秒)"
            tooltip="设置剪贴板监听的时间间隔，值越小响应越快，但会消耗更多资源"
            rules={[{ required: true, message: '请输入监听间隔' }]}
          >
            <InputNumber min={100} max={2000} step={100} style={{ width: 200 }} />
          </Form.Item>
        </Card>
        
        <Card title={<><SyncOutlined /> 同步设置</>} style={{ marginBottom: 16 }}>
          <Form.Item
            name="syncEnabled"
            label="启用多设备同步"
            valuePropName="checked"
          >
            <Switch checkedChildren="开启" unCheckedChildren="关闭" />
          </Form.Item>
          
          <Form.Item
            name="syncInterval"
            label="同步间隔 (秒)"
            tooltip="设置自动同步的时间间隔"
            rules={[{ required: true, message: '请输入同步间隔' }]}
          >
            <InputNumber min={5} max={300} step={5} style={{ width: 200 }} />
          </Form.Item>
          
          <Form.Item
            name="maxStorageItems"
            label="最大存储条目数"
            tooltip="设置本地最多保存的剪贴板条目数量"
            rules={[{ required: true, message: '请输入最大存储条目数' }]}
          >
            <InputNumber min={10} max={500} step={10} style={{ width: 200 }} />
          </Form.Item>
        </Card>
        
        <Card title={<><LockOutlined /> 安全设置</>} style={{ marginBottom: 16 }}>
          <Form.Item
            name="encryptionEnabled"
            label="启用数据加密"
            valuePropName="checked"
            tooltip="启用后，所有同步数据将使用AES-256加密"
          >
            <Switch checkedChildren="开启" unCheckedChildren="关闭" />
          </Form.Item>
        </Card>
        
        <Card title={<><BgColorsOutlined /> 界面设置</>} style={{ marginBottom: 16 }}>
          <Form.Item
            name="theme"
            label="界面主题"
          >
            <Radio.Group>
              <Radio.Button value="light">浅色</Radio.Button>
              <Radio.Button value="dark">深色</Radio.Button>
              <Radio.Button value="system">跟随系统</Radio.Button>
            </Radio.Group>
          </Form.Item>
          
          <Form.Item
            name="language"
            label="界面语言"
          >
            <Select style={{ width: 200 }}>
              <Option value="zh_CN">简体中文</Option>
              <Option value="en_US">English</Option>
            </Select>
          </Form.Item>
        </Card>
        
        <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 16 }}>
          <Button onClick={handleResetSettings}>恢复默认设置</Button>
          <Button 
            type="primary" 
            htmlType="submit" 
            icon={<SaveOutlined />}
            loading={loading}
          >
            保存设置
          </Button>
        </div>
      </Form>
    </Space>
  );
};

export default SettingsPage;