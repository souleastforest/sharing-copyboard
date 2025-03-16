import React, { useState } from 'react';
import { Input, Button, Card, List, Typography, Space, DatePicker, Select, Tag, message, Tooltip } from 'antd';
import { SearchOutlined, CopyOutlined, DeleteOutlined, PushpinOutlined, FilterOutlined, ClearOutlined } from '@ant-design/icons';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import type { RangePickerProps } from 'antd/es/date-picker';

const { Title, Text } = Typography;
const { RangePicker } = DatePicker;
const { Option } = Select;

interface ClipboardItem {
  id: string;
  content: string;
  title: string;
  created_at: number;
  updated_at: number;
  is_pinned: boolean;
}

const SearchPage: React.FC = () => {
  const [clipboardItems, setClipboardItems] = useState<ClipboardItem[]>([]);
  const [searchText, setSearchText] = useState('');
  const [dateRange, setDateRange] = useState<[number, number] | null>(null);
  const [sortBy, setSortBy] = useState<string>('date_desc');
  const [filterPinned, setFilterPinned] = useState<boolean | null>(null);
  
  // 高级搜索功能
  const handleSearch = () => {
    // 这里将来会实现与后端的搜索逻辑
    message.info('搜索功能已触发');
  };

  // 重置搜索条件
  const resetSearch = () => {
    setSearchText('');
    setDateRange(null);
    setSortBy('date_desc');
    setFilterPinned(null);
    message.success('搜索条件已重置');
  };

  // 日期范围选择器变更
  const onDateRangeChange: RangePickerProps['onChange'] = (dates, dateStrings) => {
    if (dates) {
      setDateRange([dates[0]!.valueOf(), dates[1]!.valueOf()]);
    } else {
      setDateRange(null);
    }
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

  // 过滤和排序剪贴板项目
  const filteredAndSortedItems = clipboardItems
    .filter(item => {
      // 文本搜索过滤
      const textMatch = searchText ? 
        item.content.toLowerCase().includes(searchText.toLowerCase()) || 
        item.title.toLowerCase().includes(searchText.toLowerCase()) : true;
      
      // 日期范围过滤
      const dateMatch = dateRange ? 
        item.created_at >= dateRange[0] && item.created_at <= dateRange[1] : true;
      
      // 置顶状态过滤
      const pinnedMatch = filterPinned !== null ? 
        item.is_pinned === filterPinned : true;
      
      return textMatch && dateMatch && pinnedMatch;
    })
    .sort((a, b) => {
      // 排序逻辑
      switch (sortBy) {
        case 'date_asc':
          return a.created_at - b.created_at;
        case 'date_desc':
          return b.created_at - a.created_at;
        case 'title_asc':
          return a.title.localeCompare(b.title);
        case 'title_desc':
          return b.title.localeCompare(a.title);
        default:
          return b.created_at - a.created_at;
      }
    });

  return (
    <Space direction="vertical" style={{ width: '100%' }} size="large">
      <Card title="高级搜索" style={{ width: '100%' }}>
        <Space direction="vertical" style={{ width: '100%' }} size="middle">
          <Input.Search
            placeholder="搜索内容或标题"
            value={searchText}
            onChange={(e) => setSearchText(e.target.value)}
            enterButton={<Button icon={<SearchOutlined />}>搜索</Button>}
            onSearch={handleSearch}
            allowClear
            size="large"
          />
          
          <Space style={{ width: '100%', justifyContent: 'space-between' }} wrap>
            <Space>
              <Text>时间范围:</Text>
              <RangePicker onChange={onDateRangeChange} />
            </Space>
            
            <Space>
              <Text>排序方式:</Text>
              <Select 
                value={sortBy} 
                onChange={setSortBy} 
                style={{ width: 150 }}
              >
                <Option value="date_desc">时间降序</Option>
                <Option value="date_asc">时间升序</Option>
                <Option value="title_asc">标题升序</Option>
                <Option value="title_desc">标题降序</Option>
              </Select>
            </Space>
            
            <Space>
              <Text>置顶状态:</Text>
              <Select 
                value={filterPinned} 
                onChange={setFilterPinned} 
                style={{ width: 120 }}
                allowClear
                placeholder="全部"
              >
                <Option value={true}>已置顶</Option>
                <Option value={false}>未置顶</Option>
              </Select>
            </Space>
            
            <Button 
              icon={<ClearOutlined />} 
              onClick={resetSearch}
            >
              重置
            </Button>
          </Space>
        </Space>
      </Card>
      
      <Card title={`搜索结果 (${filteredAndSortedItems.length})`}>
        {filteredAndSortedItems.length === 0 ? (
          <div style={{ textAlign: 'center', padding: '40px 0' }}>
            <Text type="secondary">暂无匹配的剪贴板内容</Text>
          </div>
        ) : (
          <List
            itemLayout="vertical"
            dataSource={filteredAndSortedItems}
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
                  <Tooltip title={item.is_pinned ? "已置顶" : "未置顶"}>
                    <Button 
                      icon={<PushpinOutlined />} 
                      type={item.is_pinned ? "primary" : "default"}
                      disabled
                    >
                      {item.is_pinned ? "已置顶" : "未置顶"}
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
      </Card>
    </Space>
  );
};

export default SearchPage;