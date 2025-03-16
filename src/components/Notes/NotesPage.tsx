import React, { useState, useEffect } from 'react';
import { List, Card, Button, Typography, Space, Input, message, Tooltip, Tag, Modal, Form } from 'antd';
import { PushpinOutlined, DeleteOutlined, EditOutlined, PlusOutlined, ExclamationCircleOutlined } from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/core';

const { Text, Title } = Typography;
const { Search } = Input;
const { confirm } = Modal;

interface Note {
  id: string;
  title: string;
  content: string;
  created_at: number;
  updated_at: number;
  is_pinned: boolean;
}

const NotesPage: React.FC = () => {
  const [notes, setNotes] = useState<Note[]>([]);
  const [searchText, setSearchText] = useState('');
  const [isModalVisible, setIsModalVisible] = useState(false);
  const [editingNote, setEditingNote] = useState<Note | null>(null);
  const [loading, setLoading] = useState(false);
  const [form] = Form.useForm();
  
  // 加载便签数据
  useEffect(() => {
    loadNotes();
  }, []);

  // 从后端加载便签数据
  const loadNotes = async () => {
    setLoading(true);
    try {
      const items = await invoke<Note[]>('get_clipboard_items');
      setNotes(Array.isArray(items) ? items : []);
    } catch (error) {
      console.error('加载便签失败:', error);
      message.error('加载便签失败: ' + (error instanceof Error ? error.message : String(error)));
      // 设置空数组确保UI不会崩溃
      setNotes([]);
    } finally {
      setLoading(false);
    }
  };
  
  // 搜索功能
  const handleSearch = (value: string) => {
    setSearchText(value);
  };

  // 过滤便签
  const filteredNotes = notes.filter(note => 
    note.content.toLowerCase().includes(searchText.toLowerCase()) || 
    note.title.toLowerCase().includes(searchText.toLowerCase())
  );

  // 置顶功能
  const togglePin = async (id: string) => {
    try {
      const noteToUpdate = notes.find(note => note.id === id);
      if (!noteToUpdate) return;
      
      const updatedNote = await invoke<Note>('update_clipboard_item', {
        id,
        is_pinned: !noteToUpdate.is_pinned
      });
      
      setNotes(prev => prev.map(note => 
        note.id === id ? updatedNote : note
      ).sort((a, b) => (b.is_pinned ? 1 : 0) - (a.is_pinned ? 1 : 0) || b.created_at - a.created_at));
      
      message.success('置顶状态已更新');
    } catch (error) {
      console.error('更新置顶状态失败:', error);
      message.error('更新置顶状态失败');
    }
  };

  // 删除功能
  const showDeleteConfirm = (id: string) => {
    confirm({
      title: '确定要删除这个便签吗？',
      icon: <ExclamationCircleOutlined />,
      content: '删除后将无法恢复',
      okText: '确定',
      okType: 'danger',
      cancelText: '取消',
      onOk() {
        deleteNote(id);
      },
    });
  };

  const deleteNote = async (id: string) => {
    try {
      await invoke('delete_clipboard_item', { id });
      setNotes(prev => prev.filter(note => note.id !== id));
      message.success('便签已删除');
    } catch (error) {
      console.error('删除便签失败:', error);
      message.error('删除便签失败');
    }
  };

  // 编辑功能
  const showEditModal = (note: Note) => {
    setEditingNote(note);
    form.setFieldsValue({
      title: note.title,
      content: note.content,
    });
    setIsModalVisible(true);
  };

  // 新建便签
  const showCreateModal = () => {
    setEditingNote(null);
    form.resetFields();
    setIsModalVisible(true);
  };

  // 处理表单提交
  const handleFormSubmit = async () => {
    try {
      const values = await form.validateFields();
      
      if (editingNote) {
        // 更新便签
        const updatedNote = await invoke<Note>('update_clipboard_item', {
          id: editingNote.id,
          content: values.content,
          title: values.title
        });
        
        setNotes(prev => prev.map(note => 
          note.id === editingNote.id ? updatedNote : note
        ));
        
        message.success('便签已更新');
      } else {
        // 创建新便签
        const newNote = await invoke<Note>('add_clipboard_item', {
          content: values.content,
          title: values.title
        });
        
        setNotes(prev => [newNote, ...prev]);
        message.success('便签已创建');
      }
      
      setIsModalVisible(false);
    } catch (error) {
      console.error('保存便签失败:', error);
      message.error('保存便签失败');
    }
  };

  return (
    <Space direction="vertical" style={{ width: '100%' }} size="large">
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <Search 
          placeholder="搜索便签内容或标题" 
          allowClear 
          enterButton="搜索" 
          size="large" 
          onSearch={handleSearch} 
          onChange={(e) => setSearchText(e.target.value)}
          style={{ width: 'calc(100% - 120px)' }}
        />
        <Button 
          type="primary" 
          icon={<PlusOutlined />} 
          size="large"
          onClick={showCreateModal}
        >
          新建便签
        </Button>
      </div>
      
      {loading ? (
        <div style={{ textAlign: 'center', padding: '40px 0' }}>
          <Text type="secondary">加载中...</Text>
        </div>
      ) : filteredNotes.length === 0 ? (
        <div style={{ textAlign: 'center', padding: '40px 0' }}>
          <Text type="secondary">暂无便签内容</Text>
        </div>
      ) : (
        <List
          grid={{ gutter: 16, xs: 1, sm: 2, md: 2, lg: 3, xl: 3, xxl: 4 }}
          dataSource={filteredNotes}
          pagination={{
            pageSize: 8,
            showSizeChanger: true,
            pageSizeOptions: ['8', '12', '16'],
            showTotal: (total) => `共 ${total} 项`
          }}
          renderItem={note => (
            <List.Item key={note.id}>
              <Card
                title={
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                    <Text ellipsis style={{ maxWidth: 'calc(100% - 30px)' }}>
                      {note.title}
                    </Text>
                    {note.is_pinned && <Tag color="blue">置顶</Tag>}
                  </div>
                }
                extra={<Text type="secondary" style={{ fontSize: '12px' }}>{new Date(note.updated_at).toLocaleString()}</Text>}
                actions={[
                  <Tooltip title={note.is_pinned ? "取消置顶" : "置顶"}>
                    <Button 
                      icon={<PushpinOutlined />} 
                      type={note.is_pinned ? "primary" : "text"}
                      onClick={() => togglePin(note.id)}
                    />
                  </Tooltip>,
                  <Tooltip title="编辑">
                    <Button 
                      icon={<EditOutlined />} 
                      onClick={() => showEditModal(note)}
                    />
                  </Tooltip>,
                  <Tooltip title="删除">
                    <Button 
                      icon={<DeleteOutlined />} 
                      danger 
                      onClick={() => showDeleteConfirm(note.id)}
                    />
                  </Tooltip>
                ]}
                hoverable
                style={{ height: '100%' }}
              >
                <div style={{ 
                  height: '120px', 
                  overflow: 'auto', 
                  whiteSpace: 'pre-wrap', 
                  wordBreak: 'break-all',
                  padding: '8px',
                  backgroundColor: '#f5f5f5',
                  borderRadius: '4px'
                }}>
                  {note.content}
                </div>
              </Card>
            </List.Item>
          )}
        />
      )}

      <Modal
        title={editingNote ? '编辑便签' : '新建便签'}
        open={isModalVisible}
        onOk={handleFormSubmit}
        onCancel={() => setIsModalVisible(false)}
        okText="保存"
        cancelText="取消"
      >
        <Form
          form={form}
          layout="vertical"
        >
          <Form.Item
            name="title"
            label="标题"
            rules={[{ required: true, message: '请输入便签标题' }]}
          >
            <Input placeholder="请输入便签标题" />
          </Form.Item>
          <Form.Item
            name="content"
            label="内容"
            rules={[{ required: true, message: '请输入便签内容' }]}
          >
            <Input.TextArea 
              placeholder="请输入便签内容" 
              autoSize={{ minRows: 4, maxRows: 8 }}
            />
          </Form.Item>
        </Form>
      </Modal>
    </Space>
  );
};

export default NotesPage;