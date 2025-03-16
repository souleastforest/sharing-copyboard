import React, { useState } from 'react';
import { List, Card, Button, Typography, Space, Input, message, Tooltip, Tag } from 'antd';
import { PushpinOutlined, DeleteOutlined, EditOutlined, CopyOutlined } from '@ant-design/icons';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';

const { Text } = Typography;
const { Search } = Input;

interface ClipboardItem {
  id: string;
  content: string;
  title: string;
  created_at: number;
  updated_at: number;
  is_pinned: boolean;
}

const HomePage: React.FC = () => {
  const [clipboardItems, setClipboardItems] = useState<ClipboardItem[]>([]);
  const [searchText, setSearchText] = useState('');
  
  // 搜索功能
  const handleSearch = (value: string) => {
    setSearchText(value);
  };

  // 过滤剪贴板项目
  const filteredItems = clipboardItems.filter(item => 
    item.content.toLowerCase().includes(searchText.toLowerCase()) || 
    item.title.toLowerCase().includes(searchText.toLowerCase())
  );

  // 置顶功能
  const togglePin = (id: string) => {
    setClipboardItems(prev => prev.map(item => 
      item.id === id ? { ...item, is_pinned: !item.is_pinned } : item
    ).sort((a, b) => (b.is_pinned ? 1 : 0) - (a.is_pinned ? 1 : 0) || b.created_at - a.created_at));
    message.success('置顶状态已更新');
  };

  // 删除功能
  const deleteItem = (id: string) => {
    setClipboardItems(prev => prev.filter(item => item.id !== id));
    message.success('已删除');
  };

  // 复制到剪贴板
  const copyToClipboard = async (content: string) => {
    try {
      await writeText(content);
      message.success('已复制到剪贴板');
    } catch (error) {
      console.error('复制失败:', error);
      message.error('复制失败');
    }
  };

  return (
    <Space direction="vertical" style={{ width: '100%' }} size="large">
      <Search 
        placeholder="搜索剪贴板内容或标题" 
        allowClear 
        enterButton="搜索" 
        size="large" 
        onSearch={handleSearch} 
        onChange={(e) => setSearchText(e.target.value)}
      />
      
      {filteredItems.length === 0 ? (
        <div style={{ textAlign: 'center', padding: '40px 0' }}>
          <Text type="secondary">暂无剪贴板内容</Text>
        </div>
      ) : (
        <List
          itemLayout="vertical"
          dataSource={filteredItems}
          pagination={{
            pageSize: 5,
            showSizeChanger: true,
            pageSizeOptions: ['5', '10', '20'],
            showTotal: (total) => `共 ${total} 项`
          }}
          renderItem={item => (
            <List.Item
              key={item.id}
              actions={[
                <Tooltip title="复制到剪贴板">
                  <Button 
                    icon={<CopyOutlined />} 
                    onClick={() => copyToClipboard(item.content)}
                  >
                    复制
                  </Button>
                </Tooltip>,
                <Tooltip title={item.is_pinned ? "取消置顶" : "置顶"}>
                  <Button 
                    icon={<PushpinOutlined />} 
                    type={item.is_pinned ? "primary" : "default"}
                    onClick={() => togglePin(item.id)}
                  >
                    {item.is_pinned ? "取消置顶" : "置顶"}
                  </Button>
                </Tooltip>,
                <Tooltip title="删除">
                  <Button 
                    icon={<DeleteOutlined />} 
                    danger 
                    onClick={() => deleteItem(item.id)}
                  >
                    删除
                  </Button>
                </Tooltip>,
                <Tooltip title="编辑">
                  <Button 
                    icon={<EditOutlined />} 
                    onClick={() => message.info("编辑功能开发中")}
                  >
                    编辑
                  </Button>
                </Tooltip>
              ]}
            >
              <Card 
                title={
                  <Space>
                    {item.title}
                    {item.is_pinned && <Tag color="blue">置顶</Tag>}
                  </Space>
                }
                extra={<Text type="secondary">{new Date(item.created_at).toLocaleString()}</Text>}
                style={{ width: "100%" }}
                hoverable
              >
                <div style={{ 
                  maxHeight: "200px", 
                  overflow: "auto", 
                  whiteSpace: "pre-wrap", 
                  wordBreak: "break-all",
                  padding: "8px",
                  backgroundColor: "#f5f5f5",
                  borderRadius: "4px"
                }}>
                  {item.content}
                </div>
              </Card>
            </List.Item>
          )}
        />
      )}
    </Space>
  );
};

export default HomePage;